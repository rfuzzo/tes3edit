use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use crate::get_plugin_id;

use crate::{
    get_plugin_names, get_unique_id, save_patch, save_plugin, EModalState, EScale, ETheme,
    PluginMetadata, TemplateApp,
};
use tes3::esp::{Header, Plugin};

impl TemplateApp {
    #[allow(unused_variables)] // for wasm
    pub fn menu_bar_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Menu Bar
        egui::menu::bar(ui, |ui| {
            // File Menu
            ui.menu_button("File", |ui| {
                // New plugin
                if ui.button("New").clicked() {
                    // insert new
                    let plugin = Plugin::new();
                    let plugin_id = "default".to_owned();
                    let mut data = PluginMetadata::new(plugin_id.clone(), None);
                    // create new header
                    let record = tes3::esp::TES3Object::from(Header::default());
                    data.records.insert(get_unique_id(&record), record);
                    self.edit_data.plugins.push(data);

                    self.edit_data.current_plugin_id = plugin_id;
                    ui.close_menu();
                }

                ui.separator();

                // Open
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Open").clicked() {
                    self.open_file_native();
                    ui.close_menu();
                }

                //  Open recent
                ui.menu_button("Open Recent", |ui| {
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

                ui.separator();

                // Save as button
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Save As").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .add_filter("omwaddon", &["omwaddon"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        // get current plugin
                        if let Some(data) = self
                            .edit_data
                            .plugins
                            .iter_mut()
                            .find(|p| p.id == self.edit_data.current_plugin_id)
                        {
                            if save_plugin(data, &path, &mut self.toasts, true) {
                                // update current path
                                data.full_path = Some(path.clone());
                                let plugin_id = get_plugin_id(data);
                                data.id = plugin_id.clone();
                                self.edit_data.current_plugin_id = plugin_id;
                                self.last_directory = path;
                            }
                        }
                    }

                    ui.close_menu();
                }

                // Save as patch button
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Save As Patch").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .add_filter("omwaddon", &["omwaddon"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        // get current plugin
                        if let Some(data) = self
                            .edit_data
                            .plugins
                            .iter_mut()
                            .find(|p| p.id == self.edit_data.current_plugin_id)
                        {
                            if save_patch(data, &path, &mut self.toasts) {
                                // update current path
                                data.full_path = Some(path.clone());
                                let plugin_id = get_plugin_id(data);
                                data.id = plugin_id.clone();
                                self.edit_data.current_plugin_id = plugin_id;
                                self.last_directory = path;
                            }
                        }
                    }

                    ui.close_menu();
                }

                ui.separator();

                // Quit button
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // View Menu
            ui.menu_button("View", |ui| {
                if self.use_experimental && ui.button("Compare View").clicked() {
                    if !self.edit_data.plugins.is_empty() {
                        self.toasts
                            .warning("Please close all open plugins before entering compare mode");
                    } else {
                        self.open_modal_window(ui, EModalState::ModalCompareInit);
                    }
                    ui.close_menu();
                }

                if ui.button("Map View").clicked() {
                    if !self.edit_data.plugins.is_empty() {
                        self.toasts
                            .warning("Please close all open plugins before entering compare mode");
                    } else {
                        self.open_modal_window(ui, EModalState::MapInit);
                    }
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Settings").clicked() {
                    self.open_modal_window(ui, EModalState::Settings);
                    ui.close_menu();
                }
            });

            ui.separator();

            // Open button
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("🖹 Open File").clicked() {
                self.open_file_native();
            }
            // Open for wasm
            #[cfg(target_arch = "wasm32")]
            if ui.button("Open File").clicked() {
                let open_data = std::rc::Rc::clone(&self.open_file_data);
                let start_directory = self.last_directory.clone();
                // async
                wasm_bindgen_futures::spawn_local(async move {
                    let file_opt = rfd::AsyncFileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .add_filter("omwaddon", &["omwaddon"])
                        .set_directory(start_directory)
                        .pick_file()
                        .await;
                    if let Some(file) = file_opt {
                        let mut plugin = Plugin::new();
                        let data = file.read().await;
                        if plugin.load_bytes(&data).is_ok() {
                            *open_data.borrow_mut() = Some((file.file_name(), plugin));
                        }
                    }
                });
            }

            // Save for wasm
            #[cfg(target_arch = "wasm32")]
            if ui.button("Save As").clicked() {
                let save_data = std::rc::Rc::clone(&self.save_file_data);
                let start_directory = self.last_directory.clone();
                // async
                wasm_bindgen_futures::spawn_local(async move {
                    let file_opt = rfd::AsyncFileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .add_filter("omwaddon", &["omwaddon"])
                        .set_directory(start_directory)
                        .pick_file()
                        .await;
                    if let Some(file) = file_opt {
                        *save_data.borrow_mut() = Some(file.file_name());
                    }
                });
            }

            if ui.button("💾 Save").clicked() {
                // get current plugin
                if let Some(data) = self
                    .edit_data
                    .plugins
                    .iter()
                    .find(|p| p.id == self.edit_data.current_plugin_id)
                {
                    if let Some(path) = &data.full_path {
                        save_plugin(data, path, &mut self.toasts, self.overwrite);
                    } else {
                        // log error
                        self.toasts.error("Please use Save As first");
                    }
                }
            }

            if ui.button("Save Patch").clicked() {
                if let Some(data) = self
                    .edit_data
                    .plugins
                    .iter()
                    .find(|p| p.id == self.edit_data.current_plugin_id)
                {
                    if let Some(path) = &data.full_path {
                        save_patch(data, path, &mut self.toasts);
                    } else {
                        // log error
                        self.toasts.error("Please use Save As first");
                    }
                }
            }

            ui.separator();

            if self.use_experimental && ui.button("↔ Compare").clicked() {
                if !self.edit_data.plugins.is_empty() {
                    self.toasts
                        .warning("Please close all open plugins before entering compare mode");
                } else {
                    self.open_modal_window(ui, EModalState::ModalCompareInit);
                }
            }

            if ui.button("🗺 Map").clicked() {
                if !self.edit_data.plugins.is_empty() {
                    self.toasts
                        .warning("Please close all open plugins before entering compare mode");
                } else {
                    self.open_modal_window(ui, EModalState::MapInit);
                }
            }

            // theme button on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                // theme
                theme_switch(ui, &mut self.theme);
                // scale
                egui::ComboBox::from_label("Scale: ")
                    .selected_text(format!("{:?}", self.scale))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.scale, EScale::Native, "Native");
                        ui.selectable_value(&mut self.scale, EScale::Small, "Small");
                        ui.selectable_value(&mut self.scale, EScale::Medium, "Medium");
                        ui.selectable_value(&mut self.scale, EScale::Large, "Large");
                    });
            });
        });
    }

    /// the tab view with all open plugins
    pub fn tab_bar(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                let plugins_sorted = get_plugin_names(&self.edit_data.plugins);

                for key in plugins_sorted {
                    let name = Path::new(&key)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();

                    // tab item view
                    // TODO fix margins, background
                    ui.push_id(key.clone(), |ui| {
                        ui.horizontal(|ui| {
                            // tab item name
                            if ui.button(name).clicked() {
                                // open Plugin
                                self.edit_data.current_plugin_id = key.clone();
                            }
                            // tab item close button
                            let close_button = ui.button("x");
                            if close_button.clicked() {
                                // remove the plugin
                                self.edit_data.current_plugin_id = "".into();

                                // get the plugin idx with the current id
                                if let Some((idx, _vm)) = self
                                    .edit_data
                                    .plugins
                                    .iter()
                                    .enumerate()
                                    .find(|p| p.1.id == key)
                                {
                                    self.edit_data.plugins.remove(idx);
                                }
                            }

                            ui.separator();
                        });
                    });
                }
            });
        });
    }
}

fn theme_switch(ui: &mut egui::Ui, theme: &mut ETheme) {
    egui::ComboBox::from_label("Theme")
        .selected_text(format!("{:?}", theme))
        .show_ui(ui, |ui| {
            ui.style_mut().wrap = Some(false);
            ui.set_min_width(60.0);
            ui.selectable_value(theme, ETheme::Latte, "LATTE");
            ui.selectable_value(theme, ETheme::Frappe, "FRAPPE");
            ui.selectable_value(theme, ETheme::Macchiato, "MACCHIATO");
            ui.selectable_value(theme, ETheme::Mocha, "MOCHA");
        });
}
