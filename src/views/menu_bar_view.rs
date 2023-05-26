use std::path::Path;

use tes3::esp::{Header, Plugin};

use crate::{get_plugin_id, get_unique_id, EScale, ETheme, PluginMetadata, TemplateApp};

impl TemplateApp {
    #[allow(unused_variables)] // for wasm
    pub fn menu_bar_view(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Menu Bar
        egui::menu::bar(ui, |ui| {
            // File Menu
            #[cfg(not(target_arch = "wasm32"))]
            ui.menu_button("File", |ui| {
                // New plugin
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("New").clicked() {
                    // insert new
                    let plugin = Plugin::new();
                    let plugin_id = "default".to_owned();
                    let mut data = PluginMetadata::new(plugin_id.clone(), None);
                    // create new header
                    let record = tes3::esp::TES3Object::from(Header::default());
                    data.records.insert(get_unique_id(&record), record);
                    self.plugins.push(data);

                    self.current_plugin_id = plugin_id;
                    ui.close_menu();
                }

                ui.separator();

                // Open
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
                if ui.button("Save As").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .add_filter("omwaddon", &["omwaddon"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        // get current plugin
                        if let Some(plugin_data) = self
                            .plugins
                            .iter_mut()
                            .find(|p| p.id == self.current_plugin_id)
                        {
                            crate::save_plugin(
                                &plugin_data.records,
                                &plugin_data.edited_records,
                                &path,
                                &mut self.toasts,
                                true, //TODO set this to always true?
                            );

                            // update current path
                            plugin_data.full_path = Some(path.clone());
                            let plugin_id = get_plugin_id(plugin_data);
                            plugin_data.id = plugin_id.clone();
                            self.current_plugin_id = plugin_id;
                            self.last_directory = path;
                        }
                    }

                    ui.close_menu();
                }

                // Save as patch button
                if ui.button("Save As Patch").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .add_filter("esm", &["esm"])
                        .add_filter("omwaddon", &["omwaddon"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        // get current plugin
                        if let Some(plugin_data) = self
                            .plugins
                            .iter_mut()
                            .find(|p| p.id == self.current_plugin_id)
                        {
                            crate::save_patch(
                                &plugin_data.records,
                                &plugin_data.edited_records,
                                &path,
                                &mut self.toasts,
                            );

                            // update current path
                            plugin_data.full_path = Some(path.clone());
                            let plugin_id = get_plugin_id(plugin_data);
                            plugin_data.id = plugin_id.clone();
                            self.current_plugin_id = plugin_id;
                            self.last_directory = path;
                        }
                    }

                    ui.close_menu();
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

            ui.separator();

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

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save Patch").clicked() {
                if let Some(data) = self.plugins.iter().find(|p| p.id == self.current_plugin_id) {
                    if let Some(path) = &data.full_path {
                        crate::save_patch(
                            &data.records,
                            &data.edited_records,
                            path,
                            &mut self.toasts,
                        );
                    } else {
                        // log error
                        self.toasts.error("Please use Save As first");
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save").clicked() {
                // get current plugin
                if let Some(data) = self.plugins.iter().find(|p| p.id == self.current_plugin_id) {
                    if let Some(path) = &data.full_path {
                        crate::save_plugin(
                            &data.records,
                            &data.edited_records,
                            path,
                            &mut self.toasts,
                            self.overwrite,
                        );
                    } else {
                        // log error
                        self.toasts.error("Please use Save As first");
                    }
                }
            }

            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            ui.checkbox(&mut self.overwrite, "Overwrite");

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

    /// Opens a plugin
    #[cfg(not(target_arch = "wasm32"))]
    fn open_file_native(&mut self) {
        let file_option = rfd::FileDialog::new()
            .add_filter("esp", &["esp"])
            .add_filter("esm", &["esm"])
            .add_filter("omwaddon", &["omwaddon"])
            .set_directory(&self.last_directory)
            .pick_file();

        if let Some(path) = file_option {
            if let Ok(plugin) = Plugin::from_path(&path) {
                Self::open_plugin(self, Some(path), plugin);
            }
        }
    }

    /// maps the input pluginviewmodel vec as list of ids
    fn get_plugin_names(map: &[PluginMetadata]) -> Vec<String> {
        let mut plugins_sorted: Vec<String> = map.iter().map(|p| p.id.clone()).collect();
        plugins_sorted.sort();
        plugins_sorted
    }

    /// the tab view with all open plugins
    pub fn tab_bar(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

                            // get the plugin idx with the current id
                            if let Some((idx, _vm)) =
                                self.plugins.iter().enumerate().find(|p| p.1.id == key)
                            {
                                self.plugins.remove(idx);
                            }

                            ui.close_menu();
                        }
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
