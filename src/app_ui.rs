use tes3::esp::Plugin;

use crate::{get_cell_name, get_theme, TemplateApp};

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

    /// Main single plugin edit view
    pub fn update_edit_view(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // drag and drop
        self.ui_file_drag_and_drop(ctx);

        // scale
        ctx.set_pixels_per_point(f32::from(self.scale));
        // themes
        catppuccin_egui::set_theme(ctx, get_theme(&self.theme));

        // wasm open and save file
        #[cfg(target_arch = "wasm32")]
        self.process_open_file_result();

        #[cfg(target_arch = "wasm32")]
        self.process_save_file_result();

        // Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.menu_bar_view(ui, frame);

            self.tab_bar(ui, frame);
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

    /// Main map view
    pub fn update_map_view(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Panel
        let mut skip = false;
        egui::TopBottomPanel::top("top_panel_map").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if self.map_data.refresh_requested {
                    // exit
                    self.open_modal_window(ui, crate::EModalState::MapInit);
                    skip = true;
                }

                if ui.button("Exit").clicked() {
                    self.app_state = crate::EAppState::Main;
                    skip = true;
                }
            });
        });

        if skip {
            return;
        }

        // Side Panel
        egui::SidePanel::left("side_panel_map")
            .min_width(250_f32)
            .show(ctx, |ui| {
                // heading
                ui.heading("Cells");
                ui.separator();

                // search bar
                ui.horizontal(|ui| {
                    ui.label("Filter: ");
                    ui.text_edit_singleline(&mut self.edit_data.search_text);
                });
                ui.separator();

                // list of cells
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut ids: Vec<&String> = self.map_data.cell_ids.keys().collect::<Vec<_>>();
                    ids.sort();

                    for key in ids.iter() {
                        // TODO upper and lowercase search
                        if !self.edit_data.search_text.is_empty()
                            && !key
                                .to_lowercase()
                                .contains(&self.edit_data.search_text.to_lowercase())
                        {
                            continue;
                        }

                        if self.map_data.selected_id == key.to_string() {
                            ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                        } else {
                            ui.visuals_mut().override_text_color = None;
                        }

                        let response =
                            ui.add(egui::Label::new(&(*key).clone()).sense(egui::Sense::click()));
                        if response.clicked() {
                            self.map_data.selected_id = key.to_string();
                        }
                    }
                });
            });

        // footer
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // Status Bar
            ui.horizontal(|ui| {
                // map bounds
                ui.label(format!(
                    "({},{}) - ({},{})",
                    self.map_data.bounds_x.0,
                    self.map_data.bounds_y.0,
                    self.map_data.bounds_x.1,
                    self.map_data.bounds_y.1
                ));
                ui.separator();
                ui.label(get_cell_name(&self.map_data, self.map_data.hover_pos));
            });
        });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            self.map_view(ui);
        });
    }
}
