use crate::{get_unique_id, TemplateApp};
use egui::epaint::ahash::HashMap;
use tes3::esp::editor::Editor;

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
                    .max_col_width(200_f32)
                    .striped(true)
                    .show(ui, |ui| {
                        // first row: mod names
                        ui.label("Name");
                        // each mod name is a column
                        let mut vms: Vec<(u64, tes3::esp::Plugin)> = vec![];
                        for mod_hash in conflicts {
                            let vm = self
                                .compare_data
                                .plugins
                                .iter()
                                .find(|e| e.id == *mod_hash)
                                .unwrap();
                            // mod name
                            ui.label(vm.path.file_name().unwrap().to_string_lossy());
                            vms.push((vm.id, vm.plugin.clone()));
                        }
                        ui.end_row();

                        // get column map
                        let mut columns: Vec<(u64, Vec<(String, String)>)> = vec![];
                        for (id, plugin) in vms.iter_mut() {
                            let mut fields: Vec<(String, String)> = vec![];

                            let record = plugin
                                .objects
                                .iter_mut()
                                .find(|e| get_unique_id(e) == key)
                                .unwrap();

                            // get fields of record
                            if let Some(record_fields) = record.get_editor_list() {
                                for (field_name, field) in record_fields {
                                    if let Some(sub) = field.get_editor_list() {
                                        // if that field is atype that has itself fields
                                        //we need to recursively get them
                                        for (field_name2, field2) in sub {
                                            // fields.push((field_name2, field2.to_json()));
                                            fields.push((field_name2.to_owned(), field2.to_json()));
                                        }
                                    } else {
                                        fields.push((field_name.to_owned(), field.to_json()));
                                    }
                                }
                            }

                            columns.push((*id, fields));
                        }

                        // display ui
                        // color conflicts
                        // transform to rows
                        let field_names = columns
                            .first()
                            .unwrap()
                            .1
                            .iter()
                            .map(|x| x.0.clone())
                            .collect::<Vec<_>>();
                        let mut rows: HashMap<String, Vec<String>> = HashMap::default();
                        for (_, column) in columns {
                            for (field_name, value) in column {
                                if rows.contains_key(&field_name) {
                                    let mut v = rows[&field_name].clone();
                                    v.push(value);
                                    rows.insert(field_name, v);
                                } else {
                                    rows.insert(field_name, vec![value]);
                                }
                            }
                        }

                        for field_name in field_names {
                            let fields = &rows[&field_name];
                            // one row
                            // start with field name
                            ui.label(field_name);
                            // loop through fields
                            for (i, field) in fields.iter().enumerate() {
                                // display field as String

                                if i > 0 {
                                    let last_str = &fields[i - 1];
                                    if last_str != field {
                                        // change color
                                        ui.visuals_mut().override_text_color =
                                            Some(egui::Color32::RED);
                                    } else {
                                        ui.visuals_mut().override_text_color = None;
                                    }
                                }

                                ui.label(field);
                            }
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
