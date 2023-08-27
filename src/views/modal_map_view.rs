use std::{
    collections::{hash_map::Entry, HashMap},
    env, fs,
    path::PathBuf,
};

use tes3::esp::{Cell, Landscape, Npc, Plugin, Region, TypeInfo};

use crate::{
    get_path_hash, get_plugins_in_folder, get_unique_id, EAppState, MapData, MapItemViewModel,
    TemplateApp,
};

impl TemplateApp {
    pub(crate) fn update_modal_map(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Logic
            if !self.map_data.path.exists() {
                if let Ok(cwd) = env::current_dir() {
                    self.map_data.path = cwd;
                } else {
                    self.map_data.path = PathBuf::from("");
                }
            }
            if self.map_data.plugins.is_empty() {
                populate_plugins(&mut self.map_data);
            }

            // Main view
            ui.heading("Plugins to map");
            ui.separator();
            // Header
            #[cfg(not(target_arch = "wasm32"))]
            ui.horizontal(|ui| {
                ui.label(self.map_data.path.display().to_string());
                if ui.button("🗁").clicked() {
                    open_compare_folder(&mut self.map_data);
                }
            });
            ui.separator();

            if !self.map_data.plugins.is_empty() {
                // plugin select view
                if !self.map_data.plugins.is_empty() {
                    ui.horizontal(|ui| {
                        if ui.button("Select all").clicked() {
                            for vm in self.map_data.plugins.iter_mut() {
                                vm.enabled = true;
                            }
                        }
                        if ui.button("Select none").clicked() {
                            for vm in self.map_data.plugins.iter_mut() {
                                vm.enabled = false;
                            }
                        }
                    });
                }

                ui.separator();

                for vm in self.map_data.plugins.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut vm.enabled, "");
                        ui.label(vm.path.file_name().unwrap().to_string_lossy());
                    });
                }
                ui.separator();
            }

            // Buttons
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    // go into compare mode
                    self.app_state = EAppState::Map;

                    self.init_data();

                    // close modal window
                    self.toasts.success("Loaded plugins");
                    self.close_modal_window(ui);
                }

                if ui.button("Cancel").clicked() {
                    self.close_modal_window(ui);
                }
            });
        });
    }

    fn init_data(&mut self) {
        // load plugins into memory
        let mut cells: HashMap<(i32, i32), Cell> = HashMap::default();
        let mut cell_id_map: HashMap<String, (i32, i32)> = HashMap::default();
        let mut cell_conflicts: HashMap<(i32, i32), Vec<u64>> = HashMap::default();

        let mut landscape: HashMap<(i32, i32), Landscape> = HashMap::default();
        let mut land_id_map: HashMap<String, (i32, i32)> = HashMap::default();

        // TODO fix loadorder
        let mut travels: HashMap<String, Vec<(i32, i32)>> = HashMap::default();
        let mut npcs: HashMap<String, (i32, i32)> = HashMap::default();

        for vm in self.map_data.plugins.iter_mut().filter(|e| e.enabled) {
            if let Ok(plugin) = Plugin::from_path(&vm.path.clone()) {
                // add travel
                for r in plugin.objects.iter().filter(|p| is_npc(p)) {
                    let npc = Npc::try_from(r.to_owned()).unwrap();
                    let dests = npc.travel_destinations.clone();
                    if !dests.is_empty() {
                        let mut v: Vec<(i32, i32)> = vec![];
                        for d in dests {
                            let mut x = (d.translation[0] / 8192.0) as i32;
                            if x < 0 {
                                x -= 1;
                            }
                            let mut y = (d.translation[1] / 8192.0) as i32;
                            if y < 0 {
                                y -= 1;
                            }

                            v.push((x, y));
                        }
                        travels.insert(npc.id, v);
                    }
                }

                // add cells
                for c in plugin.objects.iter().filter(|p| is_cell(p)) {
                    let id = get_unique_id(c);
                    let cell = Cell::try_from(c.to_owned()).unwrap();
                    if cell.is_interior() {
                        continue;
                    }

                    let x = cell.data.grid.0;
                    let y = cell.data.grid.1;

                    if x < self.map_data.bounds_x.0 {
                        self.map_data.bounds_x.0 = x;
                    }
                    if x > self.map_data.bounds_x.1 {
                        self.map_data.bounds_x.1 = x;
                    }
                    if y < self.map_data.bounds_y.0 {
                        self.map_data.bounds_y.0 = y;
                    }
                    if y > self.map_data.bounds_y.1 {
                        self.map_data.bounds_y.1 = y;
                    }

                    // add cells
                    let key = (x, y);

                    for (npc_id, _) in travels.clone() {
                        if cell.references.iter().any(|p| p.1.id == npc_id) {
                            npcs.insert(npc_id, key);
                        }
                    }

                    if let Entry::Vacant(e) = cell_conflicts.entry(key) {
                        e.insert(vec![vm.id]);
                    } else {
                        let mut value = cell_conflicts.get(&key).unwrap().to_owned();
                        value.push(vm.id);
                        cell_conflicts.insert(key, value);
                    }

                    cells.insert(key, cell);
                    cell_id_map.insert(id, key);
                }

                // add landscape
                for l in plugin.objects.iter().filter(|p| is_landscape(p)) {
                    let land = Landscape::try_from(l.to_owned()).unwrap();
                    let key = (land.grid.0, land.grid.1);

                    // add landscape
                    landscape.insert(key, land);
                    land_id_map.insert(get_unique_id(l), key);
                }

                // add regions
                for r in plugin.objects.iter().filter(|p| is_region(p)) {
                    let region = Region::try_from(r.to_owned()).unwrap();
                    self.map_data.regions.insert(region.id.clone(), region);
                }
            }
        }

        // sort travel destinations
        let mut edges: Vec<((i32, i32), (i32, i32))> = vec![];
        for (key, start) in npcs.clone() {
            if let Some(dest) = travels.get(&key) {
                for d in dest {
                    if !edges.contains(&(*d, start)) {
                        edges.push((start, *d));
                    }
                }
            }
        }
        edges.dedup();
        self.map_data.edges = edges;

        // get final list of cells
        for (k, v) in cell_conflicts.iter().filter(|p| p.1.len() > 1) {
            self.map_data.cell_conflicts.insert(*k, v.to_vec());
        }
        self.map_data.cells = cells;
        self.map_data.landscape = landscape;
        self.map_data.cell_ids = cell_id_map;
        self.map_data.land_ids = land_id_map;

        //
    }
}

