#![warn(clippy::all, rust_2018_idioms)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod app;
mod app_ui;
mod views;

pub use app::TemplateApp;

use egui::{Color32, ColorImage, TextureHandle};
use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tes3::esp::{Cell, EditorId, Landscape, Plugin, TES3Object, TypeInfo};

static GRID: usize = 9;

#[derive(Default)]
pub struct MapData {
    pub path: Option<PathBuf>,
    pub plugins: Vec<MapItemViewModel>,

    pub cells: HashMap<(i32, i32), Cell>,
    pub landscape: HashMap<(i32, i32), Landscape>,
    /// Map cell record ids to grid
    pub cell_ids: HashMap<String, (i32, i32)>,
    /// Map landscape record ids to grid
    pub land_ids: HashMap<String, (i32, i32)>,

    pub bounds_x: (i32, i32),
    pub bounds_y: (i32, i32),
    pub selected_id: String,
    pub hover_pos: (i32, i32),

    // painter
    pub refresh_requested: bool,
    pub texture_handle: Option<TextureHandle>,
}

#[derive(Default)]
pub struct MapItemViewModel {
    pub id: u64,
    pub path: PathBuf,

    pub enabled: bool,
    /// The actual plugin in memory
    pub plugin: Plugin,
}
impl MapItemViewModel {
    pub fn get_name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
}

#[derive(Default)]
pub struct CompareData {
    pub path: Option<PathBuf>,
    pub plugins: Vec<CompareItemViewModel>,

    // these must be in sync
    pub map: HashMap<String, Vec<u64>>,
    pub conflicting_ids: Vec<String>,

    pub selected_id: String,
}

#[derive(Default)]
pub struct CompareItemViewModel {
    pub id: u64,
    pub path: PathBuf,

    pub enabled: bool,
    /// The actual plugin in memory
    pub plugin: Plugin,
    /// A list of all records by unique id of that plugin
    pub records: Vec<String>,
}
impl CompareItemViewModel {
    pub fn get_name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
}

/// Gets a hash from a Pathbuf
pub fn get_path_hash(e: &std::path::PathBuf) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(e, &mut hasher);
    std::hash::Hasher::finish(&hasher)
}

/// Slow conflict lookup between plugins
pub fn generate_conflict_map(data: &CompareData) -> std::collections::HashMap<String, Vec<u64>> {
    let mut conflict_map: HashMap<String, Vec<u64>> = HashMap::default();
    // get conflicts
    for base_mod in data.plugins.iter().filter(|e| e.enabled) {
        // go through mod1s records
        for record_id in base_mod.records.iter() {
            // now check each of the other mods
            for other_mod in data.plugins.iter().filter(|e| e.enabled) {
                // don't check yourself
                if base_mod.id == other_mod.id {
                    continue;
                }

                // if any conflict found
                if other_mod.records.contains(record_id) {
                    // insert empty vector on first
                    if !conflict_map.contains_key(record_id) {
                        conflict_map.insert(record_id.clone(), vec![]);
                    }

                    // update map
                    let mut v = conflict_map[record_id].clone();
                    v.push(other_mod.id);
                    conflict_map.insert(record_id.clone(), v);
                }
            }
        }
    }
    conflict_map
}

/// App States
#[derive(Default, PartialEq, Debug)]
pub enum EAppState {
    #[default]
    Main,
    Compare,
    Map,
}

/// Modal windows
#[derive(Default, PartialEq, Debug)]
pub enum EModalState {
    #[default]
    None,
    ModalCompareInit,
    MapInit,
}

/// Catpuccino themes
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub enum ETheme {
    Frappe,
    Latte,
    Macchiato,
    Mocha,
}

