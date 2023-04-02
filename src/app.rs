use std::{collections::HashMap, path::PathBuf};

use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use tes3::esp::TES3Object;

use crate::views::menu_bar_view::menu_bar_view;
use crate::views::record_editor_view::record_text_editor_view;
use crate::views::records_list_view::records_list_view;

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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            plugin_path,
            records,
            edited_records,
            current_text,
            toasts,
        } = self;

        // Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu_bar_view(ui, records, edited_records, toasts, frame, plugin_path);
        });

        // Side Panel
        egui::SidePanel::left("side_panel")
            .min_width(250_f32)
            .show(ctx, |ui| {
                ui.heading("Records");

                records_list_view(ui, records, edited_records, current_text);
            });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            record_text_editor_view(ui, current_text, edited_records, records, toasts);
        });

        // notifications
        toasts.show(ctx);
    }
}
