use egui::{vec2, Context, Grid, Layout, RichText, Vec2, Window};

use crate::astrography::WorldAbundance;

use super::{GeneratorApp, Message};

const DEFAULT_POPUP_SIZE: Vec2 = vec2(256.0, 144.0);
pub(crate) trait Popup {
    /** Show this `Popup`.

    # Returns
    - `Some(Message)` with the `Message` to be processed when the `Popup` dialog has been answered
    - `None` if the `Popup` dialog has not been answered yet
    */
    fn show(&mut self, ctx: &Context) -> Option<Message>;
}

pub(crate) struct ButtonPopup {
    title: String,
    text: String,
    buttons: Vec<(String, Message)>,
}

impl ButtonPopup {
    pub(crate) fn add_button(&mut self, button_text: String, message: Message) {
        self.buttons.push((button_text, message));
    }

    pub(crate) fn add_confirm_buttons(&mut self, confirm: Message, cancel: Message) {
        self.buttons.push(("Confirm".to_string(), confirm));
        self.buttons.push(("Cancel".to_string(), cancel));
    }

    pub(crate) fn new(title: String, text: String) -> Self {
        Self {
            title,
            text,
            buttons: Vec::new(),
        }
    }

    pub(crate) fn unsaved_changes_dialog(
        text: String,
        save: Message,
        no_save: Message,
        cancel: Message,
    ) -> Self {
        let mut buttons = Vec::new();
        buttons.push(("Save".to_string(), save));
        buttons.push(("Don't Save".to_string(), no_save));
        buttons.push(("Cancel".to_string(), cancel));
        Self {
            title: "Unsaved Changes".to_string(),
            text,
            buttons,
        }
    }
}

impl Popup for ButtonPopup {
    fn show(&mut self, ctx: &Context) -> Option<Message> {
        let ButtonPopup {
            title,
            text,
            buttons,
        } = self;

        // `ButtonPopup` without any buttons can't be closed and will lock the app
        assert!(
            buttons.len() > 0,
            "Must add at least one button to the `ButtonPopup`!"
        );

        let mut result = None;

        Window::new(title.clone())
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(GeneratorApp::FIELD_SPACING / 2.0);
                    ui.label(text.clone());
                });
                ui.add_space(GeneratorApp::FIELD_SPACING);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        for (button_text, message) in buttons.iter().rev() {
                            if ui.button(button_text).clicked() {
                                result = Some(message.clone());
                            }
                        }
                    });
                });
            });
        result
    }
}

pub(crate) struct SubsectorRegenPopup {
    world_abundance: WorldAbundance,
}

impl Default for SubsectorRegenPopup {
    fn default() -> Self {
        Self {
            world_abundance: WorldAbundance::Nominal,
        }
    }
}

impl Popup for SubsectorRegenPopup {
    fn show(&mut self, ctx: &Context) -> Option<Message> {
        let mut result = None;

        let title = "Choose World Abundance";
        let popup_size = DEFAULT_POPUP_SIZE;

        Window::new(title.clone())
            .title_bar(false)
            .resizable(false)
            .fixed_size(popup_size)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(GeneratorApp::FIELD_SPACING / 2.0);

                    let column_count = WorldAbundance::WORLD_ABUNDANCE_VALUES.len() as f32;
                    let grid_spacing = vec2(
                        GeneratorApp::FIELD_SPACING / 2.0,
                        GeneratorApp::LABEL_SPACING,
                    );
                    let column_width =
                        (popup_size.x - (column_count - 1.0) * grid_spacing.x) / column_count;

                    Grid::new("subsector_regen_grid")
                        .spacing(grid_spacing)
                        .min_col_width(column_width)
                        .show(ui, |ui| {
                            for world_abundance in WorldAbundance::WORLD_ABUNDANCE_VALUES {
                                ui.vertical_centered(|ui| {
                                    ui.radio_value(&mut self.world_abundance, world_abundance, "");
                                });
                            }
                            ui.end_row();

                            for world_abundance in WorldAbundance::WORLD_ABUNDANCE_VALUES {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(world_abundance.to_string())
                                            .font(GeneratorApp::LABEL_FONT)
                                            .color(GeneratorApp::LABEL_COLOR),
                                    );
                                });
                            }
                        });
                });
                ui.add_space(GeneratorApp::FIELD_SPACING);

                ui.horizontal(|ui| {
                    if ui.button("Generate").clicked() {
                        result = Some(Message::ConfirmRegenSubsector {
                            world_abundance_dm: self.world_abundance.into(),
                        })
                    }

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button("Cancel").clicked() {
                            result = Some(Message::CancelRegenSubsector)
                        }
                    });
                });
            });
        result
    }
}

pub(crate) struct SubsectorRenamePopup {
    name: String,
}

impl SubsectorRenamePopup {
    pub(crate) fn new(initial_name: &str) -> Self {
        Self {
            name: initial_name.to_string(),
        }
    }
}

impl Popup for SubsectorRenamePopup {
    fn show(&mut self, ctx: &Context) -> Option<Message> {
        let mut result = None;

        let title = "Rename Subsector";

        Window::new(title.clone())
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(GeneratorApp::FIELD_SPACING / 2.0);
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.add_space(GeneratorApp::FIELD_SPACING);

                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        result = Some(Message::ConfirmRenameSubsector {
                            new_name: self.name.clone(),
                        })
                    }

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button("Cancel").clicked() {
                            result = Some(Message::CancelRenameSubsector)
                        }
                    });
                });
            });
        result
    }
}
