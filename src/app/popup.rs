use egui::{vec2, Context, Grid, Layout, Pos2, RichText, TextEdit, Vec2, Window};

use crate::{
    app::{
        gui::{FIELD_SPACING, LABEL_COLOR, LABEL_FONT, LABEL_SPACING},
        Message,
    },
    astrography::WorldAbundance,
};

const DEFAULT_POPUP_SIZE: Vec2 = vec2(256.0, 144.0);

/// Calculate and return the centered position of a default-sized popup for a given `Context`.
#[inline]
fn center(ctx: &Context) -> Pos2 {
    ctx.available_rect().center() - DEFAULT_POPUP_SIZE / 2.0
}

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
        let buttons = vec![
            ("Save".to_string(), save),
            ("Don't Save".to_string(), no_save),
            ("Cancel".to_string(), cancel),
        ];
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
            !buttons.is_empty(),
            "Must add at least one button to the `ButtonPopup`!"
        );

        let mut result = None;

        Window::new(title.clone())
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .default_pos(center(ctx))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(FIELD_SPACING / 2.0);
                    ui.label(text.clone());
                });
                ui.add_space(FIELD_SPACING);
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

        Window::new(title)
            .title_bar(false)
            .resizable(false)
            .fixed_size(popup_size)
            .default_pos(center(ctx))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(FIELD_SPACING / 2.0);

                    let column_count = WorldAbundance::WORLD_ABUNDANCE_VALUES.len() as f32;
                    let grid_spacing = vec2(FIELD_SPACING / 2.0, LABEL_SPACING);
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
                                            .font(LABEL_FONT)
                                            .color(LABEL_COLOR),
                                    );
                                });
                            }
                        });
                });
                ui.add_space(FIELD_SPACING);

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

        Window::new(title)
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .default_pos(center(ctx))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(FIELD_SPACING / 2.0);
                    ui.add(TextEdit::singleline(&mut self.name).margin(vec2(16.0, 4.0)));
                });
                ui.add_space(FIELD_SPACING);

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
