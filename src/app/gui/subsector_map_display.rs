use egui::{vec2, ColorImage, Context, Image, Pos2, Rect, Sense, Ui, Vec2};

use egui_extras::RetainedImage;

use crate::{
    app::{GeneratorApp, Message, COLORED},
    astrography::{Point, Subsector},
};

const SUBSECTOR_IMAGE_MIN_SIZE: Vec2 = vec2(1584.0, 834.0);

enum ClickKind {
    Hex(Point),
    SubsectorName,
    None,
}

impl GeneratorApp {
    /** Displays a map of the [`Subsector`] and handles any mouse clicks on it. */
    pub(crate) fn subsector_map_display(&mut self, ctx: &Context, ui: &mut Ui) {
        if let Ok(new_image) = self.worker_rx.try_recv() {
            self.subsector_image = Some(new_image);
        }

        let max_size = ui.available_size();
        ui.set_min_size(SUBSECTOR_IMAGE_MIN_SIZE);
        ui.set_max_size(max_size);

        if self.subsector_image.is_none() {
            let svg = self.subsector.generate_svg(COLORED);
            self.subsector_image = Some(generate_subsector_image(svg));
        }

        if let Some(subsector_image) = &self.subsector_image {
            let mut desired_size = subsector_image.size_vec2();
            desired_size *= (max_size.x / desired_size.x).min(1.0);
            desired_size *= (max_size.y / desired_size.y).min(1.0);

            let ui_image =
                Image::new(subsector_image.texture_id(ctx), desired_size).sense(Sense::click());

            let response = ui.add(ui_image);
            if response.clicked() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let new_point = determine_click_kind(pointer_pos, &response.rect);

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
        }
    }
}

/** Generates a [`RetainedImage`] from an SVG string.

# Panics
On invalid SVG.
*/
pub(crate) fn generate_subsector_image(svg: String) -> RetainedImage {
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

    // Find pointer position relative to the image
    let relative_pos = pointer_pos - rect.left_top();
    let relative_pos = Pos2::from([relative_pos.x, relative_pos.y]);

    // Find the rect containing the subsector name; just a centered section of the top margin
    let left_bound: f32 = 2.0 * LEFT_MARGIN * pixels_per_inch;
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
