use std::fmt;

use egui::{
    vec2, Align, Button, ComboBox, FontId, Grid, Key, Layout, RichText, ScrollArea, Style,
    TextEdit, TextStyle, Ui,
};

use crate::{
    app::{
        gui::{
            BUTTON_FONT_SIZE, DICE_ICON, FIELD_SELECTION_WIDTH, FIELD_SPACING, LABEL_COLOR,
            LABEL_FONT, LABEL_SPACING, NEGATIVE_RED, POSITIVE_BLUE, SAVE_ICON,
            SHORT_SELECTION_WIDTH, X_ICON,
        },
        GeneratorApp, Message,
    },
    astrography::{
        CulturalDiffRecord, Faction, GovRecord, StarportClass, TravelCode, World, TABLES,
    },
};

#[derive(PartialEq)]
pub(crate) enum TabLabel {
    WorldSurvey,
    GovernmentLaw,
    #[allow(dead_code)]
    Factions,
    #[allow(dead_code)]
    CultureErrata,
    Notes,
}

impl TabLabel {
    #[cfg(not(feature = "player-safe-gui"))]
    pub(crate) const ALL_VALUES: [TabLabel; 5] = [
        Self::WorldSurvey,
        Self::GovernmentLaw,
        Self::Factions,
        Self::CultureErrata,
        Self::Notes,
    ];

    #[cfg(feature = "player-safe-gui")]
    pub(crate) const ALL_VALUES: [TabLabel; 3] =
        [Self::WorldSurvey, Self::GovernmentLaw, Self::Notes];
}

impl fmt::Display for TabLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TabLabel::WorldSurvey => "World Survey",
            TabLabel::GovernmentLaw => "Government & Law",
            TabLabel::Factions => "Factions",
            TabLabel::CultureErrata => "Culture & Errata",
            TabLabel::Notes => "Notes",
        };
        write!(f, "{}", s)
    }
}

