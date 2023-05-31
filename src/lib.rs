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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ERecordType {
    TES3,
    GMST,
    GLOB,
    CLAS,
    FACT,
    RACE,
    SOUN,
    SNDG,
    SKIL,
    MGEF,
    SCPT,
    REGN,
    BSGN,
    SSCR,
    LTEX,
    SPEL,
    STAT,
    DOOR,
    MISC,
    WEAP,
    CONT,
    CREA,
    BODY,
    LIGH,
    ENCH,
    NPC_,
    ARMO,
    CLOT,
    REPA,
    ACTI,
    APPA,
    LOCK,
    PROB,
    INGR,
    BOOK,
    ALCH,
    LEVI,
    LEVC,
    CELL,
    LAND,
    PGRD,
    DIAL,
    INFO,
}

impl From<&str> for ERecordType {
    fn from(value: &str) -> Self {
        match value {
            "TES3" => ERecordType::TES3,
            "GMST" => ERecordType::GMST,
            "GLOB" => ERecordType::GLOB,
            "CLAS" => ERecordType::CLAS,
            "FACT" => ERecordType::FACT,
            "RACE" => ERecordType::RACE,
            "SOUN" => ERecordType::SOUN,
            "SNDG" => ERecordType::SNDG,
            "SKIL" => ERecordType::SKIL,
            "MGEF" => ERecordType::MGEF,
            "SCPT" => ERecordType::SCPT,
            "REGN" => ERecordType::REGN,
            "BSGN" => ERecordType::BSGN,
            "SSCR" => ERecordType::SSCR,
            "LTEX" => ERecordType::LTEX,
            "SPEL" => ERecordType::SPEL,
            "STAT" => ERecordType::STAT,
            "DOOR" => ERecordType::DOOR,
            "MISC" => ERecordType::MISC,
            "WEAP" => ERecordType::WEAP,
            "CONT" => ERecordType::CONT,
            "CREA" => ERecordType::CREA,
            "BODY" => ERecordType::BODY,
            "LIGH" => ERecordType::LIGH,
            "ENCH" => ERecordType::ENCH,
            "NPC_" => ERecordType::NPC_,
            "ARMO" => ERecordType::ARMO,
            "CLOT" => ERecordType::CLOT,
            "REPA" => ERecordType::REPA,
            "ACTI" => ERecordType::ACTI,
            "APPA" => ERecordType::APPA,
            "LOCK" => ERecordType::LOCK,
            "PROB" => ERecordType::PROB,
            "INGR" => ERecordType::INGR,
            "BOOK" => ERecordType::BOOK,
            "ALCH" => ERecordType::ALCH,
            "LEVI" => ERecordType::LEVI,
            "LEVC" => ERecordType::LEVC,
            "CELL" => ERecordType::CELL,
            "LAND" => ERecordType::LAND,
            "PGRD" => ERecordType::PGRD,
            "DIAL" => ERecordType::DIAL,
            "INFO" => ERecordType::INFO,
            _ => {
                panic!("ArgumentException")
            }
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

/// super dumb but I can't be bothered to mess around with enums now
pub fn get_all_tags() -> Vec<String> {
    let v = vec![
        "TES3", "GMST", "GLOB", "CLAS", "FACT", "RACE", "SOUN", "SNDG", "SKIL", "MGEF", "SCPT",
        "REGN", "BSGN", "SSCR", "LTEX", "SPEL", "STAT", "DOOR", "MISC", "WEAP", "CONT", "CREA",
        "BODY", "LIGH", "ENCH", "NPC_", "ARMO", "CLOT", "REPA", "ACTI", "APPA", "LOCK", "PROB",
        "INGR", "BOOK", "ALCH", "LEVI", "LEVC", "CELL", "LAND", "PGRD", "DIAL", "INFO",
    ];
    v.iter().map(|e| e.to_string()).collect::<Vec<String>>()
}

// Refactor this after e3
/// Create a new record of the given tag
pub fn create_from_tag(tag: &str) -> Option<TES3Object> {
    create(ERecordType::from(tag))
}

/// Create a new record of the given type
fn create(e: ERecordType) -> Option<TES3Object> {
    match e {
        ERecordType::TES3 => Some(TES3Object::from(tes3::esp::Header::default())),
        ERecordType::GMST => Some(TES3Object::from(tes3::esp::GameSetting::default())),
        ERecordType::GLOB => Some(TES3Object::from(tes3::esp::GlobalVariable::default())),
        ERecordType::CLAS => Some(TES3Object::from(tes3::esp::Class::default())),
        ERecordType::FACT => Some(TES3Object::from(tes3::esp::Faction::default())),
        ERecordType::RACE => Some(TES3Object::from(tes3::esp::Race::default())),
        ERecordType::SOUN => Some(TES3Object::from(tes3::esp::Sound::default())),
        ERecordType::SNDG => Some(TES3Object::from(tes3::esp::SoundGen::default())),
        ERecordType::SKIL => Some(TES3Object::from(tes3::esp::Skill::default())),
        ERecordType::MGEF => Some(TES3Object::from(tes3::esp::MagicEffect::default())),
        ERecordType::SCPT => Some(TES3Object::from(tes3::esp::Script::default())),
        ERecordType::REGN => Some(TES3Object::from(tes3::esp::Region::default())),
        ERecordType::BSGN => Some(TES3Object::from(tes3::esp::Birthsign::default())),
        ERecordType::SSCR => Some(TES3Object::from(tes3::esp::StartScript::default())),
        ERecordType::LTEX => Some(TES3Object::from(tes3::esp::LandscapeTexture::default())),
        ERecordType::SPEL => Some(TES3Object::from(tes3::esp::Spell::default())),
        ERecordType::STAT => Some(TES3Object::from(tes3::esp::Static::default())),
        ERecordType::DOOR => Some(TES3Object::from(tes3::esp::Door::default())),
        ERecordType::MISC => Some(TES3Object::from(tes3::esp::MiscItem::default())),
        ERecordType::WEAP => Some(TES3Object::from(tes3::esp::Weapon::default())),
        ERecordType::CONT => Some(TES3Object::from(tes3::esp::Container::default())),
        ERecordType::CREA => Some(TES3Object::from(tes3::esp::Creature::default())),
        ERecordType::BODY => Some(TES3Object::from(tes3::esp::Bodypart::default())),
        ERecordType::LIGH => Some(TES3Object::from(tes3::esp::Light::default())),
        ERecordType::ENCH => Some(TES3Object::from(tes3::esp::Enchanting::default())),
        ERecordType::NPC_ => Some(TES3Object::from(tes3::esp::Npc::default())),
        ERecordType::ARMO => Some(TES3Object::from(tes3::esp::Armor::default())),
        ERecordType::CLOT => Some(TES3Object::from(tes3::esp::Clothing::default())),
        ERecordType::REPA => Some(TES3Object::from(tes3::esp::RepairItem::default())),
        ERecordType::ACTI => Some(TES3Object::from(tes3::esp::Activator::default())),
        ERecordType::APPA => Some(TES3Object::from(tes3::esp::Apparatus::default())),
        ERecordType::LOCK => Some(TES3Object::from(tes3::esp::Lockpick::default())),
        ERecordType::PROB => Some(TES3Object::from(tes3::esp::Probe::default())),
        ERecordType::INGR => Some(TES3Object::from(tes3::esp::Ingredient::default())),
        ERecordType::BOOK => Some(TES3Object::from(tes3::esp::Book::default())),
        ERecordType::ALCH => Some(TES3Object::from(tes3::esp::Alchemy::default())),
        ERecordType::LEVI => Some(TES3Object::from(tes3::esp::LeveledItem::default())),
        ERecordType::LEVC => Some(TES3Object::from(tes3::esp::LeveledCreature::default())),
        ERecordType::CELL => Some(TES3Object::from(tes3::esp::Cell::default())),
        ERecordType::LAND => Some(TES3Object::from(tes3::esp::Landscape::default())),
        ERecordType::PGRD => Some(TES3Object::from(tes3::esp::PathGrid::default())),
        ERecordType::DIAL => Some(TES3Object::from(tes3::esp::Dialogue::default())),
        ERecordType::INFO => Some(TES3Object::from(tes3::esp::DialogueInfo::default())),
    }
}

//////////////////////////////////////////
// App

/// Plugin Viewmodel in-app
pub struct PluginMetadata {
    pub id: String,
    pub full_path: Option<PathBuf>,
    pub records: HashMap<String, TES3Object>,
    /// cached ids of all records and edited records of this plugin
    pub cached_ids: HashMap<String, Vec<String>>,
    pub edited_records: HashMap<String, TES3Object>,
    pub selected_record_id: Option<String>,
}

impl PluginMetadata {
    pub fn new(id: String, full_path: Option<PathBuf>) -> Self {
        Self {
            id,
            full_path,
            records: HashMap::default(),
            cached_ids: HashMap::default(),
            edited_records: HashMap::default(),
            selected_record_id: None,
        }
    }

    /// Regenerates record id cache of this plugin
    pub fn regenerate_id_cache(&mut self, filter_text: &String) {
        let mut filtered_ids_by_tag: HashMap<String, Vec<String>> = HashMap::default();
        for tag in get_all_tags() {
            filtered_ids_by_tag.insert(tag.clone(), vec![]);
            let mut records_inner = self.get_record_ids_for_tag(Some(tag.as_str()));

            // search filter
            if !filter_text.is_empty() {
                records_inner = records_inner
                    .iter()
                    .filter(|p| {
                        p.to_lowercase()
                            .contains(filter_text.to_lowercase().as_str())
                    })
                    .map(|e| e.to_owned())
                    .collect::<Vec<_>>();
            }

            if records_inner.is_empty() {
                continue;
            }

            records_inner.sort();
            filtered_ids_by_tag.insert(tag.clone(), records_inner);
        }
        self.cached_ids = filtered_ids_by_tag;
    }

    /// Get all record ids of this plugin for a specific tag
    pub fn get_record_ids_for_tag(&self, tag_or_none: Option<&str>) -> Vec<String> {
        let mut is_filter = false;
        let mut tag = "";
        if let Some(t) = tag_or_none {
            is_filter = true;
            tag = t;
        }

        // get records
        let mut records_inner = self
            .records
            .values()
            .filter(|r| is_filter && r.tag_str() == tag)
            .map(get_unique_id)
            .collect::<Vec<_>>();

        // get edited records
        for id in self
            .edited_records
            .values()
            .filter(|r| r.tag_str() == tag)
            .map(get_unique_id)
        {
            if !records_inner.contains(&id) {
                records_inner.push(id);
            }
        }
        records_inner
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
