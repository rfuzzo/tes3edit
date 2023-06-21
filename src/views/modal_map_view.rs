use tes3::esp::Cell;
use tes3::esp::TypeInfo;

use crate::MapData;
use crate::TemplateApp;

impl TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn update_modal_map(&mut self, ctx: &egui::Context) {
        use std::collections::HashMap;

        use tes3::esp::{Landscape, Plugin};

        use crate::{get_unique_id, EAppState};

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plugins to map");
            ui.separator();

            // Main view
            if let Some(in_path) = self.map_data.path.clone() {
                ui.horizontal(|ui| {
                    ui.label(in_path.display().to_string());
                    if ui.button("...").clicked() {
                        open_compare_folder(&mut self.map_data);
                    }
                });
                ui.separator();

                if self.map_data.plugins.is_empty() {
                    populate_plugins(&mut self.map_data);
                }

                let plugins = &mut self.map_data.plugins;
                plugins.sort_by_key(|a| a.get_name());

                // plugin select view
                for vm in plugins.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut vm.enabled, "");
                        ui.label(vm.path.file_name().unwrap().to_string_lossy());
                    });
                }
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        // go into compare mode
                        self.app_state = EAppState::Map;

                        // load plugins into memory
                        let mut cells: HashMap<(i32, i32), Cell> = HashMap::default();
                        let mut landscape: HashMap<(i32, i32), Landscape> = HashMap::default();
                        let mut cell_id_map: HashMap<String, (i32, i32)> = HashMap::default();
                        let mut land_id_map: HashMap<String, (i32, i32)> = HashMap::default();

                        for vm in self.map_data.plugins.iter_mut().filter(|e| e.enabled) {
                            let path = vm.path.clone();

                            if let Ok(plugin) = Plugin::from_path(&path) {
                                vm.plugin = plugin;

                                for c in vm.plugin.objects.iter().filter(|p| is_cell(p)) {
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
                                    cells.insert((x, y), cell);
                                    cell_id_map.insert(id, (x, y));
                                }

                                for c in vm.plugin.objects.iter().filter(|p| is_landscape(p)) {
                                    let id = get_unique_id(c);
                                    let land = Landscape::try_from(c.to_owned()).unwrap();
                                    let x = land.grid.0;
                                    let y = land.grid.1;

                                    // add cells
                                    landscape.insert((x, y), land);
                                    land_id_map.insert(id, (x, y));
                                }
                            }
                        }

                        // get final list of cells
                        self.map_data.cells = cells;
                        self.map_data.landscape = landscape;
                        self.map_data.cell_ids = cell_id_map;
                        self.map_data.land_ids = land_id_map;

                        // close modal window
                        self.toasts.success("Loaded plugins");
                        self.close_modal_window(ui);
                    }

                    if ui.button("Cancel").clicked() {
                        self.close_modal_window(ui);
                    }
                });
            } else {
                open_compare_folder(&mut self.map_data);
            }
        });
    }
}

fn is_cell(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "CELL"
}

fn is_landscape(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "LAND"
}

#[cfg(not(target_arch = "wasm32"))]
fn open_compare_folder(data: &mut MapData) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        if !path.is_dir() {
            return;
        }

        data.path = Some(path);
        populate_plugins(data);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn populate_plugins(data: &mut MapData) {
    use tes3::esp::Plugin;

    use crate::{get_path_hash, MapItemViewModel};

    if let Some(path) = data.path.clone() {
        // get plugins
        let plugins = crate::get_plugins_in_folder(&path, true)
            .iter()
            .map(|e| MapItemViewModel {
                id: get_path_hash(e),
                path: e.to_path_buf(),
                enabled: false,
                plugin: Plugin { objects: vec![] },
            })
            .collect::<Vec<_>>();

        data.plugins.clear();
        for p in plugins {
            data.plugins.push(p);
        }
    }
}
