#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, path::PathBuf};

use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use tes3::esp::TES3Object;

use crate::views::record_editor_view::record_editor_view;
use crate::views::records_list_view::records_list_view;

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

    // runtime
    #[serde(skip)]
    pub records: HashMap<String, TES3Object>,

    #[serde(skip)]
    pub edited_records: HashMap<String, TES3Object>,

    #[serde(skip)]
    pub current_record_id: Option<String>,

    // https://github.com/ergrelet/resym/blob/e4d243eb9459211ade0c5bae16096712a0615b0b/resym/src/resym_app.rs
    /// Field used by wasm32 targets to store file information
    /// temporarily when selecting a file to open.
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub open_pdb_data: Rc<RefCell<Option<(String, Vec<u8>)>>>,

    // ui
    #[serde(skip)]
    pub toasts: Toasts,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            recent_plugins: vec![],
            last_directory: "/".into(),
            records: HashMap::default(),
            edited_records: HashMap::default(),
            toasts: Toasts::default(),
            light_mode: false,
            current_record_id: None,
            #[cfg(target_arch = "wasm32")]
            open_pdb_data: Rc::new(RefCell::new(None)),
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
    fn process_open_pdb_file_result(&self) {
        if let Some((pdb_name, pdb_bytes)) = self.open_pdb_data.borrow_mut().take() {
            // todo
        }
    }

    fn update_top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Process keyboard shortcuts, if any
            // self.consume_keyboard_shortcuts(ui);

            // The top panel is often a good place for a menu bar
            // menu_bar_view(
            //     UiArgs::new(frame, ui, &mut self.toasts, &mut self.light_mode),
            //     &mut self.records,
            //     &mut self.edited_records,
            //     &mut self.recent_plugins,
            //     &mut self.last_directory,
            // );
            self.menu_bar_view(ui, frame);
        });
    }

    fn update_left_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                ui.heading("Records");

                records_list_view(
                    ui,
                    &mut self.records,
                    &mut self.edited_records,
                    &mut self.current_record_id,
                );
            });
    }

    fn update_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(record_id) = self.current_record_id.clone() {
                record_editor_view(
                    ui,
                    &record_id,
                    &mut self.edited_records,
                    &mut self.records,
                    &mut self.toasts,
                );
            }
        });
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
        // if light mode is requested but the app is in dark mode, we enable light mode
        if self.light_mode && ctx.style().visuals.dark_mode {
            ctx.set_visuals(egui::Visuals::light());
        }

        #[cfg(target_arch = "wasm32")]
        self.process_open_pdb_file_result();

        // Top Panel
        self.update_top_panel(ctx, frame);

        // Side Panel
        self.update_left_side_panel(ctx);

        // Central Panel
        self.update_central_panel(ctx);

        // notifications
        self.toasts.show(ctx);
    }
}
