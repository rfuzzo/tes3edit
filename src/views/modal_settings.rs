use crate::TemplateApp;

impl TemplateApp {
    pub fn update_settings(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings")
            .open(&mut self.modal_open)
            .show(ctx, |ui| {
                ui.heading("Settings");

                ui.separator();

                ui.checkbox(&mut self.overwrite, "Overwrite on plugin save");
                ui.checkbox(&mut self.use_experimental, "Show experimental features");
            });
    }
}
