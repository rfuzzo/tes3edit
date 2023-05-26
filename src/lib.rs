#![warn(clippy::all, rust_2018_idioms)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

pub use app::TemplateApp;
use egui_notify::Toasts;

mod app;
mod views;
use serde::{Deserialize, Serialize};
use tes3::esp::{EditorId, Plugin, TES3Object, TypeInfo};

/// Catpuccino themes
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub enum ETheme {
    Frappe,
    Latte,
    Macchiato,
    Mocha,
}

/// App scale
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EScale {
    Native,
    Small,
    Medium,
    Large,
}
impl From<EScale> for f32 {
    fn from(val: EScale) -> Self {
        match val {
            EScale::Native => 1.2,
            EScale::Small => 2.2,
            EScale::Medium => 3.0,
            EScale::Large => 4.5,
        }
    }
}

//////////////////////////////////////////
// Common

// https://internals.rust-lang.org/t/pathbuf-has-set-extension-but-no-add-extension-cannot-cleanly-turn-tar-to-tar-gz/14187/11
pub fn append_ext(ext: impl AsRef<std::ffi::OsStr>, path: PathBuf) -> PathBuf {
    let mut os_string: std::ffi::OsString = path.into();
    os_string.push(".");
    os_string.push(ext.as_ref());
    os_string.into()
}

//////////////////////////////////////////
// TES3

/// creates a unique id from a record
/// we take the record tag + the record id
pub fn get_unique_id(record: &TES3Object) -> String {
    format!("{},{}", record.tag_str(), record.editor_id())
}

/// Creates an id for a plugin
///
/// # Panics
///
/// Panics if no full path or path is messed up
pub fn get_plugin_id(plugin: &PluginMetadata) -> String {
    plugin
        .full_path
        .as_ref()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

//////////////////////////////////////////
// App

/// Plugin Viewmodel in-app
pub struct PluginMetadata {
    pub id: String,
    pub full_path: Option<PathBuf>,
    pub records: HashMap<String, TES3Object>,
    pub sorted_records: HashMap<String, Vec<String>>,
    pub edited_records: HashMap<String, TES3Object>,
    pub selected_record_id: Option<String>,
}

impl PluginMetadata {
    pub fn new(id: String, full_path: Option<PathBuf>) -> Self {
        Self {
            id,
            full_path,
            records: HashMap::default(),
            sorted_records: HashMap::default(),
            edited_records: HashMap::default(),
            selected_record_id: None,
        }
    }
}

/// Get assembled records in-app
///
/// # Panics
///
/// Panics if no header found
pub fn get_records(
    records: &HashMap<String, TES3Object>,
    edited_records: &HashMap<String, TES3Object>,
) -> Vec<TES3Object> {
    // construct records from both lists
    let mut final_records = records.clone();
    for r in edited_records.iter() {
        final_records.insert(r.0.to_string(), r.1.clone());
    }

    // sort records
    // todo sort all records, header first
    let mut records_vec: Vec<_> = final_records.values().cloned().collect();
    let pos = records_vec
        .iter()
        .position(|e| e.tag_str() == "TES3")
        .unwrap();
    let header = records_vec.remove(pos);
    records_vec.insert(0, header);
    records_vec
}

/// Saves records as plugin to the specified path
/// If overwrite is not specified, appends new.esp as extension
pub fn save_plugin<P>(
    records: &HashMap<String, TES3Object>,
    edited_records: &HashMap<String, TES3Object>,
    plugin_path: P,
    toasts: &mut Toasts,
    overwrite: bool,
) where
    P: AsRef<Path>,
{
    let mut plugin = Plugin {
        objects: get_records(records, edited_records),
    };
    // save
    let mut output_path = plugin_path.as_ref().to_path_buf();
    if !overwrite {
        output_path = plugin_path.as_ref().with_extension("new.esp");
    }

    match plugin.save_path(output_path) {
        Ok(_) => {
            toasts.success("Plugin saved");
        }
        Err(_) => {
            toasts.error("Could not save plugin");
        }
    }
}

/// Saves a plugin as patch, appends patch.esp as extension
///
/// # Panics
///
/// Panics if plugin has no header
pub fn save_patch<P>(
    records: &HashMap<String, TES3Object>,
    edited_records: &HashMap<String, TES3Object>,
    plugin_path: P,
    toasts: &mut Toasts,
) where
    P: AsRef<Path>,
{
    let mut records_vec: Vec<_> = edited_records.values().cloned().collect();

    // if a header in changed files, then take that one instead of the original one
    // panic here since this is undefined behavior
    let mut header = records.get("TES3,").unwrap();
    if let Some(h) = edited_records.get("TES3,") {
        header = h;
    }
    records_vec.insert(0, header.clone());

    // save
    let mut plugin = Plugin {
        objects: records_vec,
    };

    let output_path = plugin_path.as_ref().with_extension("patch.esp");
    match plugin.save_path(output_path) {
        Ok(_) => {
            toasts
                .success("Plugin saved")
                .set_duration(Some(Duration::from_secs(5)));
        }
        Err(_) => {
            toasts
                .error("Could not save plugin")
                .set_duration(Some(Duration::from_secs(5)));
        }
    }
}
