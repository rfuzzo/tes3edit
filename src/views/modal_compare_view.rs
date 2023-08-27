use std::{env, fs, path::PathBuf};

use tes3::esp::Plugin;

use crate::{
    generate_conflict_map, get_path_hash, get_unique_id, CompareData, EAppState, TemplateApp,
};

impl TemplateApp {
    /// Returns the update modal compare of this [`TemplateApp`].
    pub(crate) fn update_modal_compare(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Logic
            if !self.compare_data.path.exists() {
                if let Ok(cwd) = env::current_dir() {
                    self.compare_data.path = cwd;
                } else {
                    self.compare_data.path = PathBuf::from("");
                }
            }
            if self.compare_data.plugins.is_empty() {
                populate_plugins(&mut self.compare_data);
            }

            // Main view
            ui.heading("Plugins to compare");
            ui.separator();
            // Header
            ui.horizontal(|ui| {
                ui.label(self.compare_data.path.display().to_string());
                if ui.button("🗁").clicked() {
                    open_compare_folder(&mut self.compare_data);
                }
            });
            ui.separator();

            if !self.compare_data.plugins.is_empty() {
                // plugin select view
                if !self.compare_data.plugins.is_empty() {
                    ui.horizontal(|ui| {
                        if ui.button("Select all").clicked() {
                            for vm in self.compare_data.plugins.iter_mut() {
                                vm.enabled = true;
                            }
                        }
                        if ui.button("Select none").clicked() {
                            for vm in self.compare_data.plugins.iter_mut() {
                                vm.enabled = false;
                            }
                        }
                    });
                }

                ui.separator();

                for vm in self.compare_data.plugins.iter_mut() {
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
                    self.app_state = EAppState::Compare;

                    // calculate conflicts
                    // load plugins into memory
                    for vm in self.compare_data.plugins.iter_mut().filter(|e| e.enabled) {
                        if let Ok(plugin) = Plugin::from_path(&vm.path.clone()) {
                            vm.plugin = plugin;
                            vm.records = vm
                                .plugin
                                .objects
                                .iter()
                                .map(get_unique_id)
                                .collect::<Vec<_>>();
                        }
                    }

                    let conflict_map = generate_conflict_map(&self.compare_data);
                    self.compare_data.map = conflict_map;
                    let mut keys = self
                        .compare_data
                        .map
                        .keys()
                        .map(|e| e.to_owned())
                        .collect::<Vec<_>>();
                    keys.sort();
                    self.compare_data.conflicting_ids = keys;

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
}

fn open_compare_folder(data: &mut CompareData) {
    let folder_option = rfd::FileDialog::new().pick_folder();
    if let Some(path) = folder_option {
        if !path.is_dir() {
            return;
        }

        data.path = path;
        populate_plugins(data);
    }
}

fn populate_plugins(data: &mut CompareData) {
    data.plugins.clear();

    // get plugins
    let plugins = crate::get_plugins_in_folder(&data.path, true)
        .iter()
        .map(|e| crate::CompareItemViewModel {
            id: get_path_hash(e),
            path: e.to_path_buf(),
            enabled: false,
            plugin: Plugin { objects: vec![] },
            records: vec![],
        })
        .collect::<Vec<_>>();

    for p in plugins {
        data.plugins.push(p);
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
