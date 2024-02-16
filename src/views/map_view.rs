use egui::{pos2, Color32, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use tes3::esp::TES3Object;

use crate::{get_cell_name, TemplateApp, GRID};

impl TemplateApp {
    pub fn map_view(&mut self, ui: &mut egui::Ui) {
        use crate::get_unique_id;

        ////////////////////////////////////////////////////////////////////////////////////////////////////
        // painter
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        // zoom
        if let Some(delta) = self.zoom_data.drag_delta.take() {
            self.zoom_data.drag_offset += delta.to_vec2();
        }
        if let Some(z) = self.zoom_data.zoom_delta.take() {
            let r = z - 1.0;
            let mut current_zoom = self.zoom_data.zoom;
            current_zoom += r;
            if current_zoom > 0.0 {
                self.zoom_data.zoom = current_zoom;

                // TODO offset the image for smooth zoom
                if let Some(pointer_pos) = response.hover_pos() {
                    let d = pointer_pos * r;
                    self.zoom_data.drag_offset -= d.to_vec2();
                }
            }
        }

        // zoomed and panned canvas
        let min = self.zoom_data.drag_offset;
        let max = response.rect.max * self.zoom_data.zoom + self.zoom_data.drag_offset.to_vec2();
        let canvas = Rect::from_min_max(min, max);

        // transforms
        let pixel_width = self.map_data.width() as f32 / GRID as f32;
        let pixel_height = self.map_data.height() as f32 / GRID as f32;
        let to = canvas;
        let from: Rect = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(pixel_width, pixel_height));

        let to_screen = egui::emath::RectTransform::from_to(from, to);
        let from_screen = to_screen.inverse();

        // paint maps
        let uv = Rect::from_min_max(pos2(0.0, 0.0), Pos2::new(1.0, 1.0));

        ////////////////////////////////////////////////////////////////////////////////////////////////////
        // cache shapes
        if self.map_data.texture_handle.is_none() {
            crate::generate_map(&mut self.map_data, ui);
        }

        if let Some(texture_handle) = &self.map_data.texture_handle.clone() {
            painter.image(texture_handle.id(), canvas, uv, Color32::WHITE);
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////
        // Responses

        // hover
        if let Some(hover_pos) = painter.ctx().pointer_hover_pos() {
            let pos = self.map_data.abs_to_world_pos(from_screen * hover_pos);
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
                                for (_class, destination_keys) in self.map_data.edges.iter() {
                                    for (p1, p2) in destination_keys {
                                        if &self.map_data.hover_pos == p1 {
                                            destinations.push(get_cell_name(&self.map_data, *p2));
                                        }
                                        if &self.map_data.hover_pos == p2 {
                                            destinations.push(get_cell_name(&self.map_data, *p1));
                                        }
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
                let pos = self.map_data.abs_to_world_pos(from_screen * interact_pos);
                if let Some(cell) = self.map_data.cells.get(&pos) {
                    let c = TES3Object::from(cell.clone());
                    let id = get_unique_id(&c);
                    self.map_data.selected_id = id;
                }
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////
        // draw overlays

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

                                let p00 = self.map_data.world_to_abs_pos(key);
                                let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

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
        for x in self.map_data.bounds_x.0..self.map_data.bounds_x.1 {
            for y in self.map_data.bounds_y.0..self.map_data.bounds_y.1 {
                // get region
                let key = (x, y);
                if let Some(cell) = self.map_data.cells.get(&key) {
                    if let Some(map_color) = cell.map_color {
                        let color = Color32::from_rgb(map_color[0], map_color[1], map_color[2]);

                        let p00 = self.map_data.world_to_abs_pos(key);
                        let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

                        let rect = Rect::from_min_max(to_screen * p00, to_screen * p11);
                        let shape =
                            Shape::rect_stroke(rect, Rounding::default(), Stroke::new(2.0, color));
                        city_shapes.push(shape);
                    }
                }
            }
        }
        painter.extend(city_shapes.clone());

        // travel
        if self.map_data.overlay_travel {
            for (class, destinations) in self.map_data.edges.iter() {
                // get class color
                let color = get_color_for_class(class);
                let mut travel_shapes: Vec<Shape> = vec![];
                for (key, value) in destinations {
                    let p00 = self.map_data.world_to_abs_pos(*key) + Vec2::new(0.5, 0.5);
                    let p11 = self.map_data.world_to_abs_pos(*value) + Vec2::new(0.5, 0.5);

                    let line = Shape::LineSegment {
                        points: [to_screen * p00, to_screen * p11],
                        stroke: Stroke::new(2.0, color),
                    };
                    travel_shapes.push(line);
                }
                painter.extend(travel_shapes.clone());
            }
        }

        // conflicts
        if self.map_data.overlay_conflicts {
            for (cx, cy) in self.map_data.cell_conflicts.keys() {
                let p00 = self.map_data.world_to_abs_pos((*cx, *cy));
                let p11 = Pos2::new(p00.x + 1.0, p00.y + 1.0);

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
            let p00 = self.map_data.world_to_abs_pos((*cx, *cy));
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

        ////////////////////////////////////////////////////////////////////////////////////////////////////
        // zoom and pan
        // panning
        if response.drag_started() {
            if let Some(drag_start) = response.interact_pointer_pos() {
                self.zoom_data.drag_start = drag_start;
            }
        } else if response.dragged() {
            if let Some(current_pos) = response.interact_pointer_pos() {
                let delta = current_pos - self.zoom_data.drag_start.to_vec2();
                self.zoom_data.drag_delta = Some(delta);
                self.zoom_data.drag_start = current_pos;
            }
        }

        // zoom
        let delta = ui.ctx().input(|i| i.zoom_delta());
        // let delta = response.input(|i| i.zoom_delta());
        if delta != 1.0 {
            self.zoom_data.zoom_delta = Some(delta);
        }
        if response.middle_clicked() {
            self.reset_zoom();
            self.reset_pan();
        }

        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        ////////////////////////////////////////////////////////////////////////////////////////////////////
        // settings
        // dumb ui hack
        let settings_rect = egui::Rect::from_min_max(response.rect.min, pos2(0.0, 0.0));
        ui.put(settings_rect, egui::Label::new(""));
        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                egui::CollapsingHeader::new("Settings").show(ui, |ui| self.options_ui(ui));
            });
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_data.zoom = 1.0;
    }

    pub fn reset_pan(&mut self) {
        self.zoom_data.drag_delta = None;
        self.zoom_data.drag_offset = Pos2::default();
        self.zoom_data.drag_start = Pos2::default();
    }
}

fn get_color_for_class(class: &str) -> Color32 {
    match class {
        "Shipmaster" => Color32::BLUE,
        "Caravaner" => Color32::GOLD,
        "Gondolier" => Color32::GRAY,
        "T_Mw_RiverstriderService" => Color32::LIGHT_BLUE,
        _ => Color32::RED,
    }
}