impl GeneratorApp {
    fn apply_revert_buttons(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
            ui.add_space(12.5);
            ui.horizontal(|ui| {
                ui.add_space(12.5);

                let header_font = TextStyle::Heading.resolve(&Style::default());
                let apply_button = Button::new(
                    RichText::new(SAVE_ICON.to_string() + " Apply").font(header_font.clone()),
                )
                .fill(POSITIVE_BLUE);
                let revert_button =
                    Button::new(RichText::new(X_ICON.to_string() + " Revert").font(header_font))
                        .fill(NEGATIVE_RED);

                if ui.add_enabled(self.world_edited, revert_button).clicked() {
                    self.message(Message::RevertWorldChanges)
                }

                if ui.add_enabled(self.world_edited, apply_button).clicked() {
                    self.message(Message::ApplyWorldChanges);
                }
            });
            ui.separator();
        });
    }

    fn atmosphere_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Atmosphere")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("atmosphere_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.atmosphere.code,
                    TABLES.atmo_table[self.world.atmosphere.code as usize].composition
                ))
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for atmo in TABLES.atmo_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.atmosphere,
                                atmo.clone(),
                                format!(
                                    "{}: {}",
                                    atmo.code, TABLES.atmo_table[atmo.code as usize].composition
                                ),
                            )
                            .clicked()
                        {
                            self.message(Message::WorldModelUpdated);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldAtmosphere);
            }
        });
    }

    fn culture_display(&mut self, ui: &mut Ui) {
        ui.heading("Culture");
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            let code = self.world.culture.code as usize;
            ComboBox::from_id_source("culture_selection")
                .selected_text(&TABLES.culture_table[code].cultural_difference)
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for item in TABLES.culture_table.iter() {
                        let CulturalDiffRecord {
                            cultural_difference,
                            ..
                        } = &self.world.culture;

                        if ui
                            .selectable_label(
                                cultural_difference == &item.cultural_difference,
                                &item.cultural_difference,
                            )
                            .on_hover_text(&item.description)
                            .clicked()
                        {
                            self.message(Message::NewWorldCultureSelected {
                                new_code: item.code,
                            });
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldCulture);
            }
        });
        ui.add_space(LABEL_SPACING * 1.5);

        ui.label(
            RichText::new("Description")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ScrollArea::vertical()
            .id_source("culture_description")
            .max_height(ui.available_height() * 0.9)
            .show(ui, |ui| {
                ui.add(TextEdit::multiline(&mut self.world.culture.description));
            });
    }

    /** Tab displaying information about the culture and world tags of the `World`.

    This tab should be cut from any "player-safe" version of the app.
    */
    fn culture_errata_display(&mut self, ui: &mut Ui) {
        const NUM_COLUMNS: usize = World::NUM_TAGS + 1;
        ui.columns(NUM_COLUMNS, |columns| {
            self.culture_display(&mut columns[0]);

            self.world_tags_display(&mut columns[1..]);
        });
    }

    /** Tab displaying the non-government factions that exist on this `World`. */
    fn factions_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Factions")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.horizontal_top(|ui| {
            // Column of selectable labels, one for each faction.
            // This updates the selected `faction_idx` to control which is displayed to the right.
            ui.vertical(|ui| {
                ui.set_width(150.0);
                ScrollArea::vertical()
                    .id_source("faction_selection")
                    .show(ui, |ui| {
                        for (index, faction) in self.world.factions.iter().enumerate() {
                            ui.selectable_value(&mut self.faction_idx, index, &faction.name);
                        }
                        if ui.button("+").clicked() {
                            self.message(Message::AddNewFaction)
                        }
                    });
            });

            // Can't use an if-let here because that would require borrowing `self` and prevent
            // calling any methods.
            if self.world.factions.get(self.faction_idx).is_some() {
                // Display data about the selection faction in a column
                ui.vertical(|ui| {
                    ui.set_width(FIELD_SELECTION_WIDTH * 2.5);

                    // Regenerate and remove faction buttons
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(), |ui| {
                            let faction_removal_button = Button::new(
                                RichText::new(X_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)),
                            )
                            .fill(NEGATIVE_RED);

                            if ui
                                .add(faction_removal_button)
                                .on_hover_text_at_pointer("Double click to delete this faction")
                                .double_clicked()
                            {
                                self.message(Message::RemoveSelectedFaction);
                            }

                            // Regenerate faction button
                            if ui
                                .button(
                                    RichText::new(DICE_ICON)
                                        .font(FontId::proportional(BUTTON_FONT_SIZE)),
                                )
                                .clicked()
                            {
                                self.message(Message::RegenSelectedFaction);
                            }
                        });
                    });

                    // Faction name
                    ui.add(
                        TextEdit::singleline(&mut self.world.factions[self.faction_idx].name)
                            .desired_width(FIELD_SELECTION_WIDTH),
                    );
                    ui.add_space(LABEL_SPACING * 1.5);

                    ui.label(
                        RichText::new("Relative Strength")
                            .font(LABEL_FONT)
                            .color(LABEL_COLOR),
                    );
                    ui.add_space(LABEL_SPACING);

                    // Faction strength dropdown
                    let strength_code = self.world.factions[self.faction_idx].code as usize;
                    ComboBox::from_id_source("faction_strength_selection")
                        .selected_text(format!(
                            "{}: {}",
                            strength_code, TABLES.faction_table[strength_code].strength
                        ))
                        .width(FIELD_SELECTION_WIDTH)
                        .show_ui(ui, |ui| {
                            for faction in TABLES.faction_table.iter() {
                                let Faction { strength, .. } =
                                    &self.world.factions[self.faction_idx];

                                if ui
                                    .selectable_label(
                                        strength == &faction.strength,
                                        format!("{}: {}", faction.code, faction.strength),
                                    )
                                    .clicked()
                                {
                                    self.message(Message::NewFactionStrengthSelected {
                                        new_code: faction.code,
                                    });
                                }
                            }
                        });
                    ui.add_space(LABEL_SPACING * 1.5);

                    ui.label(
                        RichText::new("Leadership")
                            .font(LABEL_FONT)
                            .color(LABEL_COLOR),
                    );
                    ui.add_space(LABEL_SPACING);

                    // Faction government/leadership dropdown
                    let gov_code = self.world.factions[self.faction_idx].government.code as usize;
                    ComboBox::from_id_source("faction_government_selection")
                        .selected_text(format!("{}: {}", gov_code, TABLES.gov_table[gov_code].kind))
                        .width(FIELD_SELECTION_WIDTH)
                        .show_ui(ui, |ui| {
                            for gov in TABLES.gov_table.iter() {
                                let GovRecord {
                                    kind: fac_gov_kind, ..
                                } = &self.world.factions[self.faction_idx].government;

                                if ui
                                    .selectable_label(
                                        fac_gov_kind == &gov.kind,
                                        format!("{}: {}", gov.code, gov.kind),
                                    )
                                    .on_hover_text(&gov.description)
                                    .clicked()
                                {
                                    self.message(Message::NewFactionGovSelected {
                                        new_code: gov.code,
                                    });
                                }
                            }
                        });
                    ui.add_space(LABEL_SPACING * 1.5);

                    ui.label(
                        RichText::new("Description")
                            .font(LABEL_FONT)
                            .color(LABEL_COLOR),
                    );
                    ui.add_space(LABEL_SPACING);

                    // Description/notes on the faction and its leadership
                    ScrollArea::vertical()
                        .id_source("faction_description")
                        .max_height(ui.available_height() * 0.9)
                        .show(ui, |ui| {
                            let GovRecord { description, .. } =
                                &mut self.world.factions[self.faction_idx].government;
                            ui.add(TextEdit::multiline(description).desired_width(f32::INFINITY))
                        });
                });
            }
        });
    }

    fn government_display(&mut self, ui: &mut Ui) {
        ui.heading("Government");
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("government_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.government.code,
                    TABLES.gov_table[self.world.government.code as usize].kind
                ))
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for gov in TABLES.gov_table.iter() {
                        let GovRecord {
                            kind: world_gov_kind,
                            ..
                        } = &mut self.world.government;

                        if ui
                            .selectable_label(
                                world_gov_kind == &gov.kind,
                                format!("{}: {}", gov.code, gov.kind),
                            )
                            .on_hover_text(&gov.description)
                            .clicked()
                        {
                            self.message(Message::NewWorldGovSelected { new_code: gov.code });
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldGovernment);
            }
        });

        ui.add_space(LABEL_SPACING * 1.5);
        ui.label(
            RichText::new("Contraband")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.add(
            TextEdit::singleline(&mut self.world.government.contraband)
                .desired_width(FIELD_SELECTION_WIDTH),
        )
        .on_hover_text(format!(
            "Common contraband: {}",
            TABLES.gov_table[self.world.government.code as usize].contraband
        ));

        ui.add_space(LABEL_SPACING * 1.5);
        ui.label(
            RichText::new("Description")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ScrollArea::vertical()
            .id_source("government_description")
            .max_height(ui.available_height() * 0.9)
            .show(ui, |ui| {
                ui.add(TextEdit::multiline(&mut self.world.government.description));
            });
    }

    /** Tab displaying information about the government and law level of the `World`. */
    fn government_law_display(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            self.government_display(&mut columns[0]);
            self.law_level_display(&mut columns[1]);
        });
    }

    fn hydrographics_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Hydrographics")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("hydrographics_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.hydrographics.code,
                    TABLES.hydro_table[self.world.hydrographics.code as usize].description
                ))
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for hydro in TABLES.hydro_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.hydrographics,
                                hydro.clone(),
                                format!(
                                    "{}: {}",
                                    hydro.code, TABLES.hydro_table[hydro.code as usize].description
                                ),
                            )
                            .clicked()
                        {
                            self.message(Message::WorldModelUpdated);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldHydrographics);
            }
        });
    }

    fn law_level_display(&mut self, ui: &mut Ui) {
        ui.heading("Law Level");
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("law_level_selection")
                .selected_text(format!("{}", self.world.law_level.code))
                .width(SHORT_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for law_level in TABLES.law_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.law_level,
                                law_level.clone(),
                                law_level.code.to_string(),
                            )
                            .clicked()
                        {
                            self.message(Message::WorldModelUpdated);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldLawLevel);
            }
        });

        Grid::new("banned_equipment_grid")
            .spacing([FIELD_SPACING / 2.0, LABEL_SPACING])
            .min_col_width(FIELD_SELECTION_WIDTH)
            .max_col_width(FIELD_SELECTION_WIDTH)
            .striped(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Banned Weapons")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Banned Armor")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.end_row();

                let law_level = self.world.law_level.code as usize;
                for i in 0..=law_level {
                    ui.label(&TABLES.law_table[i].banned_weapons);
                    ui.label(&TABLES.law_table[i].banned_armor);
                    ui.end_row();
                }
            });
    }

    pub(crate) fn new_world_dialog(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            let height = ui.available_height();
            ui.add_space(height / 2.0);

            ui.heading(self.point.to_string());
            let header_font = TextStyle::Heading.resolve(&Style::default());
            let text = RichText::new("Add New World").font(header_font);
            if ui.button(text).clicked() {
                self.message(Message::AddNewWorld);
            }
        });
    }

    /** Tab displaying a large text area for writing notes about the `World`. */
    fn notes_display(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .id_source("world_notes")
            .max_height(ui.available_height() * 0.9)
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.world.notes)
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .margin(vec2(64.0, 32.0)),
                );
            });
    }

    fn planetary_data_display(&mut self, ui: &mut Ui) {
        ui.heading("Planetary Data");
        ui.add_space(LABEL_SPACING);

        self.size_display(ui);
        ui.add_space(FIELD_SPACING);

        self.atmosphere_display(ui);
        ui.add_space(FIELD_SPACING);

        self.temperature_display(ui);
        ui.add_space(FIELD_SPACING);

        self.hydrographics_display(ui);
        ui.add_space(FIELD_SPACING);

        self.population_display(ui);
        ui.add_space(FIELD_SPACING);

        self.tech_level_display(ui);
    }

    fn population_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Population")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("population_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.population.code,
                    TABLES.pop_table[self.world.population.code as usize].inhabitants
                ))
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for pop in TABLES.pop_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.population,
                                pop.clone(),
                                format!(
                                    "{}: {}",
                                    pop.code, TABLES.pop_table[pop.code as usize].inhabitants
                                ),
                            )
                            .clicked()
                        {
                            self.message(Message::WorldModelUpdated);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldPopulation);
            }
        });
    }

    /** Display a summarizing "profile" of the selected world.

    This profile shows the name, location, Universal World Profile (UWP), trade codes, travel code,
    and gas giant presence of the selected `World`. Also holds the `World` regeneration and
    removal buttons at the top right of the `world_data_display`.
    */
    fn profile_display(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // World name editor
            ui.add(TextEdit::singleline(&mut self.world.name).font(TextStyle::Heading));

            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add_space(FIELD_SPACING);
                let header_font = TextStyle::Heading.resolve(&Style::default());

                let world_removal_button =
                    Button::new(RichText::new(X_ICON).font(header_font.clone())).fill(NEGATIVE_RED);
                if ui.add(world_removal_button).clicked() {
                    self.message(Message::RemoveSelectedWorld);
                }

                // World regen button
                if ui
                    .button(RichText::new(DICE_ICON).font(header_font))
                    .clicked()
                {
                    self.message(Message::RegenSelectedWorld);
                }
            });
        });

        Grid::new("world_profile_grid")
            .spacing([FIELD_SPACING / 2.0, LABEL_SPACING])
            .min_col_width(100.0)
            .max_col_width(200.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Location")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.label(
                    RichText::new("World Profile")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Trade Codes")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Travel Code")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.end_row();

                // Location
                let response = ui.add(
                    TextEdit::singleline(&mut self.point_str).desired_width(SHORT_SELECTION_WIDTH),
                );

                if response.lost_focus() {
                    if ui.input().key_pressed(Key::Enter) {
                        self.message(Message::WorldLocUpdated);
                    } else {
                        self.point_str = self.point.to_string();
                    }
                }

                // World profile
                ui.label(self.world.profile_str());

                // Trade codes
                let response = ui.label(self.world.trade_code_str());
                let trade_code_long_str = self.world.trade_code_long_str();
                if !trade_code_long_str.is_empty() {
                    response.on_hover_text(trade_code_long_str);
                }

                // Travel Code
                ComboBox::from_id_source("travel_code_selection")
                    .selected_text(self.world.travel_code_str())
                    .show_ui(ui, |ui| {
                        for code in [TravelCode::Safe, TravelCode::Amber, TravelCode::Red] {
                            ui.selectable_value(
                                &mut self.world.travel_code,
                                code,
                                format!("{:?}", code),
                            );
                        }
                    });

                // Gas giant presence
                ui.checkbox(
                    &mut self.world.has_gas_giant,
                    RichText::new("Gas Giant Present")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
            });
    }

    fn size_display(&mut self, ui: &mut Ui) {
        Grid::new("world_size_grid")
            .spacing([FIELD_SPACING, LABEL_SPACING])
            .show(ui, |ui| {
                ui.label(RichText::new("Size").font(LABEL_FONT).color(LABEL_COLOR));
                ui.label(
                    RichText::new("Diameter (km)")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.end_row();

                // Size code
                ComboBox::from_id_source("size_selection")
                    .selected_text(self.world.size.to_string())
                    .width(SHORT_SELECTION_WIDTH)
                    .show_ui(ui, |ui| {
                        for size in World::SIZE_MIN..=World::SIZE_MAX {
                            if ui
                                .selectable_value(&mut self.world.size, size, size.to_string())
                                .clicked()
                            {
                                self.message(Message::WorldModelUpdated);
                            }
                        }
                    });

                // Diameter
                if ui
                    .add(
                        TextEdit::singleline(&mut self.diameter_str)
                            .desired_width(SHORT_SELECTION_WIDTH),
                    )
                    .lost_focus()
                {
                    self.message(Message::WorldDiameterUpdated);
                }

                if ui
                    .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                    .clicked()
                {
                    self.message(Message::RegenWorldSize);
                }
            });
    }

    fn starport_information_display(&mut self, ui: &mut Ui) {
        ui.heading("Starport Information");
        ui.add_space(LABEL_SPACING);

        ui.label(RichText::new("Class").font(LABEL_FONT).color(LABEL_COLOR));
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("starport_class_selection")
                .selected_text(self.world.starport.class.to_string())
                .width(SHORT_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    use StarportClass::*;
                    for starport_class in [A, B, C, D, E, X] {
                        let text = starport_class.to_string();
                        if ui
                            .selectable_value(&mut self.world.starport.class, starport_class, text)
                            .clicked()
                        {
                            self.message(Message::NewStarportClassSelected);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldStarport);
            }
        });
        ui.add_space(FIELD_SPACING);

        Grid::new("starport_grid")
            .spacing([FIELD_SPACING, LABEL_SPACING])
            .min_col_width(SHORT_SELECTION_WIDTH * 1.5)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Berthing Costs")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.label(RichText::new("Fuel").font(LABEL_FONT).color(LABEL_COLOR));
                ui.label(
                    RichText::new("Facilities")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );
                ui.end_row();

                if ui
                    .add(
                        TextEdit::singleline(&mut self.berthing_cost_str)
                            .desired_width(SHORT_SELECTION_WIDTH),
                    )
                    .lost_focus()
                {
                    self.message(Message::WorldBerthingCostsUpdated);
                }

                ui.label(&self.world.starport.fuel);
                ui.label(&self.world.starport.facilities);
            });
        ui.add_space(FIELD_SPACING);

        Grid::new("bases_grid")
            .spacing([FIELD_SPACING, LABEL_SPACING])
            .show(ui, |ui| {
                ui.label(RichText::new("Bases").font(LABEL_FONT).color(LABEL_COLOR));
                ui.end_row();

                ui.checkbox(
                    &mut self.world.has_naval_base,
                    RichText::new("Naval").font(LABEL_FONT).color(LABEL_COLOR),
                );

                ui.checkbox(
                    &mut self.world.has_scout_base,
                    RichText::new("Scout").font(LABEL_FONT).color(LABEL_COLOR),
                );

                ui.checkbox(
                    &mut self.world.has_research_base,
                    RichText::new("Research")
                        .font(LABEL_FONT)
                        .color(LABEL_COLOR),
                );

                ui.checkbox(
                    &mut self.world.has_tas,
                    RichText::new("TAS").font(LABEL_FONT).color(LABEL_COLOR),
                );
            });
    }

    /** Display a row of selectable "tabs" of data for the user to look through. */
    fn tab_labels(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for tab_label in TabLabel::ALL_VALUES {
                let text = tab_label.to_string();
                ui.selectable_value(&mut self.tab, tab_label, text);
            }
        });
    }

    fn tech_level_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Technology Level")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("tech_level_selection")
                .selected_text(self.world.tech_level.to_string())
                .width(SHORT_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for tech_level in World::TECH_MIN..=World::TECH_MAX {
                        if ui
                            .selectable_value(
                                &mut self.world.tech_level,
                                tech_level as u16,
                                tech_level.to_string(),
                            )
                            .clicked()
                        {
                            self.message(Message::WorldModelUpdated);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldTechLevel);
            }
        });
    }

    fn temperature_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Temperature")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        ui.add_space(LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("temperature_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.temperature.code,
                    TABLES.temp_table[self.world.temperature.code as usize].kind
                ))
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for temp in TABLES.temp_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.temperature,
                                temp.clone(),
                                format!(
                                    "{}: {}",
                                    temp.code, TABLES.temp_table[temp.code as usize].kind
                                ),
                            )
                            .clicked()
                        {
                            self.message(Message::WorldModelUpdated);
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldTemperature);
            }
        });
    }

    /** Displays information and fields associated with the selected `Point` and/or `World`.

    Contains and handles most of the data viewing and editing aspects of the app.
    Shows a summarizing `World` "profile" at the top and several, more detailed, selectable tabs
    beneath.
    */
    pub(crate) fn world_data_display(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.profile_display(ui);
            ui.add_space(FIELD_SPACING);

            self.tab_labels(ui);
            ui.separator();

            use TabLabel::*;
            match self.tab {
                WorldSurvey => self.world_survey_display(ui),
                GovernmentLaw => self.government_law_display(ui),
                Factions => self.factions_display(ui),
                CultureErrata => self.culture_errata_display(ui),
                Notes => self.notes_display(ui),
            }

            self.apply_revert_buttons(ui);
        });
    }

    /** Tab displaying `World` survey data such as info about the planetology and the starport. */
    fn world_survey_display(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            self.planetary_data_display(&mut columns[0]);
            self.starport_information_display(&mut columns[1]);
        });
    }

    fn world_tags_display(&mut self, columns: &mut [Ui]) {
        // In a perfect world, this would loop through the `Subsector::world_tags` array with
        // something like,
        //
        // `for (index, (column, world_tag)) in zip(columns, world_tags.iter_mut()).enumerate()`
        //
        // Unfortunately, Rust's borrowing rules will not allow mutably borrowing the
        // `world_tags` iterator and calling a method at the same time. The only way around this
        // would be to collect copies of the world tags into a temporary collection or to
        // heavily refactor the `Subsector` struct to allow for interior mutability with
        // `RefCell`.
        //
        // The length of `world_tags` isn't expected to ever grow, so this manual option works
        // for now. Refactoring for interior mutability would be a "nice-to-have" in the distant
        // future for several reasons, but copying arbitrarily long `description` strings into
        // a temporary collection is a no-go.
        let index = 0;
        columns[index].heading("World Tags");
        columns[index].add_space(LABEL_SPACING);
        columns[index].horizontal(|ui| {
            let code = self.world.world_tags[index].code as usize;
            ComboBox::from_id_source(format!("world_tag_{}_selection", index))
                .selected_text(&TABLES.world_tag_table[code].tag)
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for item in TABLES.world_tag_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.world_tags[index].tag,
                                item.tag.clone(),
                                &item.tag,
                            )
                            .clicked()
                        {
                            self.message(Message::NewWorldTagSelected {
                                index,
                                new_code: item.code,
                            })
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldTag { index });
            }
        });
        columns[index].add_space(LABEL_SPACING * 1.5);

        columns[index].label(
            RichText::new("Description")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        columns[index].add_space(LABEL_SPACING);

        ScrollArea::vertical()
            .id_source(format!("world_tag_{}_description", index))
            .max_height(columns[index].available_height() * 0.9)
            .show(&mut columns[index], |ui| {
                ui.add(TextEdit::multiline(
                    &mut self.world.world_tags[index].description,
                ));
            });

        let index = 1;
        // This is just to push down the rest of the column in line
        columns[index].heading("");
        columns[index].add_space(LABEL_SPACING);
        columns[index].horizontal(|ui| {
            let code = self.world.world_tags[index].code as usize;
            ComboBox::from_id_source(format!("world_tag_{}_selection", index))
                .selected_text(&TABLES.world_tag_table[code].tag)
                .width(FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for item in TABLES.world_tag_table.iter() {
                        if ui
                            .selectable_value(
                                &mut self.world.world_tags[index].tag,
                                item.tag.clone(),
                                &item.tag,
                            )
                            .clicked()
                        {
                            self.message(Message::NewWorldTagSelected {
                                index,
                                new_code: item.code,
                            })
                        }
                    }
                });

            if ui
                .button(RichText::new(DICE_ICON).font(FontId::proportional(BUTTON_FONT_SIZE)))
                .clicked()
            {
                self.message(Message::RegenWorldTag { index });
            }
        });
        columns[index].add_space(LABEL_SPACING * 1.5);

        columns[index].label(
            RichText::new("Description")
                .font(LABEL_FONT)
                .color(LABEL_COLOR),
        );
        columns[index].add_space(LABEL_SPACING);

        ScrollArea::vertical()
            .id_source(format!("world_tag_{}_description", index))
            .max_height(columns[index].available_height() * 0.9)
            .show(&mut columns[index], |ui| {
                ui.add(TextEdit::multiline(
                    &mut self.world.world_tags[index].description,
                ));
            });
    }
}
