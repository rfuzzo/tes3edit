use serde::{Deserialize, Serialize};

/// Generic text editor
fn add_yaml_editor<T>(ui: &mut egui::Ui, object: &mut T)
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    let mut text = serde_yaml::to_string(&object).unwrap();
    if ui.text_edit_multiline(&mut text).changed() {
        let result: Result<T, _> = serde_yaml::from_str(&text);
        if let Ok(out) = result {
            *object = out;
        }
    }
}
