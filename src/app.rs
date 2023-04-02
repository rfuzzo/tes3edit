use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{collections::HashMap, path::PathBuf};
use tes3::esp::{EditorId, Plugin, TES3Object, TypeInfo};

use crate::{get_unique_id, save_all, save_patch};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    plugin_path: PathBuf,

    #[serde(skip)]
    records: HashMap<String, TES3Object>,

    #[serde(skip)]
    edited_records: HashMap<String, TES3Object>,

    #[serde(skip)]
    current_text: (String, String),

    #[serde(skip)]
    toasts: Toasts,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            plugin_path,
            records,
            edited_records,
            current_text,
            toasts,
        } = self;

        // Top Panel
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
        });

        // Side Bar
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                ui.heading("Side Panel");

                // group by tag
                let mut tags: Vec<&str> = records.values().map(|e| e.tag_str()).collect();
                tags.sort();
                tags.dedup();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for tag in tags {
                        let records: Vec<_> =
                            records.values().filter(|r| r.tag_str() == tag).collect();
                        // add headers and tree
                        egui::CollapsingHeader::new(tag).show(ui, |ui| {
                            // add records
                            for record in records {
                                let id = get_unique_id(record);
                                // if modified, annotate it
                                let is_modified = edited_records.contains_key(&id);
                                let mut label = record.editor_id().to_string();
                                if is_modified {
                                    label = format!("{}*", label);
                                    ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                                } else {
                                    ui.visuals_mut().override_text_color = None;
                                }
                                if ui
                                    .add(egui::Label::new(label).sense(egui::Sense::click()))
                                    .clicked()
                                {
                                    // on clicked event for records
                                    // deserialize the original record or the edited

                                    if edited_records.contains_key(&id) {
                                        *current_text = (
                                            id.clone(),
                                            serde_yaml::to_string(&edited_records[&id])
                                                .unwrap_or("Error serializing".to_owned()),
                                        );
                                    } else {
                                        *current_text = (
                                            id,
                                            serde_yaml::to_string(&record)
                                                .unwrap_or("Error serializing".to_owned()),
                                        );
                                    }
                                }
                            }
                        });
                    }
                    ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
                });
            });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // Revert record button
                #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
                if ui.button("Revert").clicked() {
                    let id = current_text.0.clone();
                    // get original record
                    if edited_records.contains_key(&id) {
                        // remove from edited records
                        edited_records.remove(&id);
                        // revert text
                        *current_text = (
                            id.clone(),
                            serde_yaml::to_string(&records[&id])
                                .unwrap_or("Error serializing".to_owned()),
                        );
                        toasts
                            .info("Record reverted")
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                }

                // Save record button
                #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
                if ui.button("Save").clicked() {
                    // deserialize
                    let deserialized: Result<TES3Object, _> = serde_yaml::from_str(&current_text.1);
                    if let Ok(record) = deserialized {
                        // add or update current record to list
                        edited_records.insert(current_text.0.clone(), record);
                        toasts
                            .success("Record saved")
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                }
            });

            // text editor
            egui::ScrollArea::vertical().show(ui, |ui| {
                let _response = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut current_text.1),
                );
            });
        });

        toasts.show(ctx);
    }
}
