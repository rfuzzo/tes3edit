#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, path::PathBuf};

use egui_notify::Toasts;
use serde::{Deserialize, Serialize};

use tes3::esp::Plugin;
use tes3::esp::TES3Object;

use crate::get_unique_id;

#[derive(Default)]
pub struct PluginMetadata {
    pub records: HashMap<String, TES3Object>,
    pub sorted_records: HashMap<String, Vec<String>>,
    pub edited_records: HashMap<String, TES3Object>,
    pub current_record_id: Option<String>,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // serialized data
    /// Recently opened plugins
    pub recent_plugins: Vec<PathBuf>,

    /// The last directory used in the file picker
    pub last_directory: PathBuf,

    /// The last directory used in the file picker
    pub light_mode: bool,

    pub overwrite: bool,

    // runtime
    #[serde(skip)]
    pub current_plugin_id: String,

    #[serde(skip)]
    pub plugins: HashMap<String, PluginMetadata>,

    #[serde(skip)]
    pub search_text: String,

    // ui
    #[serde(skip)]
    pub toasts: Toasts,

    // https://github.com/ergrelet/resym/blob/e4d243eb9459211ade0c5bae16096712a0615b0b/resym/src/resym_app.rs
    /// Field used by wasm32 targets to store file information
    /// temporarily when selecting a file to open.
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub open_pdb_data: Rc<RefCell<Option<(String, Plugin)>>>,
    // #[cfg(target_arch = "wasm32")]
    // #[serde(skip)]
    // pub save_file_dialog: Option<egui_file::FileDialog>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            recent_plugins: vec![],
            last_directory: "/".into(),
            light_mode: false,
            overwrite: false,
            current_plugin_id: "".into(),
            plugins: HashMap::default(),
            search_text: "".into(),
            toasts: Toasts::default(),
            #[cfg(target_arch = "wasm32")]
            open_pdb_data: Rc::new(RefCell::new(None)),
            // #[cfg(target_arch = "wasm32")]
            // save_file_dialog: None,
        }
    }
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

    #[cfg(target_arch = "wasm32")]
    fn process_open_pdb_file_result(&mut self) {
        if let Some((name, plugin)) = self.open_pdb_data.borrow_mut().take() {
            self.current_plugin_id = name;
            if let Some(plugin_data) = self.plugins.get_mut(&self.current_plugin_id) {
                // clear old data
                plugin_data.sorted_records.clear();
                plugin_data.edited_records.clear();
                plugin_data.records.clear();

                // add new data
                for record in plugin.objects {
                    plugin_data.records.insert(get_unique_id(&record), record);
                }
            } else {
                // insert new
                let mut data = PluginMetadata::default();
                // add new data
                for record in plugin.objects {
                    data.records.insert(get_unique_id(&record), record);
                }
                self.plugins.insert(self.current_plugin_id.clone(), data);
            }
        }
    }

    fn update_top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.menu_bar_view(ui, frame);

            self.breadcrumb(ui, frame);
        });
    }

    fn update_left_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                self.records_list_view(ui);
            });
    }

    fn update_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.record_editor_view(ui);
        });
    }

    /// Open a new plugin
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn open_plugin(&mut self, path_option: Option<PathBuf>, plugin: Plugin) {
        // save paths if on native
        if let Some(path) = path_option {
            self.last_directory = path.clone();

            if !self.recent_plugins.contains(&self.last_directory) {
                self.recent_plugins.push(self.last_directory.to_path_buf());
            }
            self.recent_plugins.dedup();
            if self.recent_plugins.len() > 10 {
                self.recent_plugins.remove(0);
            }

            // if the plugin already is opened, replace
            self.current_plugin_id = path.to_str().unwrap().to_string();
            if let Some(plugin_data) = self.plugins.get_mut(&self.current_plugin_id) {
                // clear old data
                plugin_data.sorted_records.clear();
                plugin_data.edited_records.clear();
                plugin_data.records.clear();

                // add new data
                for record in plugin.objects {
                    plugin_data.records.insert(get_unique_id(&record), record);
                }
            } else {
                // insert new
                let mut data = PluginMetadata::default();
                // add new data
                for record in plugin.objects {
                    data.records.insert(get_unique_id(&record), record);
                }
                self.plugins.insert(self.current_plugin_id.clone(), data);
            }
        }
    }

    // https://github.com/EmbarkStudios/puffin/blob/dafc2ff1755e5ed85c405f7240603f1af6c71c24/puffin_viewer/src/lib.rs#L239
    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
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
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // general storage save
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // drag and drop
        self.ui_file_drag_and_drop(ctx);

        // if light mode is requested but the app is in dark mode, we enable light mode
        if self.light_mode && ctx.style().visuals.dark_mode {
            ctx.set_visuals(egui::Visuals::light());
        }

        // wasm open and save file
        #[cfg(target_arch = "wasm32")]
        self.process_open_pdb_file_result();
        // if let Some(dialog) = &mut self.save_file_dialog {
        //     if dialog.show(ctx).selected() {
        //         if let Some(path) = dialog.path() {
        //             // get current plugin
        //             if let Some(plugin_data) = self.plugins.get(&self.current_plugin_id) {
        //                 crate::save_all(
        //                     &plugin_data.records,
        //                     &plugin_data.edited_records,
        //                     &path,
        //                     &mut self.toasts,
        //                     &self.overwrite,
        //                 );
        //             }
        //         }
        //     }
        // }

        // Top Panel
        self.update_top_panel(ctx, frame);

        // bottom Panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // Status Bar
            ui.horizontal(|ui| {
                // Number of edited records
                let mut status_edited = "Edited Records: ".to_owned();
                if let Some(data) = self.plugins.get_mut(&self.current_plugin_id) {
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
}
