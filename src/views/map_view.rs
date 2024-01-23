use egui::{
    emath::{self, RectTransform},
    pos2, Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2,
};
use tes3::esp::TES3Object;

use crate::{get_cell_name, MapData, TemplateApp};

impl TemplateApp {
    pub fn map_view(&mut self, ui: &mut egui::Ui) {
        // headers
        use crate::get_unique_id;
        ui.heading("Map");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Selected Cell: {}", self.map_data.selected_id));
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
            let pos = abs_to_world_pos(&self.map_data, from_screen * hover_pos);
            self.map_data.hover_pos = pos;
            // tooltip
            if self.map_data.overlay_conflicts || self.map_data.tooltip_names {
                // conflicts list
                if self.map_data.overlay_conflicts {
                    if let Some(conflicts) = self.map_data.cell_conflicts.get(&pos) {
                        egui::show_tooltip(ui.ctx(), egui::Id::new("my_tooltip"), |ui| {
                            ui.label(get_cell_name(&self.map_data, self.map_data.hover_pos));
                            ui.separator();
                            ui.label("Conflicts:");
                            for hash in conflicts {
                                if let Some(name) = self.map_data.plugin_hashes.get(hash) {
                                    ui.label(format!("- {}", name));
                                }
                            }
                        });
                    } else if self.map_data.tooltip_names {
                        let name = get_cell_name(&self.map_data, self.map_data.hover_pos);
                        if !name.is_empty() {
                            egui::show_tooltip(ui.ctx(), egui::Id::new("my_tooltip"), |ui| {
                                ui.label(name);
                            });
                        }
                    }
                }
                // cell name
                else if self.map_data.tooltip_names {
                    let name = get_cell_name(&self.map_data, self.map_data.hover_pos);
                    if !name.is_empty() {
                        if self.map_data.overlay_travel {
                            egui::show_tooltip(ui.ctx(), egui::Id::new("my_tooltip"), |ui| {
                                ui.label(name);

                                // travel destinations
                                let mut destinations: Vec<String> = vec![];
                                for (p1, p2) in self.map_data.edges.iter() {
                                    if &self.map_data.hover_pos == p1 {
                                        destinations.push(get_cell_name(&self.map_data, *p2));
                                    }
                                    if &self.map_data.hover_pos == p2 {
                                        destinations.push(get_cell_name(&self.map_data, *p1));
                                    }
                                }
                                if !destinations.is_empty() {
                                    ui.separator();
                                }
                                for d in destinations {
                                    ui.label(d);
                                }
                            });
                        } else {
                            egui::show_tooltip(ui.ctx(), egui::Id::new("my_tooltip"), |ui| {
                                ui.label(name);
                            });
                        }
                    }
                }
            }
        }
        // click
        if let Some(interact_pos) = painter.ctx().pointer_interact_pos() {
            if ui.ctx().input(|i| i.pointer.primary_clicked()) {
                let (_, from_screen) = get_transforms(&self.map_data, &painter);
                let pos = abs_to_world_pos(&self.map_data, from_screen * interact_pos);
                if let Some(cell) = self.map_data.cells.get(&pos) {
                    let c = TES3Object::from(cell.clone());
                    let id = get_unique_id(&c);
                    self.map_data.selected_id = id;
                }
            }
        }

        paint(&painter, &self.map_data);

        // draw overlays

        if self.map_data.overlay_travel {
            let mut travel_shapes: Vec<Shape> = vec![];

            for (key, value) in self.map_data.edges.iter() {
                let p00 = world_to_abs_pos(&self.map_data, *key) + Vec2::new(0.5, 0.5);
                let p11 = world_to_abs_pos(&self.map_data, *value) + Vec2::new(0.5, 0.5);

                let (to_screen, _) = get_transforms(&self.map_data, &painter);
                let line = Shape::LineSegment {
                    points: [to_screen * p00, to_screen * p11],
                    stroke: Stroke::new(2.0, Color32::RED),
                };
                travel_shapes.push(line);
            }

            painter.extend(travel_shapes.clone());
        }

