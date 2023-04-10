use std::collections::HashMap;

use egui_notify::Toasts;

use tes3::esp::traits::editor::Editor;
use tes3::esp::TES3Object;

pub(crate) fn record_editor_view(
    ui: &mut egui::Ui,
    //current_record: &mut TES3Object,
    current_record_id: &String,
    edited_records: &mut HashMap<String, TES3Object>,
    records: &mut HashMap<String, TES3Object>,
    toasts: &mut Toasts,
) {
    // editor menu bar
    egui::menu::bar(ui, |ui| {
        // Revert record button
        if ui.button("Revert").clicked() {
            // get original record
            if edited_records.contains_key(current_record_id) {
                // remove from edited records
                edited_records.remove(current_record_id);

                toasts
                    .info("Record reverted")
                    .set_duration(Some(std::time::Duration::from_secs(5)));
            }
        }
    });

    let scroll_area = egui::ScrollArea::vertical();
    scroll_area.show(ui, |ui| {
        // get the record to edit from the original records or the edited ones
        if edited_records.contains_key(current_record_id) {
            edited_records
                .get_mut(current_record_id)
                .unwrap()
                .add_editor(ui, None);
        } else {
            records
                .get_mut(current_record_id)
                .unwrap()
                .add_editor(ui, None);
        }
    });
}
