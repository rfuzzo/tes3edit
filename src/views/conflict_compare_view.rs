use tes3::esp::editor::Editor;

use crate::{get_unique_id, TemplateApp};

impl TemplateApp {
    pub fn conflict_compare_view(&mut self, ui: &mut egui::Ui) {
        let key = self.compare_data.selected_id.clone();
        if key.is_empty() {
            return;
        }

        // heading
        ui.heading(key.clone());
        ui.separator();

        // main compare ui
        if let Some(conflicts) = self.compare_data.map.get(&key) {
            // horizontal scrolling
            egui::ScrollArea::horizontal().show(ui, |ui| {
                // start grid
                egui::Grid::new("some_unique_id").show(ui, |ui| {
                    // first row: mod names
                    for mod_hash in conflicts {
                        // variables
                        let vm = self
                            .compare_data
                            .plugins
                            .iter_mut()
                            .find(|e| e.id == *mod_hash)
                            .unwrap();
                        // mod name
                        ui.label(vm.path.file_name().unwrap().to_string_lossy());
                        ui.separator();
                    }
                    ui.end_row();

                    // second row: editor
                    for mod_hash in conflicts {
                        // variables
                        let vm = self
                            .compare_data
                            .plugins
                            .iter_mut()
                            .find(|e| e.id == *mod_hash)
                            .unwrap();
                        let plugin = &mut vm.plugin;
                        // record column
                        ui.push_id(format!("{}.{}.rc", mod_hash, key), |ui| {
                            ui.vertical(|ui| {
                                // record editor
                                egui::ScrollArea::vertical()
                                    .min_scrolled_height(600.0)
                                    .show(ui, |ui| {
                                        let record = plugin
                                            .objects
                                            .iter_mut()
                                            .find(|e| get_unique_id(e) == key)
                                            .unwrap();
                                        record.add_editor(ui, format!("{}.{}", mod_hash, key));
                                    });
                            });
                        });
                    }
                    ui.end_row();
                });
            });
        }
    }
}