fn is_cell(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "CELL"
}

fn is_landscape(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "LAND"
}

fn is_region(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "REGN"
}

fn is_npc(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "NPC_"
}

#[cfg(not(target_arch = "wasm32"))]
fn open_compare_folder(data: &mut MapData) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        if !path.is_dir() {
            return;
        }

        data.path = path;
        populate_plugins(data);
    }
}

fn populate_plugins(data: &mut MapData) {
    data.plugins.clear();
    data.plugin_hashes.clear();

    // get plugins
    let plugins = get_plugins_in_folder(&data.path, true)
        .iter()
        .map(|e| MapItemViewModel {
            id: get_path_hash(e),
            path: e.to_path_buf(),
            enabled: false,
            //plugin: Plugin { objects: vec![] },
        })
        .collect::<Vec<_>>();

    for p in plugins {
        let id = p.id;
        let name = p.path.file_name().unwrap().to_str().unwrap().to_owned();

        data.plugins.push(p);
        data.plugin_hashes.insert(id, name);
    }

    // sort
    data.plugins.sort_by(|a, b| {
        fs::metadata(a.path.clone())
            .expect("filetime")
            .modified()
            .unwrap()
            .cmp(
                &fs::metadata(b.path.clone())
                    .expect("filetime")
                    .modified()
                    .unwrap(),
            )
    });
}
