use tes3::esp::Plugin;

use crate::{get_theme, TemplateApp};

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
                if let Some(data) = self.plugins.iter().find(|p| p.id == self.current_plugin_id) {
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
    pub(crate) fn update_compare_view(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
