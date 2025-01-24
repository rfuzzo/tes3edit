use tes3::esp::Plugin;

use crate::{get_all_tags, EAppState, EModalState, TemplateApp};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl TemplateApp {
    // https://github.com/EmbarkStudios/puffin/blob/dafc2ff1755e5ed85c405f7240603f1af6c71c24/puffin_viewer/src/lib.rs#L239
    pub fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input(|i| i.screen_rect());
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                "Drop to open plugin",
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in i.raw.dropped_files.iter() {
                    if let Some(path) = &file.path {
                        if let Ok(plugin) = Plugin::from_path(path) {
                            Self::open_plugin(self, Some(path.to_path_buf()), plugin);
                        }
                        break;
                    }
                }
            }
        });
    }

    pub fn update_records_view(&mut self, ctx: &egui::Context) {
        // Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File Menu
                ui.menu_button("File", |ui| {
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
                            self.toasts.warning(
                                "Please close all open plugins before entering compare mode",
                            );
                        } else {
                            self.open_modal_window(ui, EModalState::ModalCompareInit);
                        }
                        ui.close_menu();
                    }

                    if ui.button("Records View").clicked() {
                        if !self.edit_data.plugins.is_empty() {
                            self.toasts.warning(
                                "Please close all open plugins before entering compare mode",
                            );
                        } else {
                            self.app_state = EAppState::Records;
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Settings").clicked() {
                        self.open_modal_window(ui, EModalState::Settings);
                        ui.close_menu();
                    }
                });
            });
        });

        // load plugins
        if self.records_data.records.is_empty() {
            self.load_records();
        }

        // Side Panel
        let tags = get_all_tags();

        egui::CentralPanel::default().show(ctx, |ui| {
            // search bar
            let search_text = self.records_data.search_text.clone();
            ui.horizontal(|ui| {
                ui.label("Filter: ");
                ui.text_edit_singleline(&mut self.records_data.search_text);
            });
            ui.separator();

            // regenerate records
            if (search_text != self.records_data.search_text) || self.records_data.cache.is_empty()
            {
                self.records_data.cache.clear();

                let filter = self.records_data.search_text.to_lowercase();

                for (tag, records) in self.records_data.records.iter() {
                    // get all records where the key contains the search text
                    let mut filtered_records = records
                        .keys()
                        .filter(|k| k.to_lowercase().contains(&filter))
                        .cloned()
                        .collect::<Vec<_>>();

                    filtered_records.sort();

                    self.records_data
                        .cache
                        .insert(tag.clone(), filtered_records);
                }
            }

            // show all records
            egui::ScrollArea::vertical().show(ui, |ui| {
                for tag in tags {
                    let ids_by_tag = self.records_data.cache.get(&tag).unwrap();

                    if ids_by_tag.is_empty() {
                        continue;
                    }

                    // add headers and subitems
                    let _tag_header = egui::CollapsingHeader::new(tag.clone()).show(ui, |ui| {
                        for id in ids_by_tag.iter() {
                            // lookup
                            let plugins = self
                                .records_data
                                .records
                                .get(&tag)
                                .unwrap()
                                .get(id)
                                .unwrap();

                            ui.horizontal(|ui| {
                                ui.label(id.clone());
                                ui.separator();
                                ui.label(format!("{:?}", plugins));
                            });
                        }
                    });
                }
            });
        });

        // Central Panel
        //egui::CentralPanel::default().show(ctx, |ui| {});
    }

    /// Main single plugin edit view
    pub fn update_edit_view(&mut self, ctx: &egui::Context) {
        // drag and drop
        self.ui_file_drag_and_drop(ctx);

        // scale
        ctx.set_pixels_per_point(f32::from(self.scale));
        // themes
        //catppuccin_egui::set_theme(ctx, get_theme(&self.theme));

        // wasm open and save file
        // #[cfg(target_arch = "wasm32")]
        // self.process_open_file_result();

        // #[cfg(target_arch = "wasm32")]
        // self.process_save_file_result();

        // Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.menu_bar_view(ui, ctx);

            self.tab_bar(ui);
        });

        // bottom Panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // Status Bar
            ui.horizontal(|ui| {
                // Number of edited records
                let mut status_edited = "Edited Records: ".to_owned();
                if let Some(data) = self
                    .edit_data
                    .plugins
                    .iter()
                    .find(|p| p.id == self.edit_data.current_plugin_id)
                {
                    status_edited = format!("Edited Records: {}", data.edited_records.len());
                }
                ui.label(status_edited);

                // VERSION
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(VERSION);
                    ui.label("Version: ");
                    ui.separator();
                    ui.hyperlink("https://github.com/rfuzzo/tes3edit");
                });
            });
        });

        // Side Panel
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                self.records_list_view(ui);
            });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            self.record_editor_view(ui);
        });
    }

    /// Main compare view
    pub fn update_compare_view(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Top Panel
        egui::TopBottomPanel::top("top_panel_compare").show(ctx, |ui| {
            self.conflict_menu_bar_view(ui, frame);
        });

        // Side Panel
        egui::SidePanel::left("side_panel_compare")
            .min_width(250_f32)
            .show(ctx, |ui| {
                self.conflict_list_view(ui);
            });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            self.conflict_compare_view(ui);
        });
    }
}
