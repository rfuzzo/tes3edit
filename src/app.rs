use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

use egui_notify::Toasts;

use tes3::esp::Plugin;

use crate::get_unique_id;
use crate::ERecordType;
use crate::EScale;
use crate::ETheme;
use crate::PluginMetadata;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // serialized data
    /// Recently opened plugins
    pub recent_plugins: Vec<PathBuf>,

    /// The last directory used in the file picker
    pub last_directory: PathBuf,

    /// The last directory used in the file picker
    pub theme: ETheme,
    pub scale: EScale,

    pub overwrite: bool,

    // runtime
    #[serde(skip)]
    pub current_plugin_id: String,

    #[serde(skip)]
    pub plugins: Vec<PluginMetadata>,

    #[serde(skip)]
    pub search_text: String,

    #[serde(skip)]
    pub record_type: ERecordType,

    // ui
    #[serde(skip)]
    pub toasts: Toasts,

    // https://github.com/ergrelet/resym/blob/e4d243eb9459211ade0c5bae16096712a0615b0b/resym/src/resym_app.rs
    /// Field used by wasm32 targets to store file information
    /// temporarily when selecting a file to open.
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub open_file_data: Rc<RefCell<Option<(String, Plugin)>>>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub save_file_data: Rc<RefCell<Option<String>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            recent_plugins: vec![],
            last_directory: "/".into(),
            theme: ETheme::Frappe,
            scale: EScale::Small,
            overwrite: false,
            current_plugin_id: "".into(),
            plugins: vec![],
            search_text: "".into(),
            record_type: ERecordType::MISC,
            toasts: Toasts::default(),
            #[cfg(target_arch = "wasm32")]
            open_file_data: Rc::new(RefCell::new(None)),
            #[cfg(target_arch = "wasm32")]
            save_file_data: Rc::new(RefCell::new(None)),
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
    fn process_open_file_result(&mut self) {
        if let Some((name, plugin)) = self.open_file_data.borrow_mut().take() {
            let plugin_id = name;
            self.current_plugin_id = plugin_id.clone();

            if let Some(plugin_data) = self
                .plugins
                .iter_mut()
                .find(|p| p.id == self.current_plugin_id)
            {
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
                let mut data = PluginMetadata::new(plugin_id, None);
                // add new data
                for record in plugin.objects {
                    data.records.insert(get_unique_id(&record), record);
                }
                self.plugins.push(data);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn process_save_file_result(&mut self) {
        use std::path::Path;

        if let Some(file_name) = self.save_file_data.borrow_mut().take() {
            // todo save to file in wasm
            if let Some(_plugin_data) = self.plugins.iter().find(|p| p.id == self.current_plugin_id)
            {
                let path = Path::new(file_name.as_str());
                // crate::save_all(
                //     &plugin_data.records,
                //     &plugin_data.edited_records,
                //     path,
                //     &mut self.toasts,
                //     &self.overwrite,
                // );
                self.toasts.info(path.display().to_string());
                //self.last_directory = path.to_path_buf();
            }
        }
    }

    fn update_top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.menu_bar_view(ui, frame);

            self.tab_bar(ui, frame);
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
    /// Panics if paths are messed up
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

            let plugin_id = path.to_str().unwrap().to_string();
            self.current_plugin_id = plugin_id.clone();

            // if the plugin already is opened, replace
            if let Some(plugin_data) = self
                .plugins
                .iter_mut()
                .find(|p| p.id == self.current_plugin_id)
            {
                // clear old data
                plugin_data.cached_ids.clear();
                plugin_data.edited_records.clear();
                plugin_data.records.clear();

                // add new data
                for record in plugin.objects {
                    plugin_data.records.insert(get_unique_id(&record), record);
                }
            } else {
                // insert new
                let mut data = PluginMetadata::new(plugin_id, Some(path));
                // add new data
                for record in plugin.objects {
                    data.records.insert(get_unique_id(&record), record);
                }
                self.plugins.push(data);
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
}

fn get_theme(theme: &crate::app::ETheme) -> catppuccin_egui::Theme {
    match theme {
        crate::app::ETheme::Frappe => catppuccin_egui::FRAPPE,
        crate::app::ETheme::Latte => catppuccin_egui::LATTE,
        crate::app::ETheme::Macchiato => catppuccin_egui::MACCHIATO,
        crate::app::ETheme::Mocha => catppuccin_egui::MOCHA,
    }
}
