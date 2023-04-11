use std::collections::HashMap;

use tes3::esp::{EditorId, TES3Object, TypeInfo};

use crate::get_unique_id;

/// View that holds a tree-list of all records in the current plugin
pub(crate) fn records_list_view(
    // ui
    ui: &mut egui::Ui,
    // in
    records: &mut HashMap<String, TES3Object>,
    // out
    edited_records: &mut HashMap<String, TES3Object>,
    current_record_id: &mut Option<String>,
    search_text: &mut String,
) {
    // group by tag
    let mut tags: Vec<&str> = records.values().map(|e| e.tag_str()).collect();
    // todo proper sorting
    tags.sort();
    tags.dedup();

    // search bar
    ui.horizontal(|ui| {
        ui.label("Filter: ");
        ui.text_edit_singleline(search_text);
    });

    ui.separator();

    // the record list
    egui::ScrollArea::vertical().show(ui, |ui| {
        // order by tags
        for tag in tags {
            let mut records_list: Vec<&TES3Object> =
                records.values().filter(|r| r.tag_str() == tag).collect();

            // search filter
            if !search_text.is_empty() {
                records_list = records_list
                    .iter()
                    .copied()
                    .filter(|p| {
                        get_unique_id(p)
                            .to_lowercase()
                            .contains(search_text.as_str())
                    })
                    .collect();
            }
            if records_list.is_empty() {
                continue;
            }

            // add headers and tree
            egui::CollapsingHeader::new(tag).show(ui, |ui| {
                // add records
                // sort
                records_list.sort_by(|a, b| a.editor_id().cmp(&b.editor_id()));
                for record in records_list {
                    let id = get_unique_id(record);
                    let is_modified = edited_records.contains_key(&id); // if modified, annotate it
                    let mut label = record.editor_id().to_string();
                    // hack for header record
                    if label.is_empty() && record.tag_str() == "TES3" {
                        label = "Header".into();
                    }

                    if is_modified {
                        label = format!("{}*", label);
                        ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                    } else {
                        ui.visuals_mut().override_text_color = None;
                    }

                    if ui
                        .add(egui::Label::new(label).sense(egui::Sense::click()))
                        .clicked()
                    {
                        // selected event

                        // cleanup old records
                        let mut to_remove = Vec::new();
                        for (key, edited_record) in edited_records.clone() {
                            if let Some(original) = records.get(&key) {
                                // remove if no change
                                if original.eq(&edited_record) && id != key {
                                    to_remove.push(key);
                                }
                            }
                        }
                        for r in to_remove {
                            edited_records.remove(&r);
                        }

                        // add a copy of this record to the edited records
                        if !edited_records.contains_key(&id) {
                            edited_records.insert(id.clone(), record.clone());
                        }

                        *current_record_id = Some(id);
                    }
                }
            });
        }
        ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
    });
}
