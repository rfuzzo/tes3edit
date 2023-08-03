use crate::{get_unique_id, CompareData, TemplateApp, UiData};
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
        if self.compare_data.ui_data_id != key {
            self.compare_data.ui_data = None;
        }

        if self.compare_data.ui_data.is_none() {
            self.compare_data.ui_data = Some(get_ui_data(&self.compare_data, key.clone()));
            self.compare_data.ui_data_id = key.clone();
        }

        if let Some(ui_data) = &self.compare_data.ui_data {
            // horizontal scrolling
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    // start grid
                    egui::Grid::new("conflict_view_grid")
                        .min_col_width(200_f32)
                        .max_col_width(200_f32)
                        .striped(true)
                        .show(ui, |ui| {
                            // update UI
                            ui.label("Name");
                            for n in &ui_data.plugins {
                                ui.label(n.to_string());
                            }
                            ui.end_row();

                            for (field_name, fields) in &ui_data.rows {
                                //let fields = &rows[field_name];
                                // one row

                                // start with field name
                                ui.label(field_name.to_string());
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
            });
        }
    }
}

/// .
///
/// # Panics
///
/// Panics if .
fn get_ui_data(compare_data: &CompareData, key: String) -> UiData {
    if let Some(conflicts) = compare_data.map.get(&key) {
        let mut vms: Vec<(String, tes3::esp::Plugin)> = vec![];
        for mod_hash in conflicts {
            let vm = compare_data
                .plugins
                .iter()
                .find(|e| e.id == *mod_hash)
                .unwrap();
            // mod name
            let mod_name = vm.path.file_name().unwrap().to_string_lossy().to_string();
            vms.push((mod_name, vm.plugin.clone()));
        }

        // get column map
        let mut columns: Vec<(String, Vec<(String, String)>)> = vec![];
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

            columns.push((id.to_string(), fields));
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
        let mut plugins: Vec<String> = vec![];
        for (n, column) in columns {
            for (field_name, value) in column {
                if rows.contains_key(&field_name) {
                    let mut v = rows[&field_name].clone();
                    v.push(value);
                    rows.insert(field_name, v);
                } else {
                    rows.insert(field_name, vec![value]);
                }
            }
            plugins.push(n);
        }

        let mut rows_ordered: Vec<(String, Vec<String>)> = vec![];
        for n in field_names {
            let f = rows.get(&n).unwrap();
            rows_ordered.push((n.to_string(), f.clone()));
        }

        UiData {
            id: key,
            rows: rows_ordered,
            plugins,
        }
    } else {
        let rows_ordered: Vec<(String, Vec<String>)> = vec![];
        let plugins: Vec<String> = vec![];
        UiData {
            id: key,
            rows: rows_ordered,
            plugins,
        }
    }
}
