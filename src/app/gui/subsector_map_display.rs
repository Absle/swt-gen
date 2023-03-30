use eframe::epaint::{CircleShape, QuadraticBezierShape, TextShape};
use egui::{
    vec2, Color32, ColorImage, Context, FontId, Image, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2,
};
use egui_extras::RetainedImage;

use crate::{
    app::{GeneratorApp, Message},
    astrography::{Point, Subsector, World, CENTER_MARKERS},
};

const SUBSECTOR_IMAGE_MIN_SIZE: Vec2 = vec2(1584.0, 834.0);

// SVG document dimensions in inches
const SVG_WIDTH: f32 = 8.5;
const SVG_HEIGHT: f32 = 11.0;

const SVG_VIEW_BOX_WIDTH: f64 = 215.9;

// Margins around hex grid in inches
// const LEFT_MARGIN: f32 = 1.0;
const LEFT_MARGIN: f32 = 1.02;
// const RIGHT_MARGIN: f32 = LEFT_MARGIN;
const RIGHT_MARGIN: f32 = 1.01;
const TOP_MARGIN: f32 = 0.50;
// const BOTTOM_MARGIN: f32 = 1.0;
const BOTTOM_MARGIN: f32 = 1.11;

// Hex dimensions in inches
#[allow(dead_code)]
const HEX_LONG_RADIUS: f32 = 0.52;
#[allow(dead_code)]
const HEX_LONG_DIAMETER: f32 = HEX_LONG_RADIUS * 2.0;
const HEX_SHORT_RADIUS: f32 = 0.45;
#[allow(dead_code)]
const HEX_SHORT_DIAMETER: f32 = HEX_SHORT_RADIUS * 2.0;

const WORLD_FONT_ID: FontId = FontId::proportional(13.0);

enum ClickKind {
    Hex(Point),
    SubsectorName,
    None,
}

impl GeneratorApp {
    /** Displays a map of the [`Subsector`] and handles any mouse clicks on it. */
    pub(crate) fn subsector_map_display(&mut self, ctx: &Context, ui: &mut Ui) {
        if let Ok(new_image) = self.worker_rx.try_recv() {
            self.subsector_grid_image = Some(new_image);
        }

        if self.subsector_grid_image.is_none() {
            let svg = self.subsector.generate_grid_svg();
            self.subsector_grid_image = Some(rasterize_svg(svg));
        }

        let max_size = ui.available_size();
        ui.set_min_size(SUBSECTOR_IMAGE_MIN_SIZE);
        ui.set_max_size(max_size);

        if let Some(grid_image) = &self.subsector_grid_image {
            let mut desired_size = grid_image.size_vec2();
            desired_size *= (max_size.x / desired_size.x).min(1.0);
            desired_size *= (max_size.y / desired_size.y).min(1.0);

            let grid_widget =
                Image::new(grid_image.texture_id(ctx), desired_size).sense(Sense::click());
            let grid_response = ui.add(grid_widget);
            if grid_response.clicked() {
                if let Some(pointer_pos) = grid_response.interact_pointer_pos() {
                    let new_point = determine_click_kind(pointer_pos, &grid_response.rect);

                    // A new point has been selected
                    match new_point {
                        ClickKind::Hex(new_point) => {
                            self.message(Message::HexGridClicked { new_point })
                        }

                        ClickKind::SubsectorName => self.message(Message::RenameSubsector),
                        ClickKind::None => (),
                    }
                }
            }

            let mut shapes = Vec::new();
            shapes.push(draw_subsector_name(
                ctx,
                self.subsector.name(),
                &grid_response.rect,
            ));
            for (point, world) in self.subsector.get_map() {
                shapes.append(&mut draw_world(ctx, point, world, &grid_response.rect));

                // DO NOT DELETE: Uncomment to see centers of all hexes; useful for debugging
                // let center = hex_center(point, &grid_response.rect);
                // let center = vec2(center.x, center.y);
                // let center_circle =
                //     CircleShape::filled(Pos2::from([0.0, 0.0]) + center, 3.5, Color32::GREEN);
                // shapes.push(Shape::Circle(center_circle));
            }

            ui.painter_at(grid_response.rect).extend(shapes);
        }
    }
}

/** Generates a [`RetainedImage`] from an SVG string.

# Panics
On invalid SVG.
*/
pub(crate) fn rasterize_svg(svg: String) -> RetainedImage {
    RetainedImage::from_color_image(
        "subsector_image.svg",
        load_svg_bytes(svg.as_bytes()).expect("Subsector image should rasterize from valid SVG"),
    )
}

