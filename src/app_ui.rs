#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

use tes3::esp::Plugin;

use crate::{get_theme, CompareData, EAppState, TemplateApp};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl TemplateApp {
    pub fn update_top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.menu_bar_view(ui, frame);

            self.tab_bar(ui, frame);
        });
    }

    pub fn update_left_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                self.records_list_view(ui);
            });
    }

    pub fn update_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.record_editor_view(ui);
        });
    }

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
        self.update_top_panel(ctx, frame);

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
        self.update_left_side_panel(ctx);

        // Central Panel
        self.update_central_panel(ctx);

        // notifications
        self.toasts.show(ctx);
    }

    /// Main compare view
    pub(crate) fn update_compare_view(&self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        todo!()
    }

    /////////////////////////////////////////////////
    // Modal views

    /// Returns the update modal compare of this [`TemplateApp`].
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn update_modal_compare(&mut self, ctx: &egui::Context) {
        egui::Window::new("Compare Plugins")
            .open(&mut self.modal_open)
            .show(ctx, |ui| {
                // Main view
                // a folder to chose
                if let Some(in_path) = self.compare_data.path.clone() {
                    ui.horizontal(|ui| {
                        ui.label(in_path.display().to_string());
                        if ui.button("...").clicked() {
                            open_compare_folder(&mut self.compare_data);
                        }
                    });
                    ui.separator();
                    // plugin select view
                    for vm in self.compare_data.plugins.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut vm.enabled, "");
                            ui.label(vm.path.file_name().unwrap().to_string_lossy());
                        });
                    }
                    ui.separator();
                    if ui.button("OK").clicked() {
                        // go into compare mode
                        self.app_state = EAppState::Compare;
                        // TODO leave?
                    }
                } else {
                    open_compare_folder(&mut self.compare_data);
                }
            });
    }
}

fn open_compare_folder(data: &mut CompareData) {
    let folder_option = rfd::FileDialog::new().pick_folder();
    if let Some(path) = folder_option {
        if !path.is_dir() {
            return;
        }

        data.path = Some(path.clone());
        // get plugins
        data.plugins = crate::get_plugins_in_folder(&path, true)
            .iter()
            .map(|e| crate::CompareItemViewModel {
                path: e.to_path_buf(),
                enabled: false,
            })
            .collect::<Vec<_>>();
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // general storage save
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // modal windows
        if self.modal_open {
            match self.modal_state {
                crate::EModalState::ModalCompareInit => self.update_modal_compare(ctx),
                _ => panic!("ArgumentException"),
            }
        } else {
            // other main ui views
            match self.app_state {
                EAppState::Main => self.update_edit_view(ctx, frame),
                EAppState::Compare => self.update_compare_view(ctx, frame),
            }
        }
    }
}
