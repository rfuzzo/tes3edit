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
            ui.label(format!("Height: {}", self.map_data.dbg_data));
        });

        ui.separator();

        // //draw rows painter
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );

        // cache shapes
        let s = self.map_data.texture_handle.is_none();
        if s || self.map_data.refresh_requested {
            let (to_screen, _) = get_transforms(&self.map_data, &painter);
            if self.map_data.refresh_requested {
                self.map_data.refresh_requested = false;
            }

            crate::generate_map(&mut self.map_data, to_screen, ui);
        }

        // hover
        if let Some(hover_pos) = painter.ctx().pointer_hover_pos() {
            let (_, from_screen) = get_transforms(&self.map_data, &painter);
            let real_pos = from_screen * hover_pos;

            let mut x = real_pos.x;
            let mut y = real_pos.y;
            // hacks to get the correct cell name
            if x >= 0.0 {
                x += 1.0;
            }
            // else {
            //     x -= 1.0;
            // }
            if y >= 0.0 {
                y += 1.0;
            }
            // else {
            //     y -= 1.0;
            // }
            self.map_data.hover_pos = (x as i32, y as i32);
        }

        // TODO selected overlay
        // if let Some(grid) = map_data.cell_ids.get(&map_data.selected_id) {
        //     if grid == &key {
        //         color = Color32::RED;
        //     }

        //     // dbg
        //     map_data.dbg_data = heightmap[hx][hy].to_string();
        // }

        paint(&painter, &self.map_data);
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
}

#[cfg(not(target_arch = "wasm32"))]
pub fn paint(painter: &egui::Painter, map_data: &MapData) {
    // if let Some(shapes) = map_data.shapes.clone() {
    //     painter.extend(shapes);
    // }

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
    let min = Pos2::new(data.bounds_x.0 as f32, data.bounds_y.1 as f32);
    let max = Pos2::new(data.bounds_x.1 as f32, data.bounds_y.0 as f32);

    let world = Rect::from_min_max(min, max);
    let canvas = painter.clip_rect();

    let to_screen = emath::RectTransform::from_to(world, canvas);
    let from_screen = emath::RectTransform::from_to(canvas, world);
    (to_screen, from_screen)
}
