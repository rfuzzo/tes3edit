use serde::{Deserialize, Serialize};
use tes3::esp::TES3Object;

pub(crate) fn add_editor_for(ui: &mut egui::Ui, current_record: &mut Option<TES3Object>) {
    if let Some(record) = current_record {
        match record {
            TES3Object::Header(o) => add_header_editor(ui, o),
            TES3Object::GameSetting(_) => todo!(),
            TES3Object::GlobalVariable(_) => todo!(),
            TES3Object::Class(_) => todo!(),
            TES3Object::Faction(_) => todo!(),
            TES3Object::Race(_) => todo!(),
            TES3Object::Sound(_) => todo!(),
            TES3Object::SoundGen(_) => todo!(),
            TES3Object::Skill(_) => todo!(),
            TES3Object::MagicEffect(_) => todo!(),
            TES3Object::Script(_) => todo!(),
            TES3Object::Region(_) => todo!(),
            TES3Object::Birthsign(_) => todo!(),
            TES3Object::StartScript(_) => todo!(),
            TES3Object::LandscapeTexture(_) => todo!(),
            TES3Object::Spell(_) => todo!(),
            TES3Object::Static(_) => todo!(),
            TES3Object::Door(_) => todo!(),
            TES3Object::MiscItem(o) => add_misc_editor(ui, o),
            TES3Object::Weapon(_) => todo!(),
            TES3Object::Container(_) => todo!(),
            TES3Object::Creature(_) => todo!(),
            TES3Object::Bodypart(_) => todo!(),
            TES3Object::Light(_) => todo!(),
            TES3Object::Enchanting(_) => todo!(),
            TES3Object::Npc(_) => todo!(),
            TES3Object::Armor(_) => todo!(),
            TES3Object::Clothing(_) => todo!(),
            TES3Object::RepairItem(_) => todo!(),
            TES3Object::Activator(_) => todo!(),
            TES3Object::Apparatus(_) => todo!(),
            TES3Object::Lockpick(_) => todo!(),
            TES3Object::Probe(_) => todo!(),
            TES3Object::Ingredient(_) => todo!(),
            TES3Object::Book(_) => todo!(),
            TES3Object::Alchemy(_) => todo!(),
            TES3Object::LeveledItem(_) => todo!(),
            TES3Object::LeveledCreature(_) => todo!(),
            TES3Object::Cell(_) => todo!(),
            TES3Object::Landscape(_) => todo!(),
            TES3Object::PathGrid(_) => todo!(),
            TES3Object::Dialogue(_) => todo!(),
            TES3Object::DialogueInfo(_) => todo!(),
        }
    }
}

fn add_header_editor(ui: &mut egui::Ui, o: &mut tes3::esp::Header) {
    egui::Grid::new("main_grid").show(ui, |ui| {
        ui.label("flags");
        add_yaml_editor(ui, &mut o.flags);
        ui.end_row();

        ui.label("version");
        ui.add(egui::DragValue::new(&mut o.version).speed(0.1));
        ui.end_row();

        ui.label("file_type");
        add_yaml_editor(ui, &mut o.file_type);
        ui.end_row();

        ui.label("author");
        add_yaml_editor(ui, &mut o.author);
        ui.end_row();

        ui.label("description");
        add_yaml_editor(ui, &mut o.description);
        ui.end_row();

        ui.label("num_objects");
        ui.add(egui::DragValue::new(&mut o.num_objects).speed(1));
        ui.end_row();

        ui.label("masters");
        add_yaml_editor(ui, &mut o.masters);
        ui.end_row();
    });
}

fn add_misc_editor(ui: &mut egui::Ui, o: &mut tes3::esp::MiscItem) {
    ui.label("flags");
    add_yaml_editor(ui, &mut o.flags);
    ui.end_row();

    // egui::Grid::new("main_grid").show(ui, |ui| {
    //     for (label, widget) in get_editors_misc(ui, o) {
    //         ui.label(label);
    //         ui.add(widget.into());
    //         ui.end_row();
    //     }
    // });

    ui.label("id");
    ui.text_edit_singleline(&mut o.id);
    ui.end_row();

    ui.label("name");
    ui.text_edit_singleline(&mut o.name);
    ui.end_row();

    ui.label("script");
    ui.text_edit_singleline(&mut o.script);
    ui.end_row();

    ui.label("mesh");
    ui.text_edit_singleline(&mut o.mesh);
    ui.end_row();

    ui.label("icon");
    ui.text_edit_singleline(&mut o.icon);
    ui.end_row();

    ui.label("data");
    add_yaml_editor(ui, &mut o.data);
    ui.end_row();
}

// fn get_editors_misc(
//     ui: &mut egui::Ui,
//     o: &mut tes3::esp::MiscItem,
// ) -> HashMap<String, Box<dyn Widget>> {
//     let mut map: HashMap<String, Box<dyn Widget>> = HashMap::new();
//     map.insert("id".into(), Box::new(egui::TextEdit::singleline(&mut o.id)));
//     map.insert(
//         "name".into(),
//         Box::new(egui::TextEdit::singleline(&mut o.name)),
//     );
//     map.insert(
//         "script".into(),
//         Box::new(egui::TextEdit::singleline(&mut o.script)),
//     );
//     map.insert(
//         "mesh".into(),
//         Box::new(egui::TextEdit::singleline(&mut o.mesh)),
//     );
//     map.insert(
//         "icon".into(),
//         Box::new(egui::TextEdit::singleline(&mut o.icon)),
//     );

//     map
// }

/// Generic text editor
fn add_yaml_editor<T>(ui: &mut egui::Ui, object: &mut T)
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    let mut text = serde_yaml::to_string(&object).unwrap();
    if ui.text_edit_multiline(&mut text).changed() {
        let result: Result<T, _> = serde_yaml::from_str(&text);
        if let Ok(out) = result {
            *object = out;
        }
    }
}
