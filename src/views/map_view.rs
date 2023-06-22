use egui::{
    emath::{self, RectTransform},
    pos2, Color32, Painter, Pos2, Rect, Shape, Stroke,
};
use tes3::esp::TES3Object;

use crate::{MapData, TemplateApp};

impl TemplateApp {
    pub fn map_view(&mut self, ui: &mut egui::Ui) {
        // headers
        use crate::get_unique_id;
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
            crate::generate_map(&mut self.map_data, ui);
        }

        // hover
        if let Some(hover_pos) = painter.ctx().pointer_hover_pos() {
            let (_, from_screen) = get_transforms(&self.map_data, &painter);
            self.map_data.hover_pos = self.abs_to_world_pos(from_screen * hover_pos);
        }
        // click
        if let Some(interact_pos) = painter.ctx().pointer_interact_pos() {
            if ui.ctx().input(|i| i.pointer.any_click()) {
                let (_, from_screen) = get_transforms(&self.map_data, &painter);
                let pos = self.abs_to_world_pos(from_screen * interact_pos);
                if let Some(cell) = self.map_data.cells.get(&pos) {
                    let c = TES3Object::from(cell.clone());
                    let id = get_unique_id(&c);
                    self.map_data.selected_id = id;
                }
            }
        }

        paint(&painter, &self.map_data);

        // draw overlay
        if let Some((cx, cy)) = self.map_data.cell_ids.get(&self.map_data.selected_id) {
            let (to_screen, _) = get_transforms(&self.map_data, &painter);

            let p00 = self.world_to_abs_pos((*cx, *cy));
            let p01 = Pos2::new(p00.x + 1.0, p00.y);
            let p10 = Pos2::new(p00.x, p00.y + 1.0);
            let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

            let line1 = Shape::line_segment(
                [to_screen * p00, to_screen * p11],
                Stroke::new(2.0, Color32::RED),
            );
            let line2 = Shape::line_segment(
                [to_screen * p10, to_screen * p01],
                Stroke::new(2.0, Color32::RED),
            );

            painter.add(line1);
            painter.add(line2);
        }

        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        // settings
        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                egui::CollapsingHeader::new("Settings").show(ui, |ui| self.options_ui(ui));
            });
    }

    fn abs_to_world_pos(&mut self, abs_pos: Pos2) -> (i32, i32) {
        let x = abs_pos.x as i32 + self.map_data.bounds_x.0;
        let y = -(abs_pos.y as i32 - self.map_data.bounds_y.1);
        (x, y)
    }

    fn world_to_abs_pos(&mut self, world_pos: (i32, i32)) -> Pos2 {
        let x = world_pos.0 - self.map_data.bounds_x.0;
        let y = -(world_pos.1 - self.map_data.bounds_y.1);
        Pos2::new(x as f32, y as f32)
    }
}

pub fn paint(painter: &egui::Painter, map_data: &MapData) {
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
