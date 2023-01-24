use egui::{vec2, Context, Grid, Layout, Pos2, RichText, TextEdit, Vec2, Window};

use crate::{
    app::{
        gui::{FIELD_SPACING, LABEL_COLOR, LABEL_FONT, LABEL_SPACING},
        pipe, GeneratorApp, Message,
    },
    astrography::{Point, WorldAbundance},
};

const DEFAULT_POPUP_SIZE: Vec2 = vec2(256.0, 144.0);

impl GeneratorApp {
    /** Add a `Popup` to the queue to be shown and awaiting response. */
    pub(crate) fn add_popup<T: 'static + Popup>(&mut self, popup: T) {
        self.popup_queue.push(Box::new(popup));
    }

    pub(crate) fn occupied_hex_popup(&mut self, world_name: String, location: Point) {
        let popup = ButtonPopup::new(
            "Destination Hex Occupied".to_string(),
            format!(
                "'{}' is already at {}.\nWould you like to overwrite it?",
                world_name, location
            ),
            self.message_tx.clone(),
        )
        .add_confirm_buttons(
            Message::ConfirmLocUpdate { location },
            Message::CancelLocUpdate,
        );

        self.add_popup(popup);
    }

    pub(crate) fn regen_world_popup(&mut self) {
        let popup = ButtonPopup::new(
            "Regenerating World".to_string(),
            format!(
                "Do you want to completely regenerate '{}'? This can not be undone.",
                self.world.name
            ),
            self.message_tx.clone(),
        )
        .add_confirm_buttons(Message::ConfirmRegenWorld, Message::NoOp);

        self.add_popup(popup);
    }

    pub(crate) fn remove_world_popup(&mut self) {
        let popup = ButtonPopup::new(
            "Removing World".to_string(),
            format!(
                "Do you want to completely remove '{}'? This can not be undone.",
                self.world.name
            ),
            self.message_tx.clone(),
        )
        .add_confirm_buttons(
            Message::ConfirmRemoveWorld { point: self.point },
            Message::NoOp,
        );

        self.add_popup(popup);
    }

    pub(crate) fn subsector_regen_popup(&mut self) {
        self.add_popup(SubsectorRegenPopup::new(self.message_tx.clone()));
    }

    pub(crate) fn subsector_rename_popup(&mut self) {
        self.add_popup(SubsectorRenamePopup::new(
            self.subsector.name(),
            self.message_tx.clone(),
        ));
    }

    pub(crate) fn unapplied_world_popup(&mut self, new_point: Point) {
        let popup = ButtonPopup::new(
            "Unapplied World Changes".to_string(),
            format!(
                "Do you want to apply your changes to '{}'?",
                self.world.name
            ),
            self.message_tx.clone(),
        )
        .add_button(
            "Apply".to_string(),
            Message::ApplyConfirmHexGridClicked { new_point },
        )
        .add_button(
            "Don't Apply".to_string(),
            Message::ConfirmHexGridClicked { new_point },
        )
        .add_button("Cancel".to_string(), Message::NoOp);

        self.add_popup(popup);
    }

    pub(crate) fn unsaved_exit_popup(&mut self) {
        let popup = ButtonPopup::unsaved_changes_dialog(
            format!(
                "Do you want to save changes to Subsector {}?",
                self.subsector.name()
            ),
            Message::SaveExit,
            Message::ConfirmUnsavedExit,
            Message::CancelUnsavedExit,
            self.message_tx.clone(),
        );

        self.add_popup(popup);
    }

    pub(crate) fn unsaved_subsector_regen_popup(&mut self) {
        let popup = ButtonPopup::unsaved_changes_dialog(
            format!(
                "Do you want to save changes to Subsector {}?",
                self.subsector.name()
            ),
            Message::SaveConfigRegenSubsector,
            Message::ConfigRegenSubsector,
            Message::NoOp,
            self.message_tx.clone(),
        );

        self.add_popup(popup);
    }

    pub(crate) fn unsaved_subsector_reload_popup(&mut self) {
        let popup = ButtonPopup::unsaved_changes_dialog(
            format!(
                "Do you want to save changes to Subsector {}?",
                self.subsector.name()
            ),
            Message::SaveConfirmImportJson,
            Message::ConfirmImportJson,
            Message::NoOp,
            self.message_tx.clone(),
        );
        self.add_popup(popup);
    }
}

pub(crate) trait Popup {
    /** Indicate if this `Popup` can be removed from the render queue. */
    fn is_done(&self) -> bool;

    /** Show this `Popup`.

    # Returns
    - `Some(Message)` with the `Message` to be processed when the `Popup` dialog has been answered
    - `None` if the `Popup` dialog has not been answered yet
    */
    fn show(&mut self, ctx: &Context);
}

