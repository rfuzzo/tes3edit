use crate::{create, create_from_tag, get_all_tags, get_unique_id, ERecordType, TemplateApp};

impl TemplateApp {
    pub fn records_list_view(&mut self, ui: &mut egui::Ui) {
        let mut regen = false;

        // heading
        if let Some(data) = self.plugins.iter().find(|p| p.id == self.current_plugin_id) {
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
                    .plugins
                    .iter_mut()
                    .find(|p| p.id == self.current_plugin_id) else { return };

        // search bar
        let _search_text = self.search_text.clone();
        ui.horizontal(|ui| {
            ui.label("Filter: ");
            ui.text_edit_singleline(&mut self.search_text);
        });
        ui.separator();

        // add record button
        ui.horizontal(|ui| {
            let mut record_type = ERecordType::MISC;
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", record_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut record_type, ERecordType::ACTI, "ACTI");
                    ui.selectable_value(&mut record_type, ERecordType::ALCH, "ALCH");
                    ui.selectable_value(&mut record_type, ERecordType::APPA, "APPA");
                });

            if ui.button("Add record").clicked() {
                if let Some(instance) = create(record_type) {
                    data.edited_records
                        .insert(get_unique_id(&instance), instance);
                    data.cached_ids.clear();
                }
            }
        });

        // regenerate records
        if (_search_text != self.search_text) || data.cached_ids.is_empty() {
            data.regenerate_id_cache(&self.search_text);
        }

        // the record list
        let mut record_ids_to_delete = vec![];
        egui::ScrollArea::vertical().show(ui, |ui| {
            // order by tags
            for tag in tags {
                let ids_by_tag = data
                    .cached_ids
                    .iter()
                    .filter(|p| p.split(',').collect::<Vec<_>>().first().unwrap() == &tag)
                    .map(|e| e.to_owned())
                    .collect::<Vec<_>>();
                if ids_by_tag.is_empty() {
                    continue;
                }

                // add headers and subitems
                let tag_header = egui::CollapsingHeader::new(tag.clone()).show(ui, |ui| {
                    // add records
                    // sort

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
                        }

                        // check here if any id has changed
                        // TODO
                        if !data.get_record_ids().contains(&id) {
                            regen = true;
                        }
                    }
                });

                // context menu of tag header
                tag_header.header_response.context_menu(|ui| {
                    // todo add record button
                    if ui.button("Add record").clicked() {
                        if let Some(instance) = create_from_tag(&tag.clone()) {
                            data.edited_records
                                .insert(get_unique_id(&instance), instance);
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

            ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
        });

        // delete stuff
        record_ids_to_delete.dedup();
        for k in record_ids_to_delete {
            data.records.remove(&k);
        }

        // clear cache
        if regen {
            data.cached_ids.clear();
        }
    }
}
