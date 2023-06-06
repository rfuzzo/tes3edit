use crate::TemplateApp;

impl TemplateApp {
    pub fn conflict_list_view(&mut self, ui: &mut egui::Ui) {
        // heading
        ui.heading("Conflicts");
        ui.separator();

        // search bar
        ui.horizontal(|ui| {
            ui.label("Filter: ");
            ui.text_edit_singleline(&mut self.search_text);
        });
        ui.separator();

        // list of conflicting records
        egui::ScrollArea::vertical().show(ui, |ui| {
            for key in self.compare_data.conflicting_ids.iter() {
                // TODO upper and lowercase search
                if !self.search_text.is_empty()
                    && !key
                        .to_lowercase()
                        .contains(&self.search_text.to_lowercase())
                {
                    continue;
                }
                let response = ui.add(egui::Label::new(key.clone()).sense(egui::Sense::click()));
                if response.clicked() {
                    self.compare_data.selected_id = key.to_string();
                }
            }
        });
    }
}
