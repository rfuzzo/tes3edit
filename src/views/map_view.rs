use egui::emath;
use egui::emath::RectTransform;
use egui::Painter;
use egui::Pos2;
use egui::Rect;

use crate::MapData;
use crate::TemplateApp;

impl TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn map_view(&mut self, ui: &mut egui::Ui) {
        // headers
        ui.heading("Map");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!(
                "[{},{}] - [{},{}]",
                self.map_data.bounds_x.0,
                self.map_data.bounds_y.0,
                self.map_data.bounds_x.1,
                self.map_data.bounds_y.1
            ));
            ui.separator();
            ui.label(format!("Selected Cell: {}", self.map_data.selected_id));
            ui.label(format!(
                "Hover Position: {}, {}",
                self.map_data.hover_pos.0, self.map_data.hover_pos.1
            ));
            // get cell name
            if let Some(cell) = self.map_data.cells.get(&self.map_data.hover_pos) {
                let mut name = cell.name.clone();
                if name.is_empty() {
                    if let Some(region) = cell.region.clone() {
                        name = region;
                    }
                }
                ui.label(format!("Cell: {}", name));
            }
        });

        ui.separator();

        // draw rows painter
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );

        // cache shapes
        if self.map_data.texture_handle.is_none() {
            let (to_screen, _) = get_transforms(&self.map_data, &painter);
            crate::generate_map(&mut self.map_data, to_screen, ui);
        }

        // hover
        if let Some(hover_pos) = painter.ctx().pointer_hover_pos() {
            let (_, from_screen) = get_transforms(&self.map_data, &painter);
            let world_pos = from_screen * hover_pos;

            let x = world_pos.x as i32 + self.map_data.bounds_x.0;
            let y = -(world_pos.y as i32 - self.map_data.bounds_y.1);
            self.map_data.hover_pos = (x, y);
        }

        // Make sure we allocate what we used (everything)
        paint(&painter, &self.map_data);
        ui.expand_to_include_rect(painter.clip_rect());

        // settings
        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                egui::CollapsingHeader::new("Settings").show(ui, |ui| self.options_ui(ui));
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn paint(painter: &egui::Painter, map_data: &MapData) {
    use egui::{pos2, Color32};
    if let Some(texture_handle) = map_data.texture_handle.clone() {
        painter.image(
            texture_handle.id(),
            painter.clip_rect(),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        )
    }
}

fn get_transforms(data: &MapData, painter: &Painter) -> (RectTransform, RectTransform) {
    let height =
        (data.bounds_y.0.unsigned_abs() as usize + data.bounds_y.1.unsigned_abs() as usize) + 1;
    let width =
        (data.bounds_x.0.unsigned_abs() as usize + data.bounds_x.1.unsigned_abs() as usize) + 1;

    let min = Pos2::new(0.0, 0.0);
    let max = Pos2::new(width as f32, height as f32);

    let world = Rect::from_min_max(min, max);
    let canvas = painter.clip_rect();

    let to_screen = emath::RectTransform::from_to(world, canvas);
    let from_screen = emath::RectTransform::from_to(canvas, world);
    (to_screen, from_screen)
}
