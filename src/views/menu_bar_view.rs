//#[cfg(target_arch = "wasm32")]
//use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, path::PathBuf};

use tes3::esp::{Plugin, TES3Object};

use crate::{get_unique_id, TemplateApp};

impl TemplateApp {
    pub fn menu_bar_view(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Menu Bar
        egui::menu::bar(ui, |ui| {
            // File Menu
            #[cfg(not(target_arch = "wasm32"))]
            ui.menu_button("File", |ui| {
                ui.menu_button("Recently Opened", |ui| {
                    for path in self.recent_plugins.clone() {
                        let label = path.display().to_string();
                        if ui.button(label).clicked() {
                            // open the plugin
                            if let Ok(plugin) = Plugin::from_path(&path) {
                                Self::open_plugin(
                                    Some(path.to_path_buf()),
                                    &mut self.last_directory,
                                    &mut self.recent_plugins,
                                    plugin,
                                    &mut self.edited_records,
                                    &mut self.records,
                                );
                            }
                        }
                    }
                });

                // Save as button
                if ui.button("Save As").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        crate::save_all(
                            &mut self.records,
                            &mut self.edited_records,
                            &path,
                            &mut self.toasts,
                        );
                        self.last_directory = path;
                    }
                }

                // Save as patch button
                if ui.button("Save As Patch").clicked() {
                    let some_path = rfd::FileDialog::new()
                        .add_filter("esp", &["esp"])
                        .set_directory(&self.last_directory)
                        .save_file();

                    if let Some(path) = some_path {
                        crate::save_patch(
                            &mut self.records,
                            &mut self.edited_records,
                            &path,
                            &mut self.toasts,
                        );
                        self.last_directory = path;
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
                    .set_directory(&self.last_directory)
                    .pick_file();

                if let Some(path) = file_option {
                    if let Ok(plugin) = Plugin::from_path(&path) {
                        Self::open_plugin(
                            Some(path),
                            &mut self.last_directory,
                            &mut self.recent_plugins,
                            plugin,
                            &mut self.edited_records,
                            &mut self.records,
                        );
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

            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save All").clicked() {
                if let Some(plugin_path) = self.recent_plugins.last() {
                    crate::save_all(
                        &mut self.records,
                        &mut self.edited_records,
                        plugin_path,
                        &mut self.toasts,
                    );
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save Patch").clicked() {
                if let Some(plugin_path) = self.recent_plugins.last() {
                    crate::save_patch(
                        &mut self.records,
                        &mut self.edited_records,
                        plugin_path,
                        &mut self.toasts,
                    );
                }
            }

            // theme button on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                global_dark_light_mode_switch(ui, &mut self.light_mode);
                ui.label("Theme: ");
                egui::warn_if_debug_build(ui);
            });
        });
    }

    pub fn open_plugin(
        path_option: Option<PathBuf>,
        last_directory: &mut PathBuf,
        recent_plugins: &mut Vec<PathBuf>,
        plugin: Plugin,
        edited_records: &mut HashMap<String, TES3Object>,
        records: &mut HashMap<String, TES3Object>,
    ) {
        // save paths if on native
        if let Some(path) = path_option {
            *last_directory = path;

            if !recent_plugins.contains(last_directory) {
                recent_plugins.push(last_directory.to_path_buf());
            }
            recent_plugins.dedup();
            if recent_plugins.len() > 10 {
                recent_plugins.remove(0);
            }
        }

        // clear old data
        edited_records.clear();
        records.clear();

        // add new data
        for record in plugin.objects {
            records.insert(get_unique_id(&record), record);
        }
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
