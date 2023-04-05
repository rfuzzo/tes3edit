use std::collections::HashMap;

use egui_notify::Toasts;

use tes3::esp::traits::editor::Editor;
use tes3::esp::TES3Object;

// #[allow(dead_code)]
// pub(crate) fn record_text_editor_view(
//     ui: &mut egui::Ui,
//     current_text: &mut (String, String),
//     edited_records: &mut HashMap<String, TES3Object>,
//     records: &mut HashMap<String, TES3Object>,
//     toasts: &mut Toasts,
// ) {
//     egui::menu::bar(ui, |ui| {
//         // Revert record button
//         #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
//         if ui.button("Revert").clicked() {
//             let id = current_text.0.clone();
//             // get original record
//             if edited_records.contains_key(&id) {
//                 // remove from edited records
//                 edited_records.remove(&id);
//                 // revert text
//                 *current_text = (
//                     id.clone(),
//                     serde_yaml::to_string(&records[&id]).unwrap_or("Error serializing".to_owned()),
//                 );
//                 toasts
//                     .info("Record reverted")
//                     .set_duration(Some(Duration::from_secs(5)));
//             }
//         }

//         // Save record button
//         #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
//         if ui.button("Save").clicked() {
//             // deserialize
//             let deserialized: Result<TES3Object, _> = serde_yaml::from_str(&current_text.1);
//             if let Ok(record) = deserialized {
//                 // add or update current record to list
//                 edited_records.insert(current_text.0.clone(), record);
//                 toasts
//                     .success("Record saved")
//                     .set_duration(Some(Duration::from_secs(5)));
//             }
//         }
//     });

//     // text editor
//     egui::ScrollArea::vertical().show(ui, |ui| {
//         let _response = ui.add_sized(
//             ui.available_size(),
//             egui::TextEdit::multiline(&mut current_text.1),
//         );
//     });
// }

pub(crate) fn record_editor_view(
    ui: &mut egui::Ui,
    //current_record: &mut TES3Object,
    current_record_id: &mut String,
    edited_records: &mut HashMap<String, TES3Object>,
    records: &mut HashMap<String, TES3Object>,
    _toasts: &mut Toasts,
) {
    //if let Some(record) = current_record {
    // editor menu bar
    // let id = get_unique_id(record);
    // egui::menu::bar(ui, |ui| {
    //     // Revert record button
    //     #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
    //     if ui.button("Revert").clicked() {
    //         // get original record
    //         if edited_records.contains_key(&id) {
    //             // remove from edited records
    //             edited_records.remove(&id);
    //             // revert text
    //             *current_record = Some(&records[&id]);

    //             toasts
    //                 .info("Record reverted")
    //                 .set_duration(Some(Duration::from_secs(5)));
    //         }
    //     }
    // });

    // text editor
    //let widget = egui::ScrollArea::horizontal();
    //ui.add_sized(ui.available_size(), widget);

    //widget.show(ui, |ui| {
    //let _response = ui.add_sized(ui.available_size(), );
    //add_editor_for(ui, current_record);

    //});
    //}

    if edited_records.contains_key(current_record_id) {
        edited_records
            .get_mut(current_record_id)
            .unwrap()
            .add_editor(ui);
    } else {
        records.get_mut(current_record_id).unwrap().add_editor(ui);
    }

    //current_record.editor(ui);
}