pub fn get_theme(theme: &ETheme) -> catppuccin_egui::Theme {
    match theme {
        ETheme::Frappe => catppuccin_egui::FRAPPE,
        ETheme::Latte => catppuccin_egui::LATTE,
        ETheme::Macchiato => catppuccin_egui::MACCHIATO,
        ETheme::Mocha => catppuccin_egui::MOCHA,
    }
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

#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Display)]
pub enum ERecordType {
    TES3,
    ACTI,
    ALCH,
    APPA,
    ARMO,
    BODY,
    BOOK,
    BSGN,
    CELL,
    CLAS,
    CLOT,
    CONT,
    CREA,
    DIAL,
    DOOR,
    ENCH,
    FACT,
    GLOB,
    GMST,
    INFO,
    INGR,
    LAND,
    LEVC,
    LEVI,
    LIGH,
    LOCK,
    LTEX,
    MGEF,
    MISC,
    NPC_,
    PGRD,
    PROB,
    RACE,
    REGN,
    REPA,
    SCPT,
    SKIL,
    SNDG,
    SOUN,
    SPEL,
    SSCR,
    STAT,
    WEAP,
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

    pub fn clear_cache(&mut self) {
        self.cached_ids.clear();
    }

    /// Regenerates record id cache of this plugin
    pub fn regenerate_id_cache(&mut self, filter_text: &String) {
        self.clear_cache();

        let mut ids = self
            .get_record_ids()
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        // search filter
        if !filter_text.is_empty() {
            ids = ids
                .iter()
                .filter(|p| {
                    p.to_lowercase()
                        .contains(filter_text.to_lowercase().as_str())
                })
                .map(|e| e.to_owned())
                .collect::<Vec<_>>();
        }

        // group by tag
        let mut grouped = HashMap::default();
        for tag in get_all_tags() {
            let mut ids_by_tag = ids
                .iter()
                .filter(|p| p.split(',').collect::<Vec<_>>().first().unwrap() == &tag)
                .map(|e| e.to_owned())
                .collect::<Vec<_>>();
            ids_by_tag.sort();
            grouped.insert(tag, ids_by_tag);
        }

        self.cached_ids = grouped;
    }

    /// Returns the get records of this [`PluginMetadata`].
    fn get_record_ids(&self) -> Vec<&String> {
        let records = self.records.keys();
        let edited_ids = self.edited_records.keys();
        // for r in self.edited_records.iter() {
        //     final_records.insert(r.0.to_string(), r.1.clone());
        // }

        records.into_iter().chain(edited_ids).collect::<Vec<_>>()
    }

    /// Get assembled records in-app
    ///
    /// # Panics
    ///
    /// Panics if no header found
    fn get_records_sorted(&self) -> Vec<TES3Object> {
        // construct records from both lists
        let mut final_records = self.records.clone();
        for r in self.edited_records.iter() {
            final_records.insert(r.0.to_string(), r.1.clone());
        }

        // sort records
        // todo sort all records, header first
        let mut records: Vec<_> = final_records.values().cloned().collect();
        let pos = records.iter().position(|e| e.tag_str() == "TES3").unwrap();
        let header = records.remove(pos);
        records.insert(0, header);

        records
    }
}

/// Saves records as plugin to the specified path
/// If overwrite is not specified, appends new.esp as extension
pub fn save_plugin<P>(
    data: &PluginMetadata,
    plugin_path: P,
    toasts: &mut Toasts,
    overwrite: bool,
) -> bool
where
    P: AsRef<Path>,
{
    let mut plugin = Plugin {
        objects: data.get_records_sorted(),
    };
    // save
    let mut output_path = plugin_path.as_ref().to_path_buf();
    if !overwrite {
        output_path = plugin_path.as_ref().with_extension("new.esp");
    }

    match plugin.save_path(output_path) {
        Ok(_) => {
            toasts.success("Plugin saved");
            true
        }
        Err(_) => {
            toasts.error("Could not save plugin");
            false
        }
    }
}

/// Saves a plugin as patch, appends patch.esp as extension
///
/// # Panics
///
/// Panics if plugin has no header
pub fn save_patch<P>(data: &PluginMetadata, plugin_path: P, toasts: &mut Toasts) -> bool
where
    P: AsRef<Path>,
{
    let mut records_vec: Vec<_> = data.edited_records.values().cloned().collect();

    // if a header in changed files, then take that one instead of the original one
    // TODO panic here if no header since this is undefined behavior
    let mut header = data.records.get("TES3,").unwrap();
    if let Some(h) = data.edited_records.get("TES3,") {
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
            toasts.success("Plugin saved");
            true
        }
        Err(_) => {
            toasts.error("Could not save plugin");
            false
        }
    }
}

/// maps the input pluginviewmodel vec as list of ids
pub fn get_plugin_names(map: &[PluginMetadata]) -> Vec<String> {
    let mut plugins_sorted: Vec<String> = map.iter().map(|p| p.id.clone()).collect();
    plugins_sorted.sort();
    plugins_sorted
}

