use eframe::{App, Frame};
use egui::{CentralPanel, ColorImage, Context, Image, Label, Sense};
use egui_extras::RetainedImage;

use super::astrography::Subsector;

pub struct GeneratorApp {
    subsector: Subsector,
    subsector_svg: String,
    subsector_image: RetainedImage,
}

impl Default for GeneratorApp {
    fn default() -> Self {
        let subsector = Subsector::new(0);
        let subsector_svg = subsector.generate_svg();
        let subsector_image = generate_subsector_image(subsector.name(), &subsector_svg).unwrap();

        Self {
            subsector,
            subsector_svg,
            subsector_image,
        }
    }
}

impl App for GeneratorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("SVG example");

            //ui.label("The SVG is rasterized and displayed as a texture.");
            let label = Label::new("The SVG is rasterized and displayed as a texture.")
                .sense(Sense::click());
            if ui.add(label).clicked() {
                println!("Label Clicked!")
            }

            ui.separator();

            let max_size = ui.available_size();
            let mut desired_size = self.subsector_image.size_vec2();
            desired_size *= (max_size.x / desired_size.x).min(1.0);
            desired_size *= (max_size.y / desired_size.y).min(1.0);

            let image = Image::new(self.subsector_image.texture_id(&ctx), desired_size)
                .sense(Sense::click());

            let response = ui.add(image);
            if response.clicked() {
                self.subsector = Subsector::new(0);
                self.subsector_svg = self.subsector.generate_svg();
                self.subsector_image =
                    generate_subsector_image(self.subsector.name(), &self.subsector_svg).unwrap();
            }
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

    // Linux.
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        "Liberation Sans".to_string()
    }
}
