use tes3::esp::editor::EditorList;

use crate::{create_from_tag, get_unique_id, TemplateApp};

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
                egui::Grid::new("conflict_view_grid")
                    .min_col_width(200_f32)
                    .striped(true)
                    .show(ui, |ui| {
                        // first row: mod names
                        // each mod name is a column
                        for mod_hash in conflicts {
                            let vm = self
                                .compare_data
                                .plugins
                                .iter_mut()
                                .find(|e| e.id == *mod_hash)
                                .unwrap();
                            // mod name
                            ui.label(vm.path.file_name().unwrap().to_string_lossy());
                        }
                        ui.end_row();

                        // next rows for each field in the struct
                        let tag = key
                            .split(',')
                            .collect::<Vec<_>>()
                            .first()
                            .unwrap()
                            .to_owned();
                        let instance = create_from_tag(tag).unwrap();
                        for (i, row_name) in instance.get_editor_names().iter().enumerate() {
                            ui.label(row_name);
                            ui.separator();

                            // each conflict is a column
                            //let mut last_value: &mut dyn Editor;
                            for mod_hash in conflicts {
                                let vm = self
                                    .compare_data
                                    .plugins
                                    .iter_mut()
                                    .find(|e| e.id == *mod_hash)
                                    .unwrap();
                                let plugin = &mut vm.plugin;
                                let record = plugin
                                    .objects
                                    .iter_mut()
                                    .find(|e| get_unique_id(e) == key)
                                    .unwrap();
                                let field = record.get_editor_list().get_mut(i).unwrap();

                                field.add_editor(ui, row_name.to_owned());

                                let result = serde_yaml::to_string(field);
                                let str1 = match result {
                                    Ok(t) => t,
                                    Err(e) => "".to_owned(),
                                };
                            }
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