/** Loads an SVG byte array and rasterizes it into a [`ColorImage`].

# Returns
- `Ok<ColorImage>` if successful,
- `Err<String>` if the given SVG is invalid
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

/** Converts a pointer position to its corresponding interaction type with the subsector map image.

# Returns
- [`ClickKind::Hex(Point)`] containing the hex grid coordinate if a hex is clicked near its center,
- [`ClickKind::SubsectorName`] if the click is near the subsector name in the top margin,
- [`ClickKind::None`] otherwise
*/
fn determine_click_kind(pointer_pos: Pos2, rect: &Rect) -> ClickKind {
    let pixels_per_inch = rect.width() / SVG_WIDTH;

    // Find pointer position relative to the image
    let relative_pos = pointer_pos - rect.left_top();
    let relative_pos = Pos2::from([relative_pos.x, relative_pos.y]);

    // Find the rect containing the subsector name; just a centered section of the top margin
    let left_bound = 2.0 * LEFT_MARGIN * pixels_per_inch;
    let right_bound = (SVG_WIDTH - 2.0 * RIGHT_MARGIN) * pixels_per_inch;
    let top_bound = 0.0;
    let bottom_bound = 0.75 * TOP_MARGIN * pixels_per_inch;

    let left_top = Pos2::from([left_bound, top_bound]);
    let right_bottom = Pos2::from([right_bound, bottom_bound]);
    let subsector_name_rect = Rect::from_min_max(left_top, right_bottom);
    if subsector_name_rect.contains(relative_pos) {
        return ClickKind::SubsectorName;
    }

    let left_bound = LEFT_MARGIN * pixels_per_inch;
    let right_bound = (SVG_WIDTH - RIGHT_MARGIN) * pixels_per_inch;
    let top_bound = TOP_MARGIN * pixels_per_inch;
    let bottom_bound = (SVG_HEIGHT - BOTTOM_MARGIN) * pixels_per_inch;

    let left_top = Pos2::from([left_bound, top_bound]);
    let right_bottom = Pos2::from([right_bound, bottom_bound]);
    let grid_rect = Rect::from_min_max(left_top, right_bottom);
    if !grid_rect.contains(relative_pos) {
        return ClickKind::None;
    }

    // Find the hex center that is nearest to the click position
    let mut smallest_distance = f32::MAX;
    let mut point = Point { x: 0, y: 0 };
    for x in 1..=Subsector::COLUMNS {
        for y in 1..=Subsector::ROWS {
            let center = hex_center(
                &Point {
                    x: x as i32,
                    y: y as i32,
                },
                rect,
            );
            let distance = center.distance(pointer_pos);
            if distance < smallest_distance {
                smallest_distance = distance;
                point = Point {
                    x: x as i32,
                    y: y as i32,
                };
            }
        }
    }

    if smallest_distance < HEX_SHORT_RADIUS * pixels_per_inch {
        ClickKind::Hex(point)
    } else {
        ClickKind::None
    }
}

fn draw_subsector_name(ctx: &Context, subsector_name: &str, rect: &Rect) -> Shape {
    const SUBSECTOR_NAME_FONT_ID: FontId = FontId::proportional(28.0);
    let galley = ctx.fonts().layout_no_wrap(
        format!("{} Subsector", subsector_name),
        SUBSECTOR_NAME_FONT_ID,
        Color32::BLACK,
    );
    let text_width = galley.rect.width();
    let offset = vec2(-text_width / 2.0, 0.0);
    let position = rect.center_top() + offset;
    Shape::Text(TextShape::new(position, galley))
}

fn draw_world(ctx: &Context, point: &Point, world: &World, rect: &Rect) -> Vec<Shape> {
    let mut shapes = Vec::new();

    let center = hex_center(point, rect);
    let pixels_per_unit = rect.width() / SVG_VIEW_BOX_WIDTH as f32;

    // Draw world gas giant indicator
    if world.has_gas_giant() {
        shapes.append(&mut draw_world_gas_giant(&center, pixels_per_unit));
    }

    // Draw world name
    shapes.push(draw_world_name(ctx, &center, &world.name));

    // Draw wet/dry world indicator
    shapes.push(draw_world_wet_dry_indicator(
        &center,
        pixels_per_unit,
        world.is_wet_world(),
    ));

    // Draw Starport-TechLevel
    shapes.push(draw_world_starport_tl(
        ctx,
        &center,
        pixels_per_unit,
        &world.starport_tl_str(),
    ));

    // Draw UWP
    shapes.push(draw_world_profile(
        ctx,
        &center,
        pixels_per_unit,
        &world.profile_str(),
    ));

    shapes
}

