use std::{collections::HashMap, time::Duration};

use egui_notify::Toasts;
use tes3::esp::TES3Object;

pub(crate) fn record_text_editor_view(
    ui: &mut egui::Ui,
    current_text: &mut (String, String),
    edited_records: &mut HashMap<String, TES3Object>,
    records: &mut HashMap<String, TES3Object>,
    toasts: &mut Toasts,
) {
    egui::menu::bar(ui, |ui| {
        // Revert record button
        #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
        if ui.button("Revert").clicked() {
            let id = current_text.0.clone();
            // get original record
            if edited_records.contains_key(&id) {
                // remove from edited records
                edited_records.remove(&id);
                // revert text
                *current_text = (
                    id.clone(),
                    serde_yaml::to_string(&records[&id]).unwrap_or("Error serializing".to_owned()),
                );
                toasts
                    .info("Record reverted")
                    .set_duration(Some(Duration::from_secs(5)));
            }
        }

        // Save record button
        #[cfg(not(target_arch = "wasm32"))] // no Save on web pages!
        if ui.button("Save").clicked() {
            // deserialize
            let deserialized: Result<TES3Object, _> = serde_yaml::from_str(&current_text.1);
            if let Ok(record) = deserialized {
                // add or update current record to list
                edited_records.insert(current_text.0.clone(), record);
                toasts
                    .success("Record saved")
                    .set_duration(Some(Duration::from_secs(5)));
            }
        }
    });

    // text editor
    egui::ScrollArea::vertical().show(ui, |ui| {
        let _response = ui.add_sized(
            ui.available_size(),
            egui::TextEdit::multiline(&mut current_text.1),
        );
    });
}
