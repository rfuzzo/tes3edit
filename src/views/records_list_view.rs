use std::collections::HashMap;

use tes3::esp::{EditorId, TES3Object, TypeInfo};

use crate::{get_unique_id, TemplateApp};

impl TemplateApp {
    pub fn records_list_view(&mut self, ui: &mut egui::Ui) {
        // heading
        if let Some(kvp) = self.plugins.get_key_value(&self.current_plugin_id) {
            let name = std::path::Path::new(&kvp.0)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            ui.heading(name);
        } else {
            ui.heading("Records");
        }

        // search bar
        let _search_text = self.search_text.clone();
        ui.horizontal(|ui| {
            ui.label("Filter: ");
            ui.text_edit_singleline(&mut self.search_text);
        });
        ui.separator();

        let tags = get_all_tags();
        // editor for a specific plugin
        if let Some(data) = self.plugins.get_mut(&self.current_plugin_id) {
            // a plugin was found
            if (_search_text != self.search_text) || data.sorted_records.is_empty() {
                // regenerate records
                let mut filtered_records_by_tag: HashMap<String, Vec<String>> = HashMap::default();
                for tag in &tags {
                    filtered_records_by_tag.insert(tag.clone(), vec![]);
                    let mut records_inner: Vec<&TES3Object> = data
                        .records
                        .values()
                        .filter(|r| r.tag_str() == tag)
                        .collect();

                    // search filter
                    if !self.search_text.is_empty() {
                        records_inner = records_inner
                            .iter()
                            .copied()
                            .filter(|p| {
                                get_unique_id(p)
                                    .to_lowercase()
                                    .contains(self.search_text.to_lowercase().as_str())
                            })
                            .collect();
                    }
                    if records_inner.is_empty() {
                        continue;
                    }

                    records_inner.sort_by(|a, b| a.editor_id().cmp(&b.editor_id()));
                    filtered_records_by_tag.insert(
                        tag.clone(),
                        records_inner.iter().map(|e| get_unique_id(e)).collect(),
                    );
                }
                data.sorted_records = filtered_records_by_tag;
            }

            // logic
            let mut record_ids_to_delete = vec![];

            // add button // todo move?
            // if ui.button("Add record").clicked() {
            //     // get type and id
            // }

            // the record list
            egui::ScrollArea::vertical().show(ui, |ui| {
                // order by tags
                for tag in tags {
                    let records_of_tag: Vec<&TES3Object> = data.sorted_records[&tag]
                        .iter()
                        .filter_map(|e| data.records.get(e))
                        .collect();

                    if records_of_tag.is_empty() {
                        continue;
                    }

                    // add headers and subitems
                    let tag_header = egui::CollapsingHeader::new(tag).show(ui, |ui| {
                        // add records
                        // sort

                        for record_ptr in records_of_tag.iter() {
                            let record = *record_ptr;
                            let id = get_unique_id(record);

                            // annotations
                            let mut label = record.editor_id().to_string();
                            // hack for header record
                            if label.is_empty() && record.tag_str() == "TES3" {
                                label = "Header".into();
                            }
                            // modified records
                            if data.edited_records.contains_key(&id) {
                                label = format!("{}*", label);
                                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                            } else {
                                ui.visuals_mut().override_text_color = None;
                            }

                            // record list item view
                            let w = egui::Label::new(label).sense(egui::Sense::click());
                            let response = ui.add(w);
                            // context menu
                            response.clone().context_menu(|ui| {
                                if ui.button("Delete").clicked() {
                                    // delete a record
                                    record_ids_to_delete.push(id.clone());
                                    ui.close_menu();
                                }
                            });

                            // selected event
                            if response.clicked() {
                                // cleanup old records
                                let mut to_remove = Vec::new();
                                for (key, edited_record) in data.edited_records.clone() {
                                    if let Some(original) = data.records.get(&key) {
                                        // remove if no change
                                        if original.eq(&edited_record) {
                                            to_remove.push(key);
                                        }
                                    }
                                }
                                for r in to_remove {
                                    data.edited_records.remove(&r);
                                }

                                // add a copy of this record to the edited records
                                if !data.edited_records.contains_key(&id) {
                                    data.edited_records.insert(id.clone(), record.clone());
                                }

                                data.current_record_id = Some(id);
                            }
                        }
                    });

                    // context menu of tag header
                    tag_header.header_response.context_menu(|ui| {
                        // if ui.button("Add").clicked() {
                        //     // todo
                        //     let instance = Create(tag);
                        //     self.records.insert(get_unique_id(&instance), instance);

                        //     ui.close_menu();
                        // }
                        if ui.button("Delete all").clicked() {
                            for r in records_of_tag.iter() {
                                let id = get_unique_id(r);
                                record_ids_to_delete.push(id.clone());
                            }
                            ui.close_menu();
                        }
                    });
                }

                ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
            });

            // delete stuff
            record_ids_to_delete.dedup();
            for k in record_ids_to_delete {
                data.records.remove(&k);
            }
        }
    }
}

/// todo super dumb but I can't be bothered to mess around with enums now
fn get_all_tags() -> Vec<String> {
    let v = vec![
        "TES3", "GMST", "GLOB", "CLAS", "FACT", "RACE", "SOUN", "SNDG", "SKIL", "MGEF", "SCPT",
        "REGN", "BSGN", "SSCR", "LTEX", "SPEL", "STAT", "DOOR", "MISC", "WEAP", "CONT", "CREA",
        "BODY", "LIGH", "ENCH", "NPC_", "ARMO", "CLOT", "REPA", "ACTI", "APPA", "LOCK", "PROB",
        "INGR", "BOOK", "ALCH", "LEVI", "LEVC", "CELL", "LAND", "PGRD", "DIAL", "INFO",
    ];
    v.iter().map(|e| e.to_string()).collect::<Vec<String>>()
}

// fn Create(tag: &str) -> TES3Object {
//     todo!()
// }
