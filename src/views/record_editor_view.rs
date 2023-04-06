use std::collections::HashMap;

use egui_notify::Toasts;

use tes3::esp::traits::editor::Editor;
use tes3::esp::TES3Object;

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

    let scroll_area = egui::ScrollArea::vertical();
    scroll_area.show(ui, |ui| {
        if edited_records.contains_key(current_record_id) {
            edited_records
                .get_mut(current_record_id)
                .unwrap()
                .add_editor(ui);
        } else {
            records.get_mut(current_record_id).unwrap().add_editor(ui);
        }
    });
}
