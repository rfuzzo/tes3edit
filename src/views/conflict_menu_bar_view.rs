use crate::{EAppState, TemplateApp};

impl TemplateApp {
    pub fn conflict_menu_bar_view(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Menu Bar
        egui::menu::bar(ui, |ui| {
            if ui.button("Exit").clicked() {
                self.app_state = EAppState::Main;
            }
        });
    }
}
