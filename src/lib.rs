#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod views;
pub use app::TemplateApp;
use egui_notify::Toasts;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};
use tes3::esp::{EditorId, FixedString, Header, ObjectFlags, Plugin, TES3Object, TypeInfo};

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

pub fn get_unique_id(record: &TES3Object) -> String {
    format!("{},{}", record.tag_str(), record.editor_id())
}

//////////////////////////////////////////
// App

pub fn get_records(
    records: &mut HashMap<String, TES3Object>,
    edited_records: &mut HashMap<String, TES3Object>,
) -> Vec<TES3Object> {
    // construct records from both lists
    let mut final_records = records.clone();
    for r in edited_records.iter() {
        final_records.insert(r.0.to_string(), r.1.clone());
    }

    // sort records
    // todo sort all records
    let mut records_vec: Vec<_> = final_records.values().cloned().collect();
    let pos = records_vec
        .iter()
        .position(|e| e.tag_str() == "TES3")
        .unwrap();
    let header = records_vec.remove(pos);
    records_vec.insert(0, header);
    records_vec
}

pub fn save_all(
    records: &mut HashMap<String, TES3Object>,
    edited_records: &mut HashMap<String, TES3Object>,
    plugin_path: &Path,
    toasts: &mut Toasts,
) {
    let mut plugin = Plugin {
        objects: get_records(records, edited_records),
    };
    // save
    let output_path = append_ext("dbg.esp", plugin_path.to_path_buf());
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

pub fn save_patch(
    edited_records: &mut HashMap<String, TES3Object>,
    plugin_path: &Path,
    toasts: &mut Toasts,
) {
    // todo figure out a header
    let mut records_vec: Vec<_> = edited_records.values().cloned().collect();
    records_vec.insert(
        0,
        TES3Object::Header(Header {
            flags: ObjectFlags::empty(),
            version: 1.0_f32,
            file_type: tes3::esp::FileType::Esp,
            author: FixedString("".into()),
            description: FixedString("".into()),
            num_objects: records_vec.len() as u32, //todo correct?
            masters: vec![],
        }),
    );

    // save
    let mut plugin = Plugin {
        objects: records_vec,
    };
    let output_path = append_ext("patch.esp", plugin_path.to_path_buf());
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