fn draw_world_gas_giant(center: &Pos2, pixels_per_unit: f32) -> Vec<Shape> {
    // How much offset from hex's center to place the gas giant in SVG userspace units
    const OFFSET: Vec2 = vec2(0.0, -6.0);

    const ELLIPSE_MAJOR_AXIS: f32 = 14.0;
    // Rotation of ring tilt about center of gas giant; degrees to radians
    const ELLIPSE_ANGLE: f32 = 21.0 * (std::f32::consts::PI / 180.0);

    // Bezier control point distance from major axis
    const CP_LEN: f32 = 2.825;
    // Absolute angle about gas giant center to rotate the control point
    const CP_ANGLE: f32 = (90.0 * (std::f32::consts::PI / 180.0)) - ELLIPSE_ANGLE;

    let offset = OFFSET * pixels_per_unit;
    let center = vec2(center.x, center.y);
    let stroke = Stroke::from((1.0, Color32::BLACK));

    let x = (ELLIPSE_MAJOR_AXIS / 2.0) * ELLIPSE_ANGLE.cos();
    let y = (ELLIPSE_MAJOR_AXIS / 2.0) * ELLIPSE_ANGLE.sin();
    let p1 = Pos2::from([-x, y]) + center + offset;
    let p2 = Pos2::from([x, -y]) + center + offset;

    let x = CP_LEN * CP_ANGLE.cos();
    let y = CP_LEN * CP_ANGLE.sin();
    let cp1 = Pos2::from([-x, -y]) + center + offset;
    let cp2 = Pos2::from([x, y]) + center + offset;

    let upper_curve = QuadraticBezierShape::from_points_stroke(
        [p1, cp1, p2],
        false,
        Color32::TRANSPARENT,
        stroke,
    );

    let lower_curve = QuadraticBezierShape::from_points_stroke(
        [p1, cp2, p2],
        false,
        Color32::TRANSPARENT,
        stroke,
    );

    let circle = CircleShape::filled(
        Pos2::from([0.0, 0.0]) + center + offset,
        3.5,
        Color32::BLACK,
    );

    vec![
        Shape::QuadraticBezier(upper_curve),
        Shape::QuadraticBezier(lower_curve),
        Shape::Circle(circle),
    ]
}

fn draw_world_name(ctx: &Context, center: &Pos2, name: &str) -> Shape {
    let galley = ctx
        .fonts()
        .layout_no_wrap(name.to_string(), WORLD_FONT_ID, Color32::BLACK);
    let text_width = galley.rect.width();
    let text_height = galley.rect.height();
    let offset = vec2(-text_width / 2.0, -text_height / 1.5);
    let position = *center + offset;
    Shape::Text(TextShape::new(position, galley))
}

fn draw_world_profile(
    ctx: &Context,
    center: &Pos2,
    pixels_per_unit: f32,
    profile_str: &str,
) -> Shape {
    const UWP_FONT_ID: FontId = FontId::proportional(10.0);
    let galley = ctx
        .fonts()
        .layout_no_wrap(profile_str.to_string(), UWP_FONT_ID, Color32::BLACK);
    let text_width = galley.rect.width();
    let text_height = galley.rect.height();
    let x = -text_width / 2.0;
    let y = 10.0 * pixels_per_unit - text_height / 2.0;
    let offset = vec2(x, y);
    let position = *center + offset;
    Shape::Text(TextShape::new(position, galley))
}

fn draw_world_starport_tl(
    ctx: &Context,
    center: &Pos2,
    pixels_per_unit: f32,
    starport_tl: &str,
) -> Shape {
    let galley = ctx
        .fonts()
        .layout_no_wrap(starport_tl.to_string(), WORLD_FONT_ID, Color32::BLACK);
    let text_width = galley.rect.width();
    let text_height = galley.rect.height();
    let x = 5.0 * pixels_per_unit - text_width / 2.0;
    let y = 5.0 * pixels_per_unit - text_height / 1.5;
    let offset = vec2(x, y);
    let position = *center + offset;
    Shape::Text(TextShape::new(position, galley))
}

fn draw_world_wet_dry_indicator(center: &Pos2, pixels_per_unit: f32, is_wet_world: bool) -> Shape {
    const RADIUS: f32 = 5.0;
    let offset = vec2(-5.0 * pixels_per_unit, 4.5 * pixels_per_unit);
    let position = *center + offset;
    if is_wet_world {
        Shape::Circle(CircleShape::filled(position, RADIUS, Color32::BLACK))
    } else {
        Shape::Circle(CircleShape::stroke(position, RADIUS, (1.0, Color32::BLACK)))
    }
}

fn hex_center(point: &Point, rect: &Rect) -> Pos2 {
    let pixels_per_unit = rect.width() as f64 / SVG_VIEW_BOX_WIDTH;

    let translation = CENTER_MARKERS[point];
    let x = translation.x * pixels_per_unit + rect.left() as f64;
    let y = translation.y * pixels_per_unit + rect.top() as f64;

    Pos2::from([x as f32, y as f32])
}
/** Returns the best guess of the system's default sans-serif font. */
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
