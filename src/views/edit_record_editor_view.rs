use tes3::esp::editor::Editor;

use crate::TemplateApp;

impl TemplateApp {
    pub fn record_editor_view(&mut self, ui: &mut egui::Ui) {
        // editor for a specific plugin
        if let Some(plugin_data) = self
            .edit_data
            .plugins
            .iter_mut()
            .find(|p| p.id == self.edit_data.current_plugin_id)
        {
            // a plugin was found
            if let Some(current_record_id) = &plugin_data.selected_record_id {
                // editor menu bar
                egui::menu::bar(ui, |ui| {
                    // Revert record button
                    if ui.button("Revert").clicked() {
                        // get original record
                        if plugin_data.edited_records.contains_key(current_record_id) {
                            // remove from edited records
                            plugin_data.edited_records.remove(current_record_id);

                            self.toasts.info("Record reverted");
                        }
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // get the record to edit from the original records or the edited ones
                    if plugin_data.edited_records.contains_key(current_record_id) {
                        let object = plugin_data
                            .edited_records
                            .get_mut(current_record_id)
                            .unwrap();
                        object.add_editor(ui, current_record_id.to_owned());
                    } else if plugin_data.records.contains_key(current_record_id) {
                        let object = plugin_data.records.get_mut(current_record_id).unwrap();
                        object.add_editor(ui, current_record_id.to_owned());
                    }
                });
            }
        }
    }
}
