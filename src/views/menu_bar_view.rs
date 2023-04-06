use std::{collections::HashMap, path::PathBuf};

use egui_notify::Toasts;
use tes3::esp::{Plugin, TES3Object};

use crate::{get_unique_id, save_all, save_patch};

pub struct UiArgs<'a> {
    frame: &'a mut eframe::Frame,
    ui: &'a mut egui::Ui,
    toasts: &'a mut Toasts,
    light_mode: &'a mut bool,
}

impl<'a> UiArgs<'a> {
    pub fn new(
        frame: &'a mut eframe::Frame,
        ui: &'a mut egui::Ui,
        toasts: &'a mut Toasts,
        light_mode: &'a mut bool,
    ) -> Self {
        Self {
            frame,
            light_mode,
            toasts,
            ui,
        }
    }
}

pub(crate) fn menu_bar_view(
    // ui
    ui_args: UiArgs<'_>,
    // app
    records: &mut HashMap<String, TES3Object>,
    edited_records: &mut HashMap<String, TES3Object>,
    plugin_path: &mut PathBuf,
    recent_plugins: &mut Vec<PathBuf>,
    last_directory: &mut PathBuf,
) {
    // Menu Bar
    egui::menu::bar(ui_args.ui, |ui| {
        // File Menu
        ui.menu_button("File", |ui| {
            // todo open recent
            #[cfg(not(target_arch = "wasm32"))]
            ui.menu_button("Recently Opened", |ui| {
                for path in recent_plugins.clone() {
                    let label = path.display().to_string();
                    if ui.button(label).clicked() {
                        // open the plugin
                        open_plugin(
                            path.to_path_buf(),
                            plugin_path,
                            last_directory,
                            edited_records,
                            records,
                            recent_plugins,
                        );
                    }
                }
            });

            // Save as button
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save As").clicked() {
                let some_path = rfd::FileDialog::new()
                    .add_filter("esp", &["esp"])
                    .set_directory(&last_directory)
                    .save_file();

                if let Some(path) = some_path {
                    save_all(records, edited_records, &path, ui_args.toasts);
                    *last_directory = path;
                }
            }

            // Save as patch button
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save As Patch").clicked() {
                let some_path = rfd::FileDialog::new()
                    .add_filter("esp", &["esp"])
                    .set_directory(&last_directory)
                    .save_file();

                if let Some(path) = some_path {
                    save_patch(records, edited_records, &path, ui_args.toasts);
                    *last_directory = path;
                }
            }

            ui.separator();

            // Quit button
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Quit").clicked() {
                ui_args.frame.close();
            }
        });

        // Open button
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Open File").clicked() {
            let file_option = rfd::FileDialog::new()
                .add_filter("esp", &["esp"])
                .set_directory(&last_directory)
                .pick_file();

            if let Some(path) = file_option {
                open_plugin(
                    path,
                    plugin_path,
                    last_directory,
                    edited_records,
                    records,
                    recent_plugins,
                );
            }
        }
        #[cfg(target_arch = "wasm32")]
        if ui.button("Open File").clicked() {
            let future = async {
                let file_option = rfd::AsyncFileDialog::new()
                    .add_filter("esp", &["esp"])
                    .set_directory(&last_directory)
                    .pick_file()
                    .await;

                if let Some(path) = file_option {
                    let data = path.read().await;

                    let mut plugin = Plugin::new();
                    if let Ok(p) = &plugin.load_bytes(&data) {
                        //*plugin_path = path.clone();
                        //*last_directory = path;

                        // clear old data
                        edited_records.clear();
                        records.clear();
                        for record in plugin.objects {
                            records.insert(get_unique_id(&record), record);
                        }
                    }
                }
                //let data = file.unwrap().read().await;
            };
        }

        ui.separator();

        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save All").clicked() {
            save_all(records, edited_records, plugin_path, ui_args.toasts);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save Patch").clicked() {
            save_patch(records, edited_records, plugin_path, ui_args.toasts);
        }

        // theme button on right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
            global_dark_light_mode_switch(ui, ui_args.light_mode);
            ui.label("Theme: ");
            egui::warn_if_debug_build(ui);
        });
    });
}

fn open_plugin(
    path: PathBuf,
    plugin_path: &mut PathBuf,
    last_directory: &mut PathBuf,
    edited_records: &mut HashMap<String, TES3Object>,
    records: &mut HashMap<String, TES3Object>,
    recent_plugins: &mut Vec<PathBuf>,
) {
    if let Ok(p) = Plugin::from_path(&path) {
        *plugin_path = path.clone();
        *last_directory = path;
        let path = plugin_path.to_path_buf();
        if !recent_plugins.contains(&path) {
            recent_plugins.push(path);
        }
        recent_plugins.dedup();
        if recent_plugins.len() > 10 {
            recent_plugins.remove(0);
        }

        // clear old data
        edited_records.clear();
        records.clear();
        for record in p.objects {
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
