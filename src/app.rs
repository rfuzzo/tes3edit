#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, path::PathBuf};

use crate::{get_all_tags, RecordsData};
use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use tes3::esp::{EditorId, Plugin, TypeInfo};

use crate::{get_unique_id, CompareData, EAppState, EModalState, EScale, EditData, PluginMetadata};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // serialized data
    /// Recently opened plugins
    pub recent_plugins: Vec<PathBuf>,
    /// The last directory used in the file picker
    pub last_directory: PathBuf,

    // settings
    pub overwrite: bool,
    pub use_experimental: bool,
    pub scale: EScale,

    // runtime
    #[serde(skip)]
    pub edit_data: EditData,
    #[serde(skip)]
    pub compare_data: CompareData,
    #[serde(skip)]
    pub records_data: RecordsData,

    // runtime ui
    #[serde(skip)]
    pub toasts: Toasts,
    #[serde(skip)]
    pub app_state: EAppState,
    #[serde(skip)]
    pub modal_open: bool,
    #[serde(skip)]
    pub modal_state: EModalState,

    // wasm
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
            // runtime data
            compare_data: CompareData::default(),
            edit_data: EditData::default(),
            records_data: RecordsData::default(),
            // settings
            overwrite: false,
            use_experimental: false,
            // ui data
            scale: EScale::Small,

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

    /// Open a new plugin
    ///
    /// # Panics
    ///
    /// Panics if paths are messed up
    pub fn open_plugin(&mut self, path_option: Option<PathBuf>, plugin: Plugin) {
        // save paths if on native
        if let Some(path) = path_option {
            self.last_directory.clone_from(&path);

            if !self.recent_plugins.contains(&self.last_directory) {
                self.recent_plugins.push(self.last_directory.to_path_buf());
            }
            self.recent_plugins.dedup();
            if self.recent_plugins.len() > 10 {
                self.recent_plugins.remove(0);
            }

            let plugin_id = path.to_str().unwrap().to_string();
            self.edit_data.current_plugin_id.clone_from(&plugin_id);

            // if the plugin already is opened, replace
            if let Some(plugin_data) = self
                .edit_data
                .plugins
                .iter_mut()
                .find(|p| p.id == self.edit_data.current_plugin_id)
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
                self.edit_data.plugins.push(data);
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
    pub(crate) fn open_modal_window(&mut self, _ui: &mut egui::Ui, modal: EModalState) {
        // cleanup
        self.compare_data = CompareData::default();

        // disable ui
        self.modal_open = true;
        self.modal_state = modal;
    }

    /// Opens a modal window of specified state
    pub(crate) fn close_modal_window(&mut self, _ui: &mut egui::Ui) {
        // enable ui
        self.modal_open = false;
        self.modal_state = EModalState::None;
    }

    pub(crate) fn load_records(&mut self) {
        if !self.compare_data.path.exists() {
            if let Ok(cwd) = std::env::current_dir() {
                self.compare_data.path = cwd;
            } else {
                self.compare_data.path = PathBuf::from("");
            }
        }

        let plugin_paths = crate::get_plugins_sorted(&self.compare_data.path, false);
        let mut plugins = Vec::new();
        for path in plugin_paths.iter() {
            if let Ok(plugin) = crate::parse_plugin(path) {
                let filename = path.file_name().unwrap().to_str().unwrap();

                plugins.push((filename, plugin));
            }
        }

        let mut map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
        for tag in get_all_tags().iter() {
            map.insert(tag.to_string(), HashMap::new());
        }

        for (plugin_name, plugin) in plugins.iter() {
            for record in plugin.objects.iter() {
                let id: String = record.editor_id().to_string();
                let tag = record.tag_str().to_string();
                if let Some(records) = map.get_mut(&tag) {
                    if let Some(plugins) = records.get_mut(&id) {
                        plugins.push(plugin_name.to_string());
                    } else {
                        records.insert(id, vec![plugin_name.to_string()]);
                    }
                }
            }
        }

        self.records_data.records = map;
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
                EModalState::None => panic!("ArgumentException"),
                EModalState::ModalCompareInit => self.update_modal_compare(ctx),
                EModalState::Settings => self.update_settings(ctx),
            }
        } else {
            // other main ui views
            match self.app_state {
                EAppState::SingleEdit => self.update_edit_view(ctx),
                EAppState::Compare => self.update_compare_view(ctx, frame),
                EAppState::Records => self.update_records_view(ctx),
            }
        }

        // notifications
        self.toasts.show(ctx);
    }
}
