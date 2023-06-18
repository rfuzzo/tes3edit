use tes3::esp::Cell;
use tes3::esp::TypeInfo;

use crate::MapData;
use crate::TemplateApp;

impl TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn update_modal_map(&mut self, ctx: &egui::Context) {
        use std::collections::HashMap;

        use tes3::esp::Plugin;

        use crate::{get_unique_id, EAppState};

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plugins to map");
            ui.separator();

            // Main view
            // a folder to chose
            if let Some(in_path) = self.map_data.path.clone() {
                ui.horizontal(|ui| {
                    ui.label(in_path.display().to_string());
                    if ui.button("...").clicked() {
                        open_compare_folder(&mut self.map_data);
                    }
                });
                ui.separator();

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
                if ui.button("OK").clicked() {
                    // go into compare mode
                    self.app_state = EAppState::Map;

                    // load plugins into memory
                    let mut cells: HashMap<(i32, i32), Cell> = HashMap::default();
                    let mut ids: HashMap<String, (i32, i32)> = HashMap::default();
                    for vm in self.map_data.plugins.iter_mut().filter(|e| e.enabled) {
                        let path = vm.path.clone();

                        if let Ok(plugin) = Plugin::from_path(&path) {
                            vm.plugin = plugin;

                            let plugin_cells = vm
                                .plugin
                                .objects
                                .iter()
                                .filter(|p| is_cell(p))
                                .collect::<Vec<_>>();
                            for c in plugin_cells {
                                let id = get_unique_id(c);
                                let cell = to_cell(c);
                                if cell.is_interior() {
                                    continue;
                                }
                                // if cell.map_color.is_none() {
                                //     continue;
                                // }

                                let x = cell.data.grid.0;
                                let y = cell.data.grid.1;

                                if x < self.map_data.min {
                                    self.map_data.min = x;
                                }
                                if x > self.map_data.max {
                                    self.map_data.max = x;
                                }
                                if y < self.map_data.min {
                                    self.map_data.min = y;
                                }
                                if y > self.map_data.max {
                                    self.map_data.max = y;
                                }

                                // add cells
                                cells.insert((x, y), cell);
                                ids.insert(id, (x, y));
                            }
                        }
                    }

                    // get final list of cells
                    self.map_data.cells = cells;
                    self.map_data.cell_ids = ids;

                    // close modal window
                    self.toasts.success("Loaded plugins");
                    self.close_modal_window(ui);
                }
            } else {
                open_compare_folder(&mut self.map_data);
            }
        });
    }
}

// TODO cloning
fn to_cell(c: &tes3::esp::TES3Object) -> Cell {
    let cell: Cell = Cell::try_from(c.to_owned()).unwrap();
    cell
}

fn is_cell(p: &tes3::esp::TES3Object) -> bool {
    p.tag_str() == "CELL"
}

#[cfg(not(target_arch = "wasm32"))]
fn open_compare_folder(data: &mut MapData) {
    use tes3::esp::Plugin;

    use crate::{get_path_hash, MapItemViewModel};

    let folder_option = rfd::FileDialog::new().pick_folder();
    if let Some(path) = folder_option {
        if !path.is_dir() {
            return;
        }

        data.path = Some(path.clone());
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

        for p in plugins {
            data.plugins.push(p);
        }
    }
}
