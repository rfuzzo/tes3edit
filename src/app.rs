use std::{collections::HashMap, path::PathBuf};

use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use tes3::esp::TES3Object;

use crate::views::menu_bar_view::{menu_bar_view, UiArgs};
use crate::views::record_editor_view::record_editor_view;
use crate::views::records_list_view::records_list_view;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // serialized data
    /// The currently loaded plugin path
    plugin_path: PathBuf,

    /// The last directory used in the file picker
    last_directory: PathBuf,

    /// The last directory used in the file picker
    light_mode: bool,

    // runtime
    #[serde(skip)]
    records: HashMap<String, TES3Object>,

    #[serde(skip)]
    edited_records: HashMap<String, TES3Object>,

    #[serde(skip)]
    current_text: (String, String),

    #[serde(skip)]
    current_record: Option<TES3Object>,

    // ui
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            plugin_path: "".into(),
            last_directory: "/".into(),
            records: HashMap::default(),
            edited_records: HashMap::default(),
            current_text: ("".into(), "".into()),
            toasts: Toasts::default(),
            light_mode: false,
            current_record: None,
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
        let Self {
            plugin_path,
            records,
            edited_records,
            current_text,
            toasts,
            last_directory,
            light_mode,
            current_record,
        } = self;

        // if light mode is requested but the app is in dark mode, we enable light mode
        if *light_mode && ctx.style().visuals.dark_mode {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu_bar_view(
                UiArgs::new(frame, ui, toasts, light_mode),
                records,
                edited_records,
                plugin_path,
                last_directory,
            );
        });

        // Side Panel
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                ui.heading("Records");

                records_list_view(ui, records, edited_records, current_text, current_record);
            });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // custom editors
            record_editor_view(ui, current_record, edited_records, records, toasts);

            // text editor
            //record_text_editor_view(ui, current_text, edited_records, records, toasts);
        });

        // notifications
        toasts.show(ctx);
    }
}
