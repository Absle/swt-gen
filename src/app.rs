use eframe::{App, Frame};

use egui::{
    vec2, CentralPanel, Color32, ColorImage, ComboBox, Context, FontId, Image, Label, Pos2, Rect,
    RichText, ScrollArea, Sense, TextEdit, Ui, Vec2,
};
use egui_extras::RetainedImage;

use crate::astrography::world::{TravelCode, World};

use super::astrography::{Point, Subsector};

pub struct GeneratorApp {
    subsector: Subsector,
    subsector_svg: String,
    subsector_image: RetainedImage,

    // Control fields
    /// A new point has been selected this frame
    new_point_selected: bool,

    /// A point is currently selected
    point_selected: bool,

    /// A world is currently selected
    world_selected: bool,

    selected_point: Point,
    selected_world: World,

    // Mirror fields
    world_loc: String,
    world_diameter: String,
}

impl GeneratorApp {
    const CENTRAL_PANEL_MIN_SIZE: Vec2 = vec2(1584.0, 834.0);

    const LABEL_FONT: FontId = FontId::proportional(11.0);
    const LABEL_COLOR: Color32 = Color32::GRAY;
    const FIELD_SPACING: f32 = 15.0;

    /** Regenerate `subsector_svg` and `subsector_image` after a change to `subsector`. */
    fn regenerate_subsector_image(&mut self) {
        self.subsector_svg = self.subsector.generate_svg();
        self.subsector_image =
            generate_subsector_image(self.subsector.name(), &self.subsector_svg).unwrap();
    }

