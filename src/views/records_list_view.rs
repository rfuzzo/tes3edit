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
    current_text: &mut (String, String),
    current_record: &mut Option<TES3Object>,
) {
    // group by tag
    let mut tags: Vec<&str> = records.values().map(|e| e.tag_str()).collect();
    tags.sort();
    tags.dedup();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for tag in tags {
            let records: Vec<_> = records.values().filter(|r| r.tag_str() == tag).collect();
            // add headers and tree
            egui::CollapsingHeader::new(tag).show(ui, |ui| {
                // add records
                for record in records {
                    let id = get_unique_id(record);
                    // if modified, annotate it
                    let is_modified = edited_records.contains_key(&id);
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
                        // on clicked event for records
                        // deserialize the original record or the edited
                        if edited_records.contains_key(&id) {
                            *current_text = (
                                id.clone(),
                                serde_yaml::to_string(&edited_records[&id])
                                    .unwrap_or("Error serializing".to_owned()),
                            );
                            *current_record = Some(edited_records[&id].clone());
                        } else {
                            *current_text = (
                                id,
                                serde_yaml::to_string(&record)
                                    .unwrap_or("Error serializing".to_owned()),
                            );
                            *current_record = Some(record.clone());
                        }
                    }
                }
            });
        }
        ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
    });
}
