use strum::IntoEnumIterator;
use tes3::esp::TypeInfo;

use crate::{create, create_from_tag, get_all_tags, get_unique_id, ERecordType, TemplateApp};

impl TemplateApp {
    pub fn records_list_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // heading
        if let Some(data) = self
            .edit_data
            .plugins
            .iter()
            .find(|p| p.id == self.edit_data.current_plugin_id)
        {
            if let Some(path) = &data.full_path {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                ui.heading(name);
            } else {
                ui.heading("New plugin");
            }
        } else {
            ui.heading("Records");
        }

        // editor for a specific plugin
        let tags = get_all_tags();
        let Some(data) = self
            .edit_data
            .plugins
            .iter_mut()
            .find(|p| p.id == self.edit_data.current_plugin_id)
        else {
            return;
        };

        // search bar
        let search_text = self.edit_data.search_text.clone();
        ui.horizontal(|ui| {
            ui.label("Filter: ");
            ui.text_edit_singleline(&mut self.edit_data.search_text);
        });
        ui.separator();

        // add record button
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.edit_data.record_type))
                .show_ui(ui, |ui| {
                    let tags = ERecordType::iter().collect::<Vec<_>>();
                    for t in tags {
                        // not allowed to create a header manually
                        if t == ERecordType::TES3 {
                            continue;
                        }
                        ui.selectable_value(&mut self.edit_data.record_type, t, t.to_string());
                    }
                });

            if ui.button("Add record").clicked() {
                if let Some(instance) = create(self.edit_data.record_type) {
                    let new_id = get_unique_id(&instance);
                    data.edited_records.insert(new_id.clone(), instance);
                    data.clear_cache();
                    data.selected_record_id = Some(new_id);
                }
            }
        });

        // regenerate records
        if (search_text != self.edit_data.search_text) || data.cached_ids.is_empty() {
            data.regenerate_id_cache(&self.edit_data.search_text);
        }

        let mut clicked = false;

        // the record list
        let mut record_ids_to_delete = vec![];
        egui::ScrollArea::vertical().show(ui, |ui| {
            // order by tags
            for tag in tags {
                let ids_by_tag = data.cached_ids[&tag].clone();
                if ids_by_tag.is_empty() {
                    continue;
                }

                // add headers and subitems
                let tag_header = egui::CollapsingHeader::new(tag.clone()).show(ui, |ui| {
                    for id in ids_by_tag.iter() {
                        // annotations
                        let mut label = id.to_string();
                        // hack for header record
                        if id.starts_with("TES3,") {
                            label = "Header".into();
                        }
                        // modified records
                        if data.edited_records.contains_key(id) {
                            label = format!("{}*", label);
                            ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                        } else {
                            ui.visuals_mut().override_text_color = None;
                        }

                        // record list item view
                        let response = ui.add(egui::Label::new(label).sense(egui::Sense::click()));
                        // context menu
                        response.clone().context_menu(|ui| {
                            // copy record
                            if ui.button("Copy").clicked() {
                                self.edit_data.copied_record =
                                    if data.edited_records.contains_key(id) {
                                        Some(data.edited_records.get_mut(id).unwrap().clone())
                                    } else if data.records.contains_key(id) {
                                        Some(data.records.get_mut(id).unwrap().clone())
                                    } else {
                                        None
                                    };
                                self.toasts.success("Copied record");
                                ui.close_menu();
                            }

                            // paste record
                            if self.edit_data.copied_record.is_some()
                                && ui.button("Paste").clicked()
                            {
                                let copy = self.edit_data.copied_record.clone().unwrap();

                                if let Some(current_record) =
                                    if data.edited_records.contains_key(id) {
                                        Some(data.edited_records.get_mut(id).unwrap())
                                    } else if data.records.contains_key(id) {
                                        Some(data.records.get_mut(id).unwrap())
                                    } else {
                                        None
                                    }
                                {
                                    if copy.tag_str() == current_record.tag_str() {
                                        // then we can copy paste
                                        *current_record = copy;
                                    }
                                }

                                ui.close_menu();
                            }

                            ui.separator();

                            // copy id
                            if ui.button("Copy ID").clicked() {
                                ctx.copy_text(id.clone().get(5..).unwrap().to_string());
                                ui.close_menu();
                            }

                            ui.separator();

                            // delete a record
                            if ui.button("Delete").clicked() {
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
                            if !data.edited_records.contains_key(id) {
                                // get record
                                let mut record_or_none = data.records.get(id);
                                if data.edited_records.contains_key(id) {
                                    record_or_none = data.edited_records.get(id);
                                }
                                if let Some(record) = record_or_none {
                                    data.edited_records.insert(id.clone(), record.clone());
                                }
                            }

                            data.selected_record_id = Some(id.to_string());

                            clicked = true;
                        }
                    }
                });

                // context menu of tag header
                if tag != "TES3" {
                    tag_header.header_response.context_menu(|ui| {
                        // add record button
                        if ui.button("Add record").clicked() {
                            if let Some(instance) = create_from_tag(&tag.clone()) {
                                let new_id = get_unique_id(&instance);
                                data.edited_records.insert(new_id.clone(), instance);
                                data.selected_record_id = Some(new_id);
                            } else {
                                self.toasts.warning("Could not create record");
                            }

                            ui.close_menu();
                        }

                        ui.separator();

                        // delete all button
                        if ui.button("Delete all").clicked() {
                            for id in ids_by_tag.iter() {
                                record_ids_to_delete.push(id.clone());
                            }
                            ui.close_menu();
                        }
                    });
                }
            }

            ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
        });

        // delete stuff
        // TODO deleting actually removes, not undoable
        record_ids_to_delete.dedup();
        if !record_ids_to_delete.is_empty() {
            for k in record_ids_to_delete {
                if data.records.contains_key(&k) {
                    data.records.remove(&k);
                }
                if data.edited_records.contains_key(&k) {
                    data.edited_records.remove(&k);
                }
            }
            data.clear_cache();
        }

        // fix ids
        if clicked {
            let mut updates = vec![];
            for (key, v) in data.edited_records.clone() {
                let new_key = get_unique_id(&v);
                if new_key != *key {
                    updates.push((key, new_key));
                }
            }
            if !updates.is_empty() {
                for (old_key, new_key) in updates {
                    // update current_plugin_id
                    if data.selected_record_id == Some(old_key.clone()) {
                        data.selected_record_id = Some(new_key.clone());
                    }
                    // update HashMap
                    if let Some(v) = data.edited_records.remove(&old_key) {
                        data.edited_records.insert(new_key, v);
                        data.records.remove(&old_key);
                    }
                }
                data.clear_cache();
            }
        }
    }
}