    fn subsector_map_display(&mut self, ctx: &Context, ui: &mut Ui) {
        let max_size = ui.available_size();
        ui.set_min_size(Self::CENTRAL_PANEL_MIN_SIZE);
        ui.set_max_size(max_size);

        let mut desired_size = self.subsector_image.size_vec2();
        desired_size *= (max_size.x / desired_size.x).min(1.0);
        desired_size *= (max_size.y / desired_size.y).min(1.0);

        let subsector_image =
            Image::new(self.subsector_image.texture_id(&ctx), desired_size).sense(Sense::click());

        let response = ui.add(subsector_image);
        if response.clicked() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let new_point = pointer_pos_to_hex_point(pointer_pos, &response.rect);

                // A new point has been selected
                if let Some(new_point) = new_point {
                    self.new_point_selected = true;

                    // Save world changes with any user-edited data
                    if self.world_selected {
                        self.subsector
                            .map
                            .insert(self.selected_point.clone(), self.selected_world.clone());
                    }

                    self.point_selected = true;
                    self.selected_point = new_point;

                    // Update data fields to new worlds
                    let world = self.subsector.map.get(&self.selected_point);
                    if let Some(world) = world {
                        self.world_selected = true;
                        self.selected_world = world.clone();
                        self.world_loc = self.selected_world.location.to_string();
                        self.world_diameter = self.selected_world.diameter.to_string();
                    } else {
                        self.world_selected = false;
                    }
                }
            }
        }
    }

    fn world_data_display(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("World Data");
            self.world_profile_display(ui);

            ui.add_space(Self::FIELD_SPACING / 2.0);
            ui.separator();
            ui.add_space(Self::FIELD_SPACING / 2.0);

            self.world_fields_display(ui);
        });
    }

    fn world_profile_display(&mut self, ui: &mut Ui) {
        let Self {
            subsector: _,
            subsector_svg: _,
            subsector_image: _,

            new_point_selected: _,
            point_selected: _,
            world_selected: _,

            selected_point: _,
            selected_world: world,
            world_loc,
            world_diameter: _,
        } = self;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("World Name")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.add(TextEdit::singleline(&mut world.name).desired_width(150.0));
            });

            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Location")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.add(TextEdit::singleline(world_loc).desired_width(50.0));
            });

            ui.vertical(|ui| {
                ui.label(
                    RichText::new("World Profile")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                if ui
                    .add(Label::new(world.profile()).sense(Sense::click()))
                    .clicked()
                {
                    ui.output().copied_text = world.profile();
                }
            });

            ui.add_space(Self::FIELD_SPACING);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Trade Codes")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                if ui
                    .add(Label::new(world.trade_code_str()).sense(Sense::click()))
                    .clicked()
                {
                    ui.output().copied_text = world.trade_code_str();
                }
            });

            ui.add_space(Self::FIELD_SPACING);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Travel Code")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ComboBox::from_id_source("Travel Code")
                    .selected_text(world.travel_code_str())
                    .show_ui(ui, |ui| {
                        for code in [TravelCode::Safe, TravelCode::Amber, TravelCode::Red] {
                            ui.selectable_value(
                                &mut world.travel_code,
                                code,
                                format!("{:?}", code),
                            );
                        }
                    });
            });

            ui.add_space(Self::FIELD_SPACING);

            ui.checkbox(
                &mut world.has_gas_giant,
                RichText::new("Gas Giant Present")
                    .font(Self::LABEL_FONT)
                    .color(Self::LABEL_COLOR),
            );
        });
    }

    fn world_fields_display(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            // World size fields
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Size")
                            .font(Self::LABEL_FONT)
                            .color(Self::LABEL_COLOR),
                    );
                    ComboBox::from_id_source("Size")
                        .selected_text(format!("{}", self.selected_world.size))
                        .width(45.0)
                        .show_ui(ui, |ui| {
                            for size in World::SIZE_MIN..=World::SIZE_MAX {
                                ui.selectable_value(
                                    &mut self.selected_world.size,
                                    size,
                                    format!("{:?}", size),
                                );
                            }
                        });
                });

                ui.add_space(Self::FIELD_SPACING / 2.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Diameter (km)")
                            .font(Self::LABEL_FONT)
                            .color(Self::LABEL_COLOR),
                    );
                    if ui
                        .add(TextEdit::singleline(&mut self.world_diameter).desired_width(50.0))
                        .lost_focus()
                    {
                        if let Ok(diameter) = self.world_diameter.parse::<u32>() {
                            self.selected_world.diameter = diameter;
                        } else {
                            self.world_diameter = self.selected_world.diameter.to_string();
                        }
                    }
                });

                ui.add_space(Self::FIELD_SPACING);

                ui.vertical(|ui| {
                    // This just here to push the button down in line with Combobox
                    ui.label(
                        RichText::new("")
                            .font(Self::LABEL_FONT)
                            .color(Self::LABEL_COLOR),
                    );
                    // Regenerate size
                    if ui.button("🎲").clicked() {
                        self.selected_world.generate_size();
                        self.selected_world.resolve_trade_codes();
                        self.world_diameter = self.selected_world.diameter.to_string();
                    }
                });
            });

            ui.add_space(Self::FIELD_SPACING);

            ui.label(
                RichText::new("Atmosphere")
                    .font(Self::LABEL_FONT)
                    .color(Self::LABEL_COLOR),
            );
            ui.add(TextEdit::singleline(
                &mut self.selected_world.atmosphere.composition,
            ));
        });
    }
}

impl Default for GeneratorApp {
    fn default() -> Self {
        let subsector = Subsector::default();
        let subsector_svg = subsector.generate_svg();
        let subsector_image = generate_subsector_image(subsector.name(), &subsector_svg).unwrap();
        let selected_point = Point::default();
        let selected_world = World::empty();
        let world_loc = String::new();
        let world_diameter = String::new();

        Self {
            subsector,
            subsector_svg,
            subsector_image,
            new_point_selected: false,
            point_selected: false,
            world_selected: false,
            selected_point,
            selected_world,
            world_loc,
            world_diameter,
        }
    }
}

impl App for GeneratorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.new_point_selected {
            self.regenerate_subsector_image();
            self.new_point_selected = false;
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                self.subsector_map_display(ctx, ui);

                ui.separator();

                if self.point_selected && self.world_selected {
                    self.world_data_display(ui);
                }
            });
        });
    }
}

/** Generate `RetainedImage` from a `Subsector`. */
fn generate_subsector_image(name: &str, svg: &String) -> Result<RetainedImage, String> {
    Ok(RetainedImage::from_color_image(
        format!("{}.svg", name),
        load_svg_bytes(svg.as_bytes())?,
    ))
}

