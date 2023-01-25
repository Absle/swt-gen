mod popup;
mod subsector_map_display;
mod world_data_display;

use egui::{menu, Button, CentralPanel, Color32, Context, FontId, TopBottomPanel};

use crate::app::{GeneratorApp, Message};

pub(crate) use popup::Popup;
pub(crate) use subsector_map_display::generate_subsector_image;
pub(crate) use world_data_display::TabLabel;

pub(crate) const LABEL_FONT: FontId = FontId::proportional(11.0);
pub(crate) const LABEL_COLOR: Color32 = Color32::GRAY;
pub(crate) const LABEL_SPACING: f32 = 4.0;

/// Soft, nice blue color used by egui `SelectableValue` widgets.
pub(crate) const POSITIVE_BLUE: Color32 = Color32::from_rgb(144, 209, 255);

/// "Negative" red color meant to pair well with `POSITIVE_BLUE` aesthetically
pub(crate) const NEGATIVE_RED: Color32 = Color32::from_rgb(255, 144, 144);

pub(crate) const BUTTON_FONT_SIZE: f32 = 16.0;

pub(crate) const FIELD_SPACING: f32 = 15.0;
pub(crate) const FIELD_SELECTION_WIDTH: f32 = 225.0;
pub(crate) const SHORT_SELECTION_WIDTH: f32 = 50.0;

pub(crate) const DICE_ICON: &str = "🎲";
pub(crate) const X_ICON: &str = "❌";
pub(crate) const SAVE_ICON: &str = "💾";

impl GeneratorApp {
    /** Handles displaying the overall central panel of the app.

    Shows the map of the `Subsector` on the left half of the panel and any information of the
    selected `Point` and/or `World` on the right half.
    If there is no `World` at the selected `Point`, it shows a button to add a new world at there.
    If there is a `World` there, displays the data associated with that `World`.
    */
    fn show_central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(self.popup_queue.is_empty(), |ui| {
                ui.horizontal_top(|ui| {
                    self.subsector_map_display(ctx, ui);

                    ui.separator();

                    if self.point_selected && self.world_selected {
                        self.world_data_display(ui);
                    } else if self.point_selected {
                        self.new_world_dialog(ui);
                    }
                });
            });
        });
    }

    /** Render all GUI elements. */
    pub(crate) fn show_gui(&mut self, ctx: &Context) {
        self.show_top_panel(ctx);
        self.show_central_panel(ctx);
        self.show_popups(ctx);
    }

    /** Display all `Popup`'s in the queue and process any messages they return. */
    fn show_popups(&mut self, ctx: &Context) {
        let mut done = Vec::new();
        for (i, popup) in self.popup_queue.iter_mut().enumerate() {
            if popup.is_done() {
                done.push(i);
            } else {
                popup.show(ctx);
            }
        }

        for i in done {
            if self.popup_queue.get(i).is_some() {
                self.popup_queue.remove(i);
            }
        }
    }

    /** Displays the top panel of the app.

    Currently just a menu bar.
    */
    fn show_top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_enabled_ui(self.popup_queue.is_empty(), |ui| {
                menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        let new_subsector_button =
                            Button::new("Generate New Subsector...").wrap(false);
                        if ui.add(new_subsector_button).clicked() {
                            ui.close_menu();
                            self.message(Message::RegenSubsector);
                        }

                        ui.separator();

                        if ui.button("Open...              Ctrl-O").clicked() {
                            ui.close_menu();
                            self.message(Message::OpenJson);
                        }

                        if ui.button("Save                   Ctrl-S").clicked() {
                            ui.close_menu();
                            self.message(Message::Save);
                        }

                        if ui.button("Save As...           Ctrl-Shift-S").clicked() {
                            ui.close_menu();
                            self.message(Message::SaveAs);
                        }

                        ui.separator();

                        ui.menu_button("Export", |ui| {
                            if ui.button("Subsector Map SVG...").clicked() {
                                ui.close_menu();
                                self.message(Message::ExportSubsectorMapSvg);
                            }

                            let button = Button::new("Player-Safe Subsector JSON...").wrap(false);
                            if ui.add(button).clicked() {
                                self.message(Message::ExportPlayerSafeSubsectorJson);
                            }
                        });
                    });

                    ui.menu_button("Edit", |ui| {
                        let rename_button =
                            Button::new("Rename Subsector...    Ctrl-N").wrap(false);
                        if ui.add(rename_button).clicked() {
                            ui.close_menu();
                            self.message(Message::RenameSubsector);
                        }
                    });
                });
            });
        });
    }
}
