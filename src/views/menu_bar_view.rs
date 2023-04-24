use std::{collections::HashMap, path::Path};

use tes3::esp::Plugin;

use crate::{EScale, TemplateApp};

impl TemplateApp {
    #[allow(unused_variables)] // for wasm
    pub fn menu_bar_view(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Menu Bar
        egui::menu::bar(ui, |ui| {
            // File Menu
            #[cfg(not(target_arch = "wasm32"))]
            ui.menu_button("File", |ui| {
                ui.menu_button("Opened Recent", |ui| {
                    for (i, path) in self.recent_plugins.clone().iter().enumerate() {
                        let label = path.display().to_string();
                        if ui.button(label).clicked() {
                            // check if file exists
                            if !path.exists() {
                                self.recent_plugins.remove(i);
                            } else {
                                // open the plugin
                                if let Ok(plugin) = Plugin::from_path(path) {
                                    Self::open_plugin(self, Some(path.to_path_buf()), plugin);
                                }
                                ui.close_menu();
                            }
                        }
                    }
                });

                // Save as button
                if ui.button("Save As").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        // get current plugin
                        if let Some(plugin_data) = self.plugins.get(&self.current_plugin_id) {
                            crate::save_all(
                                &plugin_data.records,
                                &plugin_data.edited_records,
                                &path,
                                &mut self.toasts,
                                &self.overwrite,
                            );
                            self.last_directory = path;
                        }
                    }
                }

                // Save as patch button
                if ui.button("Save As Patch").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        // get current plugin
                        if let Some(plugin_data) = self.plugins.get(&self.current_plugin_id) {
                            crate::save_patch(
                                &plugin_data.records,
                                &plugin_data.edited_records,
                                &path,
                                &mut self.toasts,
                            );
                            self.last_directory = path;
                        }
                    }
                }

                ui.separator();

                // Quit button
                if ui.button("Quit").clicked() {
                    frame.close();
                }
            });

            // Open button
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Open File").clicked() {
                let file_option = rfd::FileDialog::new()
                    .add_filter("esp", &["esp"])
                    .add_filter("esm", &["esm"])
                    .set_directory(&self.last_directory)
                    .pick_file();

                if let Some(path) = file_option {
                    if let Ok(plugin) = Plugin::from_path(&path) {
                        Self::open_plugin(self, Some(path), plugin);
                    }
                }
            }

            // Open for wasm
            #[cfg(target_arch = "wasm32")]
            if ui.button("Open File").clicked() {
                let open_pdb_data = std::rc::Rc::clone(&self.open_pdb_data);
                let start_directory = self.last_directory.clone();
                // async
                wasm_bindgen_futures::spawn_local(async move {
                    let file_opt = rfd::AsyncFileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .set_directory(start_directory)
                        .pick_file()
                        .await;
                    if let Some(file) = file_opt {
                        let mut plugin = Plugin::new();
                        let data = file.read().await;
                        if plugin.load_bytes(&data).is_ok() {
                            *open_pdb_data.borrow_mut() = Some((file.file_name(), plugin));
                        }
                    }
                });
            }

            // Save for wasm
            #[cfg(target_arch = "wasm32")]
            if ui.button("Save As").clicked() {
                // let mut dialog =
                //     egui_file::FileDialog::save_file(Some(std::path::PathBuf::from("/")));
                // dialog.open();
                // self.save_file_dialog = Some(dialog);
            }

            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save Patch").clicked() {
                if let Some((key, data)) = self.plugins.get_key_value(&self.current_plugin_id) {
                    crate::save_patch(
                        &data.records,
                        &data.edited_records,
                        Path::new(key),
                        &mut self.toasts,
                    );
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save All").clicked() {
                // get current plugin
                if let Some((key, data)) = self.plugins.get_key_value(&self.current_plugin_id) {
                    crate::save_all(
                        &data.records,
                        &data.edited_records,
                        Path::new(key),
                        &mut self.toasts,
                        &self.overwrite,
                    );
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            ui.checkbox(&mut self.overwrite, "Overwrite");

            // theme button on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                global_dark_light_mode_switch(ui, &mut self.light_mode);

                ui.label("Theme: ");
                egui::warn_if_debug_build(ui);

                egui::ComboBox::from_label("Scale!")
                    .selected_text(format!("{:?}", self.scale))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.scale, EScale::Small, "Small");
                        ui.selectable_value(&mut self.scale, EScale::Medium, "Medium");
                        ui.selectable_value(&mut self.scale, EScale::Large, "Large");
                    });
            });
        });
    }

    fn get_plugin_names(map: &HashMap<String, crate::app::PluginMetadata>) -> Vec<String> {
        let mut plugins_sorted: Vec<String> = map.iter().map(|kvp| kvp.0.clone()).collect();
        plugins_sorted.sort();

        plugins_sorted
    }

    pub fn breadcrumb(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                let plugins_sorted = Self::get_plugin_names(&self.plugins);

                for key in plugins_sorted {
                    let name = Path::new(&key)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    let r = ui.button(name);
                    if r.clicked() {
                        // open Plugin
                        self.current_plugin_id = key.clone();
                    }
                    r.context_menu(|ui| {
                        if ui.button("Close").clicked() {
                            // remove the plugin
                            self.current_plugin_id = "".into();
                            self.plugins.remove(&key);
                            ui.close_menu();
                        }
                    });
                }
            });
        });
    }
}

// taken from egui
fn global_dark_light_mode_switch(ui: &mut egui::Ui, light_mode: &mut bool) {
    let style: egui::Style = (*ui.ctx().style()).clone();
    let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
    if let Some(visuals) = new_visuals {
        *light_mode = !visuals.dark_mode;
        ui.ctx().set_visuals(visuals);
    }
}
