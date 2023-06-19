use crate::MapData;
use crate::TemplateApp;

impl TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn map_view(&mut self, ui: &mut egui::Ui) {
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
        });

        ui.separator();

        // painter
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        paint(&painter, &mut self.map_data);
        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());
    }
}

///
#[cfg(not(target_arch = "wasm32"))]
pub fn paint(painter: &egui::Painter, map_data: &mut MapData) {
    use egui::{emath, Color32, Pos2, Rect, Shape};

    let bounds = std::cmp::max(map_data.min.abs(), map_data.max.abs());
    let boundsf = bounds as f32;

    let rect = painter.clip_rect();
    let to_screen = emath::RectTransform::from_to(
        Rect::from_min_max(Pos2::new(-boundsf, -boundsf), Pos2::new(boundsf, boundsf)),
        rect,
    );
    let from_screen = emath::RectTransform::from_to(
        rect,
        Rect::from_min_max(Pos2::new(-boundsf, -boundsf), Pos2::new(boundsf, boundsf)),
    );

    //draw rows
    let mut shapes: Vec<Shape> = Vec::new();
    for x in -bounds..bounds {
        for y in -bounds..bounds {
            let key = (x, -y); // draw upside down

            // get LAND record
            if let Some(land) = map_data.landscape.get(&key) {
                let heightmap = land.world_map_data.data.clone().to_vec();
                // draw 9x9 grid
                (0..9).for_each(|hx| {
                    for hy in 0..9 {
                        let mut color: Color32;
                        let h = heightmap[hx][hy] as i32;
                        if h < 1 {
                            color = Color32::BLUE;
                        } else if h < 64 {
                            color = Color32::WHITE;
                        } else if h < 128 {
                            color = Color32::BROWN;
                        } else if h < 192 {
                            color = Color32::GREEN;
                        } else if h < 254 {
                            color = Color32::BLUE;
                        } else {
                            color = Color32::BLACK;
                        }
                        // cities
                        if map_data.cells.contains_key(&key) {
                            let cell = map_data.cells.get(&key).unwrap();

                            if let Some(map_color) = cell.map_color {
                                color = Color32::from_rgb_additive(
                                    map_color[0],
                                    map_color[1],
                                    map_color[2],
                                );
                            }
                        }
                        // selected
                        // if let Some(grid) = map_data.cell_ids.get(&map_data.selected_id) {
                        //     if grid == &key {
                        //         color = Color32::RED;
                        //     }
                        // }

                        let grid_scale = 1.0 / 9.0;
                        let grid_x = (x as f32) + ((hx as f32) * grid_scale);
                        let grid_y = (y as f32) + ((hy as f32) * grid_scale);

                        let cell_pos = to_screen * Pos2::new(grid_x, grid_y);
                        let cell_pos2 =
                            to_screen * Pos2::new(grid_x + grid_scale, grid_y + grid_scale);
                        let cell_rect = egui::epaint::Rect::from_two_pos(cell_pos, cell_pos2);
                        let rect_shape =
                            Shape::rect_filled(cell_rect, egui::Rounding::none(), color);
                        shapes.push(rect_shape);
                    }
                });
            }
        }
    }
    painter.extend(shapes);

    if let Some(hover_pos) = painter.ctx().pointer_hover_pos() {
        let real_pos = from_screen * hover_pos;

        let mut x = real_pos.x;
        let mut y = -real_pos.y;
        // hacks to get the correct cell name
        if x < 0.0 {
            x -= 1.0;
        }
        if y > 0.0 {
            y += 1.0;
        }
        map_data.hover_pos = (x as i32, y as i32);
    }
}