        // regions
        if self.map_data.overlay_region {
            let mut region_shapes: Vec<Shape> = vec![];

            // if self.map_data.region_shapes.is_empty() {
            for x in self.map_data.bounds_x.0..self.map_data.bounds_x.1 {
                for y in self.map_data.bounds_y.0..self.map_data.bounds_y.1 {
                    // get region
                    let key = (x, y);
                    if let Some(cell) = self.map_data.cells.get(&key) {
                        if let Some(region_name) = cell.region.clone() {
                            if let Some(region) = self.map_data.regions.get(&region_name) {
                                let region_color = Color32::from_rgb(
                                    region.map_color[0],
                                    region.map_color[1],
                                    region.map_color[2],
                                );

                                let p00 = world_to_abs_pos(&self.map_data, key);
                                let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

                                let (to_screen, _) = get_transforms(&self.map_data, &painter);
                                let rect = Rect::from_min_max(to_screen * p00, to_screen * p11);
                                let shape =
                                    Shape::rect_filled(rect, Rounding::default(), region_color);
                                region_shapes.push(shape);
                            }
                        }
                    }
                }
            }
            //}

            painter.extend(region_shapes.clone());
        }
        // cities
        let mut city_shapes: Vec<Shape> = vec![];
        // if self.map_data.city_shapes.is_empty() {
        for x in self.map_data.bounds_x.0..self.map_data.bounds_x.1 {
            for y in self.map_data.bounds_y.0..self.map_data.bounds_y.1 {
                // get region
                let key = (x, y);
                if let Some(cell) = self.map_data.cells.get(&key) {
                    if let Some(map_color) = cell.map_color {
                        let color = Color32::from_rgb(map_color[0], map_color[1], map_color[2]);

                        let p00 = world_to_abs_pos(&self.map_data, key);
                        // let p01 = Pos2::new(p00.x + 1.0, p00.y);
                        // let p10 = Pos2::new(p00.x, p00.y + 1.0);
                        let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

                        let (to_screen, _) = get_transforms(&self.map_data, &painter);
                        let rect = Rect::from_min_max(to_screen * p00, to_screen * p11);
                        let shape =
                            Shape::rect_stroke(rect, Rounding::default(), Stroke::new(2.0, color));
                        city_shapes.push(shape);
                    }
                }
            }
        }
        // }
        painter.extend(city_shapes.clone());

        // conflicts
        if self.map_data.overlay_conflicts {
            for (cx, cy) in self.map_data.cell_conflicts.keys() {
                let p00 = world_to_abs_pos(&self.map_data, (*cx, *cy));
                let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

                let (to_screen, _) = get_transforms(&self.map_data, &painter);
                let rect = Rect::from_min_max(to_screen * p00, to_screen * p11);
                let shape = Shape::rect_filled(
                    rect,
                    Rounding::default(),
                    Color32::from_rgba_unmultiplied(0, 255, 0, 10),
                );
                painter.add(shape);
            }
        }
        // selected
        if let Some((cx, cy)) = self.map_data.cell_ids.get(&self.map_data.selected_id) {
            let p00 = world_to_abs_pos(&self.map_data, (*cx, *cy));
            let p01 = Pos2::new(p00.x + 1.0, p00.y);
            let p10 = Pos2::new(p00.x, p00.y + 1.0);
            let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

            let (to_screen, _) = get_transforms(&self.map_data, &painter);
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

fn abs_to_world_pos(map_data: &MapData, abs_pos: Pos2) -> (i32, i32) {
    let x = abs_pos.x as i32 + map_data.bounds_x.0;
    let y = -(abs_pos.y as i32 - map_data.bounds_y.1);
    (x, y)
}

fn world_to_abs_pos(map_data: &MapData, world_pos: (i32, i32)) -> Pos2 {
    let x = world_pos.0 - map_data.bounds_x.0;
    let y = -(world_pos.1 - map_data.bounds_y.1);
    Pos2::new(x as f32, y as f32)
}
