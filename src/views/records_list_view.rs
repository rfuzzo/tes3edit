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
                        *current_record_id = Some(id);
                    }
                }
            });
        }
        ui.allocate_space(ui.available_size()); // put this LAST in your panel/window code
    });
}