/// Get all plugins (esp, omwaddon, omwscripts) in a folder
pub fn get_plugins_in_folder<P>(path: &P, use_omw_plugins: bool) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    // get all plugins
    let mut results: Vec<PathBuf> = vec![];
    if let Ok(plugins) = std::fs::read_dir(path) {
        plugins.for_each(|p| {
            if let Ok(file) = p {
                let file_path = file.path();
                if file_path.is_file() {
                    if let Some(ext_os) = file_path.extension() {
                        let ext = ext_os.to_ascii_lowercase();
                        if ext == "esm"
                            || ext == "esp"
                            || (use_omw_plugins && ext == "omwaddon")
                            || (use_omw_plugins && ext == "omwscripts")
                        {
                            results.push(file_path);
                        }
                    }
                }
            }
        });
    }
    results
}

#[cfg(not(target_arch = "wasm32"))]
fn generate_map(map_data: &mut MapData, _to_screen: egui::emath::RectTransform, ui: &mut egui::Ui) {
    // TODO use slice
    let mut map: Vec<Color32> = vec![];
    let height = ((map_data.bounds_y.0.unsigned_abs() as usize
        + map_data.bounds_y.1.unsigned_abs() as usize)
        + 1)
        * GRID;
    let width = ((map_data.bounds_x.0.unsigned_abs() as usize
        + map_data.bounds_x.1.unsigned_abs() as usize)
        + 1)
        * GRID;

    for grid_y in 0..height {
        for grid_x in (0..width).rev() {
            // we can divide by grid to get the cell and subtract the bounds to get the cell coordinates
            let x = (grid_x / GRID) as i32 + map_data.bounds_x.0;
            let y = (grid_y / GRID) as i32 + map_data.bounds_y.0;

            // get LAND record
            let key = (x, y);
            if let Some(land) = map_data.landscape.get(&key) {
                // get remainder
                let hx = grid_x % GRID;
                let hy = grid_y % GRID;

                let heightmap = land.world_map_data.data.clone().to_vec();
                let mut color = get_map_color(heightmap[hy][hx] as f32);

                // cities
                if map_data.cells.contains_key(&key) {
                    if let Some(map_color) = map_data.cells.get(&key).unwrap().map_color {
                        if hx == 0
                            // || hx == 1
                            // || hx == 7
                            || hx == 8
                            || hy == 0
                            // || hy == 1
                            // || hy == 7
                            || hy == 8
                        {
                            color = Color32::from_rgba_premultiplied(
                                map_color[0],
                                map_color[1],
                                map_color[2],
                                map_color[3],
                            );
                        }
                    }
                }

                map.push(color);
            } else {
                map.push(Color32::TRANSPARENT);
            }
        }
    }

    let mut pixels: Vec<u8> = vec![];
    map.reverse();
    for c in map {
        pixels.push(c.r());
        pixels.push(c.g());
        pixels.push(c.b());
        pixels.push(c.a());
    }

    let size: [usize; 2] = [width, height];
    let image = ColorImage::from_rgba_premultiplied(size, &pixels);
    let texture_handle: TextureHandle = ui.ctx().load_texture("map", image, Default::default());
    map_data.texture_handle = Some(texture_handle);
}

/// https://github.com/NullCascade/morrowind-mods/blob/master/User%20Interface%20Expansion/plugin_source/PatchWorldMap.cpp#L158
fn get_map_color(h: f32) -> Color32 {
    #[derive(Default)]
    struct MyColor {
        pub r: f32,
        pub g: f32,
        pub b: f32,
    }

    let height_data = 16.0 * h;
    let mut clipped_data = height_data / 2048.0;
    clipped_data = (-1.0_f32).max(clipped_data.min(1.0)); // rust wtf

    let mut pixel_color: MyColor = MyColor::default();
    // Above ocean level.
    if height_data >= 0.0 {
        // Darker heightmap threshold.
        if clipped_data > 0.3 {
            let base = (clipped_data - 0.3) * 1.428;
            pixel_color.r = 34.0 - base * 29.0;
            pixel_color.g = 25.0 - base * 20.0;
            pixel_color.b = 17.0 - base * 12.0;
        }
        // Lighter heightmap threshold.
        else {
            let mut base = clipped_data * 8.0;
            if clipped_data > 0.1 {
                base = clipped_data - 0.1 + 0.8;
            }
            pixel_color.r = 66.0 - base * 32.0;
            pixel_color.g = 48.0 - base * 23.0;
            pixel_color.b = 33.0 - base * 16.0;
        }
    }
    // Underwater, fade out towards the water color.
    else {
        pixel_color.r = 38.0 + clipped_data * 14.0;
        pixel_color.g = 56.0 + clipped_data * 20.0;
        pixel_color.b = 51.0 + clipped_data * 18.0;
    }

    Color32::from_rgb(
        pixel_color.r as u8,
        pixel_color.g as u8,
        pixel_color.b as u8,
    )
}
