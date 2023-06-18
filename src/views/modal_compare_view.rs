use crate::CompareData;
use crate::TemplateApp;

impl TemplateApp {
    /////////////////////////////////////////////////
    // Modal views

    /// Returns the update modal compare of this [`TemplateApp`].
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn update_modal_compare(&mut self, ctx: &egui::Context) {
        use tes3::esp::Plugin;

        use crate::{generate_conflict_map, get_unique_id, EAppState};

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plugins to compare");
            ui.separator();

            // Main view
            // a folder to chose
            if let Some(in_path) = self.compare_data.path.clone() {
                ui.horizontal(|ui| {
                    ui.label(in_path.display().to_string());
                    if ui.button("...").clicked() {
                        open_compare_folder(&mut self.compare_data);
                    }
                });
                ui.separator();

                let plugins = &mut self.compare_data.plugins;
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
                    self.app_state = EAppState::Compare;

                    // calculate conflicts
                    // load plugins into memory
                    for vm in self.compare_data.plugins.iter_mut().filter(|e| e.enabled) {
                        let path = vm.path.clone();
                        if let Ok(plugin) = Plugin::from_path(&path) {
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
            } else {
                open_compare_folder(&mut self.compare_data);
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_compare_folder(data: &mut CompareData) {
    use tes3::esp::Plugin;

    use crate::get_path_hash;

    let folder_option = rfd::FileDialog::new().pick_folder();
    if let Some(path) = folder_option {
        if !path.is_dir() {
            return;
        }

        data.path = Some(path.clone());
        // get plugins
        let plugins = crate::get_plugins_in_folder(&path, true)
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
    }
}
