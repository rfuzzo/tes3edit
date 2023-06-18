use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

use egui::emath;
use egui::Color32;
use egui::Pos2;
use egui::Rect;
use egui::Shape;
use egui_notify::Toasts;

use tes3::esp::Plugin;

use crate::get_unique_id;
use crate::CompareData;
use crate::EAppState;
use crate::EModalState;
use crate::ERecordType;
use crate::EScale;
use crate::ETheme;
use crate::MapData;
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

    #[serde(skip)]
    pub compare_data: CompareData,
    #[serde(skip)]
    pub map_data: MapData,

    // ui
    #[serde(skip)]
    pub toasts: Toasts,
    #[serde(skip)]
    pub app_state: EAppState,
    #[serde(skip)]
    pub modal_open: bool,
    #[serde(skip)]
    pub modal_state: EModalState,

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
            // runtime data
            compare_data: CompareData::default(),
            map_data: MapData::default(),
            // TODO refactor
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
            app_state: EAppState::default(),
            modal_state: EModalState::default(),
            modal_open: false,
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
    pub fn process_save_file_result(&mut self) {
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

    #[cfg(target_arch = "wasm32")]
    pub fn process_open_file_result(&mut self) {
        if let Some((name, plugin)) = self.open_file_data.borrow_mut().take() {
            let plugin_id = name;
            self.current_plugin_id = plugin_id.clone();

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
                let mut data = PluginMetadata::new(plugin_id, None);
                // add new data
                for record in plugin.objects {
                    data.records.insert(get_unique_id(&record), record);
                }
                self.plugins.push(data);
            }
        }
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
                plugin_data.clear_cache();
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

    /// Opens a plugin
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_file_native(&mut self) {
        let file_option = rfd::FileDialog::new()
            .add_filter("esp", &["esp"])
            .add_filter("esm", &["esm"])
            .add_filter("omwaddon", &["omwaddon"])
            .set_directory(&self.last_directory)
            .pick_file();

        if let Some(path) = file_option {
            if let Ok(plugin) = Plugin::from_path(&path) {
                Self::open_plugin(self, Some(path), plugin);
            }
        }
    }

    /// Opens a modal window of specified state
    #[cfg(debug_assertions)]
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn open_modal_window(&mut self, ui: &mut egui::Ui, modal: EModalState) {
        // cleanup
        self.compare_data = CompareData::default();
        self.map_data = MapData::default();

        // disable ui
        ui.set_enabled(false);
        self.modal_open = true;
        self.modal_state = modal;
    }

    /// Opens a modal window of specified state
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn close_modal_window(&mut self, ui: &mut egui::Ui) {
        // enable ui
        ui.set_enabled(true);
        self.modal_open = false;
        self.modal_state = EModalState::None;
    }

    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn paint(&mut self, painter: &egui::Painter) {
        let bounds = std::cmp::max(self.map_data.min.abs(), self.map_data.max.abs());
        let boundsf = bounds as f32;

        let rect = painter.clip_rect();
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_max(Pos2::new(-boundsf, -boundsf), Pos2::new(boundsf, boundsf)),
            rect,
        );
        let from_screen = emath::RectTransform::from_to(
            rect,
            Rect::from_min_max(Pos2::new(-boundsf, -boundsf), Pos2::new(boundsf, boundsf)),
        );

        //draw rows
        let mut shapes: Vec<Shape> = Vec::new();
        for x in -bounds..bounds {
            for y in -bounds..bounds {
                let key = (x, -y); // draw upside down

                let mut color = Color32::DARK_BLUE;
                if self.map_data.cells.contains_key(&key) {
                    let cell = self.map_data.cells.get(&key).unwrap();

                    if let Some(map_color) = cell.map_color {
                        color =
                            Color32::from_rgb_additive(map_color[0], map_color[1], map_color[2]);
                    } else {
                        color = Color32::DARK_GREEN;
                    }
                }
                if let Some(grid) = self.map_data.cell_ids.get(&self.map_data.selected_id) {
                    if grid == &key {
                        color = Color32::RED;
                    }
                }

                let cell_pos = to_screen * Pos2::new(x as f32, y as f32);
                let cell_pos2 = to_screen * Pos2::new((x as f32) + 1.0, (y as f32) + 1.0);
                let cell_rect = egui::epaint::Rect::from_two_pos(cell_pos, cell_pos2);
                let rect_shape = Shape::rect_filled(cell_rect, egui::Rounding::none(), color);
                shapes.push(rect_shape);
            }
        }
        painter.extend(shapes);

        if let Some(hover_pos) = painter.ctx().pointer_hover_pos() {
            let real_pos = from_screen * hover_pos;

            let mut x = real_pos.x;
            let mut y = -real_pos.y;
            // hacks to get the correct cell name
            if x < 0.0 {
                x -= 1.0;
            }
            if y > 0.0 {
                y += 1.0;
            }
            self.map_data.hover_pos = (x as i32, y as i32);
        }
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
            #[cfg(not(target_arch = "wasm32"))]
            match self.modal_state {
                EModalState::ModalCompareInit => self.update_modal_compare(ctx),
                EModalState::MapInit => self.update_modal_map(ctx),
                EModalState::None => panic!("ArgumentException"),
            }
        } else {
            // other main ui views
            match self.app_state {
                EAppState::Main => self.update_edit_view(ctx, frame),
                EAppState::Compare => self.update_compare_view(ctx, frame),
                EAppState::Map => self.update_map_view(ctx, frame),
            }
        }

        // notifications
        self.toasts.show(ctx);
    }
}