struct ButtonPopup {
    buttons: Vec<(String, Message)>,
    is_done: bool,
    message_tx: pipe::Sender<Message>,
    text: String,
    title: String,
}

impl ButtonPopup {
    fn add_button(mut self, button_text: String, message: Message) -> Self {
        self.buttons.push((button_text, message));
        self
    }

    fn add_confirm_buttons(mut self, confirm: Message, cancel: Message) -> Self {
        self.buttons.push(("Confirm".to_string(), confirm));
        self.buttons.push(("Cancel".to_string(), cancel));
        self
    }

    fn new(title: String, text: String, message_tx: pipe::Sender<Message>) -> Self {
        Self {
            buttons: Vec::new(),
            is_done: false,
            message_tx,
            text,
            title,
        }
    }

    fn unsaved_changes_dialog(
        text: String,
        save: Message,
        no_save: Message,
        cancel: Message,
        message_tx: pipe::Sender<Message>,
    ) -> Self {
        let buttons = vec![
            ("Save".to_string(), save),
            ("Don't Save".to_string(), no_save),
            ("Cancel".to_string(), cancel),
        ];
        Self {
            buttons,
            is_done: false,
            message_tx,
            text,
            title: "Unsaved Changes".to_string(),
        }
    }
}

impl Popup for ButtonPopup {
    fn is_done(&self) -> bool {
        self.is_done
    }

    fn show(&mut self, ctx: &Context) {
        // `ButtonPopup` without any buttons can't be closed and will lock the app
        assert!(
            !self.buttons.is_empty(),
            "All ButtonPopups should have at lease one button added."
        );

        Window::new(self.title.clone())
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .default_pos(center(ctx))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(self.title.clone());
                    ui.separator();
                    ui.add_space(FIELD_SPACING / 2.0);
                    ui.label(self.text.clone());
                });
                ui.add_space(FIELD_SPACING);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        for (button_text, message) in self.buttons.iter().rev() {
                            if ui.button(button_text).clicked() {
                                self.message_tx.send(message.clone());
                                self.is_done = true;
                            }
                        }
                    });
                });
            });
    }
}

struct SubsectorRegenPopup {
    is_done: bool,
    message_tx: pipe::Sender<Message>,
    world_abundance: WorldAbundance,
}

impl SubsectorRegenPopup {
    fn new(message_tx: pipe::Sender<Message>) -> SubsectorRegenPopup {
        Self {
            is_done: false,
            message_tx,
            world_abundance: WorldAbundance::Nominal,
        }
    }
}

impl Popup for SubsectorRegenPopup {
    fn is_done(&self) -> bool {
        self.is_done
    }

    fn show(&mut self, ctx: &Context) {
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
                        self.message_tx.send(Message::ConfirmRegenSubsector {
                            world_abundance_dm: self.world_abundance.into(),
                        });
                        self.is_done = true;
                    }

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.message_tx.send(Message::NoOp);
                            self.is_done = true;
                        }
                    });
                });
            });
    }
}

struct SubsectorRenamePopup {
    is_done: bool,
    message_tx: pipe::Sender<Message>,
    name: String,
}

impl SubsectorRenamePopup {
    fn new(initial_name: &str, message_tx: pipe::Sender<Message>) -> Self {
        Self {
            is_done: false,
            message_tx,
            name: initial_name.to_string(),
        }
    }
}

impl Popup for SubsectorRenamePopup {
    fn is_done(&self) -> bool {
        self.is_done
    }

    fn show(&mut self, ctx: &Context) {
        const TITLE: &str = "Rename Subsector";

        Window::new(TITLE)
            .title_bar(false)
            .resizable(false)
            .fixed_size(DEFAULT_POPUP_SIZE)
            .default_pos(center(ctx))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(TITLE);
                    ui.separator();
                    ui.add_space(FIELD_SPACING / 2.0);
                    ui.add(TextEdit::singleline(&mut self.name).margin(vec2(16.0, 4.0)));
                });
                ui.add_space(FIELD_SPACING);

                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        self.message_tx.send(Message::ConfirmRenameSubsector {
                            new_name: self.name.clone(),
                        });
                        self.is_done = true;
                    }

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.message_tx.send(Message::NoOp);
                            self.is_done = true;
                        }
                    });
                });
            });
    }
}

/// Calculate and return the centered position of a default-sized popup for a given `Context`.
#[inline]
fn center(ctx: &Context) -> Pos2 {
    ctx.available_rect().center() - DEFAULT_POPUP_SIZE / 2.0
}
