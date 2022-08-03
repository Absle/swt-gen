use egui::{vec2, Context, Layout, RichText, Slider, Vec2, Window};

use super::{GeneratorApp, Message};

const DEFAULT_POPUP_SIZE: Vec2 = vec2(256.0, 144.0);
pub(crate) trait Popup {
    /** Show this `Popup`.

    ## Returns

    - `Some(Message)` with the `Message` to be processed when the `Popup` dialog has been answered
    - `None` if the `Popup` dialog has not been answered yet
    */
    fn show(&mut self, ctx: &Context) -> Option<Message>;
}

#[derive(Clone)]
pub(crate) struct ConfirmationPopup {
    title: String,
    text: String,
    confirm_message: Message,
    cancel_message: Message,
}

impl ConfirmationPopup {
    pub(crate) fn new(
        title: String,
        text: String,
        confirm_message: Message,
        cancel_message: Message,
    ) -> Self {
        Self {
            title,
            text,
            confirm_message,
            cancel_message,
        }
    }

    pub(crate) fn unsaved_changes(confirm_message: Message, cancel_message: Message) -> Self {
        Self {
            title: "Unsaved Subsector Changes".to_string(),
            text: "Any unsaved changes will be lost.\nAre you sure you'd like to continue?"
                .to_string(),
            confirm_message,
            cancel_message,
        }
    }
}

impl Popup for ConfirmationPopup {
    fn show(&mut self, ctx: &Context) -> Option<Message> {
        let ConfirmationPopup {
            title,
            text,
            confirm_message,
            cancel_message,
        } = self;

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
                    if ui.button("Confirm").clicked() {
                        result = Some(confirm_message.clone())
                    }

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button("Cancel").clicked() {
                            result = Some(cancel_message.clone())
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

pub(crate) struct SubsectorRegenPopup {
    world_abundance_dm: i16,
}

impl Default for SubsectorRegenPopup {
    fn default() -> Self {
        Self {
            world_abundance_dm: 0,
        }
    }
}

impl Popup for SubsectorRegenPopup {
    fn show(&mut self, ctx: &Context) -> Option<Message> {
        let mut result = None;

        let title = "Regenerate Subsector";

        Window::new(title.clone())
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(title);
                    ui.separator();
                    ui.add_space(GeneratorApp::FIELD_SPACING / 2.0);

                    ui.label(
                        RichText::new("World Abundance Modifier")
                            .font(GeneratorApp::LABEL_FONT)
                            .color(GeneratorApp::LABEL_COLOR),
                    );
                    ui.add_space(GeneratorApp::LABEL_SPACING);
                    let slider = Slider::new(&mut self.world_abundance_dm, -2..=2);
                    ui.add(slider);
                });
                ui.add_space(GeneratorApp::FIELD_SPACING);

                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        result = Some(Message::ConfirmRegenSubsector {
                            world_abundance_dm: self.world_abundance_dm,
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
