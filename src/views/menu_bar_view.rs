use std::{collections::HashMap, path::PathBuf};

use egui_notify::Toasts;
use tes3::esp::{Plugin, TES3Object};

use crate::{get_unique_id, save_all, save_patch};

pub(crate) fn menu_bar_view(
    ui: &mut egui::Ui,
    records: &mut HashMap<String, TES3Object>,
    edited_records: &mut HashMap<String, TES3Object>,
    toasts: &mut Toasts,
    _frame: &mut eframe::Frame,
    plugin_path: &mut PathBuf,
) {
    // Menu Bar
    egui::menu::bar(ui, |ui| {
        // File Menu
        ui.menu_button("File", |ui| {
            // todo open recent

            // Save as button
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Save as").clicked() {
                let some_path = rfd::FileDialog::new()
                    .add_filter("esp", &["esp"])
                    .set_directory("/")
                    .save_file();

                if let Some(path) = some_path {
                    save_all(records, edited_records, &path, toasts);
                }
            }

            // todo save as patch

            ui.separator();

            // Quit button
            if ui.button("Quit").clicked() {
                _frame.close();
            }
        });

        // Open button // todo wasm
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Open File").clicked() {
            let file_option = rfd::FileDialog::new()
                .add_filter("esp", &["esp"])
                .set_directory("/")
                .pick_file();

            if let Some(path) = file_option {
                if let Ok(p) = Plugin::from_path(&path) {
                    *plugin_path = path;
                    records.clear();
                    for record in p.objects {
                        records.insert(get_unique_id(&record), record);
                    }
                }
            }
        }

        ui.separator();

        // Save plugin button // todo wasm
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save All").clicked() {
            save_all(records, edited_records, plugin_path, toasts);
        }

        // Save patch button // todo wasm
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save Patch").clicked() {
            save_patch(edited_records, plugin_path, toasts);
        }

        // theme button on right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
            ui.label("Theme: ");
            egui::warn_if_debug_build(ui);
        });
    });
}