/** Load an SVG and rasterize it into a `ColorImage`.

## Errors
On invalid SVG.
*/
fn load_svg_bytes(svg_bytes: &[u8]) -> Result<ColorImage, String> {
    let mut opt = usvg::Options {
        font_family: system_sans_serif_font(),
        ..Default::default()
    };
    opt.fontdb.load_system_fonts();

    let rtree = usvg::Tree::from_data(svg_bytes, &opt.to_ref()).map_err(|err| err.to_string())?;

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let [w, h] = [pixmap_size.width(), pixmap_size.height()];

    let mut pixmap = tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| format!("Failed to create SVG Pixmap of size {}x{}", w, h))?;

    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .ok_or_else(|| "Failed to render SVG".to_owned())?;

    let image = ColorImage::from_rgba_unmultiplied(
        [pixmap.width() as _, pixmap.height() as _],
        pixmap.data(),
    );

    Ok(image)
}

/** Return name of system default sans-serif font. */
fn system_sans_serif_font() -> String {
    #[cfg(target_os = "windows")]
    {
        "Arial".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "San Francisco".to_string()
    }

    // Linux
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        "Liberation Sans".to_string()
    }
}

/** Return `Point` of clicked hex or `None` if click position is outside the hex grid. */
fn pointer_pos_to_hex_point(pointer_pos: Pos2, rect: &Rect) -> Option<Point> {
    // In inches
    const SVG_WIDTH: f32 = 8.5;
    const SVG_HEIGHT: f32 = 11.0;

    // Margins around hex grid in inches
    const LEFT_MARGIN: f32 = 1.0;
    const RIGHT_MARGIN: f32 = LEFT_MARGIN;
    const TOP_MARGIN: f32 = 0.5;
    const BOTTOM_MARGIN: f32 = 1.0;

    // Hex dimensions in inches
    const HEX_LONG_RADIUS: f32 = 0.52;
    const HEX_LONG_DIAMETER: f32 = HEX_LONG_RADIUS * 2.0;
    const HEX_SHORT_RADIUS: f32 = 0.45;
    const HEX_SHORT_DIAMETER: f32 = HEX_SHORT_RADIUS * 2.0;

    let pixels_per_inch = rect.width() / SVG_WIDTH;

    let left_bound = LEFT_MARGIN * pixels_per_inch;
    let right_bound = (SVG_WIDTH - RIGHT_MARGIN) * pixels_per_inch;
    let top_bound = TOP_MARGIN * pixels_per_inch;
    let bottom_bound = (SVG_HEIGHT - BOTTOM_MARGIN) * pixels_per_inch;

    let left_top = Pos2::from([left_bound, top_bound]);
    let right_bottom = Pos2::from([right_bound, bottom_bound]);
    let grid_rect = Rect::from_min_max(left_top, right_bottom);

    // Make sure click is inside the grid's rectangle, return None if not
    let relative_pos = pointer_pos - rect.left_top();
    let relative_pos = Pos2::from([relative_pos.x, relative_pos.y]);
    if !grid_rect.contains(relative_pos) {
        return None;
    }

    // Find the hex center that is nearest to the click position
    let mut smallest_distance = f32::MAX;
    let mut point = Point { x: 0, y: 0 };
    for x in 1..=Subsector::COLUMNS {
        for y in 1..=Subsector::ROWS {
            let center_x = ((x - 1) as f32 * 0.75 * HEX_LONG_DIAMETER + HEX_LONG_RADIUS)
                * pixels_per_inch
                + left_bound;

            // Even columns are shifted a short radius downwards
            let offset = if x % 2 == 0 {
                HEX_SHORT_RADIUS * pixels_per_inch
            } else {
                0.0
            };
            let center_y = ((y - 1) as f32 * HEX_SHORT_DIAMETER + HEX_SHORT_RADIUS)
                * pixels_per_inch
                + offset
                + top_bound;

            let center = Pos2::from([center_x, center_y]);
            let distance = center.distance(relative_pos);
            if distance < smallest_distance {
                smallest_distance = distance;
                point = Point {
                    x: x as u16,
                    y: y as u16,
                };
            }
        }
    }

    if smallest_distance < HEX_SHORT_RADIUS * pixels_per_inch {
        Some(point)
    } else {
        None
    }
}
