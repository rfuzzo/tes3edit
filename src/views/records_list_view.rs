use std::collections::HashMap;

use tes3::esp::{EditorId, TES3Object, TypeInfo};

use crate::{get_unique_id, TemplateApp};

impl TemplateApp {
    pub fn records_list_view(&mut self, ui: &mut egui::Ui) {
        // store all tags
        if self.tags.is_none() && !self.records.is_empty() {
            let mut tags: Vec<String> = self.records.values().map(|e| e.tag_str().into()).collect();
            // todo proper sorting
            tags.sort();
            tags.dedup();
            self.tags = Some(tags);
        }

        // search bar
        let _search_text = self.search_text.clone();
        ui.horizontal(|ui| {
            ui.label("Filter: ");
            ui.text_edit_singleline(&mut self.search_text);
        });
        ui.separator();

        // do not render list if no tags
        if self.tags.is_none() {
            return;
        }

        if (_search_text != self.search_text) || self.sorted_records.is_empty() {
            // regenerate records
            let mut filtered_records_by_tag: HashMap<String, Vec<String>> = HashMap::default();
            for tag in self.tags.as_ref().unwrap() {
                filtered_records_by_tag.insert(tag.clone(), vec![]);
                let mut records_inner: Vec<&TES3Object> = self
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
            self.sorted_records = filtered_records_by_tag;
        }

        // logic
        let mut records_to_delete = vec![];

        // the record list
        egui::ScrollArea::vertical().show(ui, |ui| {
            // order by tags
            for tag in self.tags.as_ref().unwrap() {
                let records_of_tag: Vec<&TES3Object> = self.sorted_records[tag]
                    .iter()
                    .map(|e| self.records.get(e).unwrap())
                    .collect();

                if records_of_tag.is_empty() {
                    continue;
                }

                // add headers and subitems
                let tag_header = egui::CollapsingHeader::new(tag).show(ui, |ui| {
                    // add records
                    // sort

                    for recordt in records_of_tag.iter() {
                        let record = *recordt;
                        let id = get_unique_id(record);

                        // annotations
                        let mut label = record.editor_id().to_string();
                        // hack for header record
                        if label.is_empty() && record.tag_str() == "TES3" {
                            label = "Header".into();
                        }
                        // modified records
                        if self.edited_records.contains_key(&id) {
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
                                if !records_to_delete.contains(&id) {
                                    records_to_delete.push(id.clone());
                                }
                                ui.close_menu();
                            }
                        });

                        // selected event
                        if response.clicked() {
                            // cleanup old records
                            let mut to_remove = Vec::new();
                            for (key, edited_record) in self.edited_records.clone() {
                                if let Some(original) = self.records.get(&key) {
                                    // remove if no change
                                    if original.eq(&edited_record) && id != key {
                                        to_remove.push(key);
                                    }
                                }
                            }
                            for r in to_remove {
                                self.edited_records.remove(&r);
                            }

                            // add a copy of this record to the edited records
                            if !self.edited_records.contains_key(&id) {
                                self.edited_records.insert(id.clone(), record.clone());
                            }

                            self.current_record_id = Some(id);
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
                            if !records_to_delete.contains(&id) {
                                records_to_delete.push(id.clone());
                            }
                        }
                        ui.close_menu();
                    }
                });
            }

            ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
        });

        // delete stuff
        for k in records_to_delete {
            self.records.remove(&k);
        }
    }
}

fn Create(tag: &str) -> TES3Object {
    todo!()
}
