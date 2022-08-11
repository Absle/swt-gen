mod popup;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use eframe::{App, Frame};

use egui::{
    menu, vec2, Align, Button, CentralPanel, Color32, ColorImage, ComboBox, Context, FontId, Grid,
    Image, Key, Label, Layout, Modifiers, Pos2, Rect, RichText, ScrollArea, Sense, Style, TextEdit,
    TextStyle, TopBottomPanel, Ui, Vec2,
};

use egui_extras::RetainedImage;

use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::astrography::{Point, Subsector};

use crate::astrography::table::{
    CulturalDiffRecord, GovRecord, StarportClass, WorldTagRecord, TABLES,
};

use crate::astrography::world::{Faction, TravelCode, World};

use popup::{ButtonPopup, Popup, SubsectorRegenPopup, SubsectorRenamePopup};

/** Set of messages respresenting all non-trivial GUI events. */
#[derive(Clone)]
pub(crate) enum Message {
    AddNewFaction,
    AddNewWorld,
    ApplyConfirmHexGridClicked { new_point: Point },
    ApplyWorldChanges,
    CancelHexGridClicked,
    CancelImportJson,
    CancelLocUpdate,
    CancelRegenSubsector,
    CancelRegenWorld,
    CancelRemoveWorld,
    CancelRenameSubsector,
    CancelUnsavedExit,
    ConfigRegenSubsector,
    ConfirmHexGridClicked { new_point: Point },
    ConfirmImportJson,
    ConfirmLocUpdate { location: Point },
    ConfirmRegenSubsector { world_abundance_dm: i16 },
    ConfirmRegenWorld,
    ConfirmRemoveWorld,
    ConfirmRenameSubsector { new_name: String },
    ConfirmUnsavedExit,
    ExportPlayerSafeSubsectorJson,
    ExportSubsectorMapSvg,
    HexGridClicked { new_point: Point },
    NewFactionGovSelected { new_code: u16 },
    NewStarportClassSelected,
    NewWorldCultureSelected { new_code: u16 },
    NewWorldGovSelected { new_code: u16 },
    NewWorldTagSelected { index: usize, new_code: u16 },
    Open,
    RedrawSubsectorImage,
    RegenSelectedFaction,
    RegenSelectedWorld,
    RegenSubsector,
    RegenWorldAtmosphere,
    RegenWorldCulture,
    RegenWorldGovernment,
    RegenWorldHydrographics,
    RegenWorldLawLevel,
    RegenWorldPopulation,
    RegenWorldSize,
    RegenWorldStarport,
    RegenWorldTag { index: usize },
    RegenWorldTechLevel,
    RegenWorldTemperature,
    RemoveSelectedFaction,
    RemoveSelectedWorld,
    RenameSubsector,
    RevertWorldChanges,
    Save,
    SaveAs,
    SaveConfigRegenSubsector,
    SaveConfirmImportJson,
    SaveExit,
    SubsectorModelUpdated,
    WorldBerthingCostsUpdated,
    WorldDiameterUpdated,
    WorldLocUpdated,
    WorldModelUpdated,
}

#[derive(PartialEq)]
enum TabLabel {
    WorldSurvey,
    GovernmentLaw,
    #[allow(dead_code)]
    Factions,
    #[allow(dead_code)]
    CultureErrata,
    Notes,
}

impl TabLabel {
    #[cfg(feature = "player-safe-gui")]
    const TAB_LABELS: [TabLabel; 3] = [Self::WorldSurvey, Self::GovernmentLaw, Self::Notes];

    #[cfg(not(feature = "player-safe-gui"))]
    const TAB_LABELS: [TabLabel; 5] = [
        Self::WorldSurvey,
        Self::GovernmentLaw,
        Self::Factions,
        Self::CultureErrata,
        Self::Notes,
    ];
}

impl ToString for TabLabel {
    fn to_string(&self) -> String {
        use TabLabel::*;
        match self {
            WorldSurvey => "World Survey".to_string(),
            GovernmentLaw => "Government & Law".to_string(),
            Factions => "Factions".to_string(),
            CultureErrata => "Culture & Errata".to_string(),
            Notes => "Notes".to_string(),
        }
    }
}

pub struct GeneratorApp {
    can_exit: bool,
    directory: String,
    filename: String,
    subsector: Subsector,
    subsector_edited: bool,
    subsector_image: RetainedImage,
    message_queue: VecDeque<Message>,
    /// List of blocking confirmation popups
    popup_queue: Vec<Box<dyn Popup>>,
    /// Selected display `TabLabel`
    tab: TabLabel,
    /// Whether a `Point` on the hex grid is currently selected or not
    point_selected: bool,
    /// Selected `Point`
    point: Point,
    /// Whether a `World` is at the selected `Point` or not
    world_selected: bool,
    world_edited: bool,
    /// Selected `World`
    world: World,
    /// Buffer for `String` representation of the selected world's `Point` location
    location: String,
    /// Buffer for `String` representation of the selected world's diameter in km
    diameter: String,
    /// Buffer for `String` representation of the selected world's starport berthing cost
    berthing_cost: String,
    /// Index of selected `Faction`
    faction_idx: usize,
}

impl GeneratorApp {
    const SUBSECTOR_IMAGE_MIN_SIZE: Vec2 = vec2(1584.0, 834.0);

    const LABEL_FONT: FontId = FontId::proportional(11.0);
    const LABEL_COLOR: Color32 = Color32::GRAY;
    const LABEL_SPACING: f32 = 4.0;

    /// Soft, nice blue color used by egui `SelectableValue` widgets.
    const POSITIVE_BLUE: Color32 = Color32::from_rgb(144, 209, 255);

    /// "Negative" red color meant to pair well with `POSITIVE_BLUE` aesthetically
    const NEGATIVE_RED: Color32 = Color32::from_rgb(255, 144, 144);

    const BUTTON_FONT_SIZE: f32 = 16.0;

    const FIELD_SPACING: f32 = 15.0;
    const FIELD_SELECTION_WIDTH: f32 = 225.0;
    const SHORT_SELECTION_WIDTH: f32 = 50.0;

    const DICE_ICON: &'static str = "🎲";
    const X_ICON: &'static str = "❌";
    const SAVE_ICON: &'static str = "💾";

    fn check_world_edited(&mut self) {
        self.world_edited = match self.subsector.get_world(&self.point) {
            Some(stored_world) => self.world != *stored_world,
            None => false,
        };
    }

    fn process_hotkeys(&mut self, ctx: &Context) {
        if ctx.input_mut().consume_key(Modifiers::CTRL, Key::N) {
            self.message(Message::RenameSubsector);
        } else if ctx.input_mut().consume_key(Modifiers::CTRL, Key::O) {
            self.message(Message::Open);
        } else if ctx.input_mut().consume_key(Modifiers::CTRL, Key::S) {
            self.message(Message::Save);
        } else if ctx
            .input_mut()
            .consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::S)
        {
            self.message(Message::SaveAs);
        }
    }

    fn unsaved_changes(&self) -> bool {
        self.subsector_edited || self.world_edited
    }

    /** Queue a message to be handled at the beginning of the next frame. */
    fn message(&mut self, message: Message) {
        self.message_queue.push_back(message);
    }

    /** Process all messages in the queue. */
    fn process_message_queue(&mut self) {
        while !self.message_queue.is_empty() {
            let message = self.message_queue.pop_front().unwrap();
            self.message_immediate(message);
        }
    }

    /** Handle a `Message` generated by a GUI event immediately.

    # Returns
    - `true`, if the message was completed successfully, or
    - `false`, otherwise
    */
    fn message_immediate(&mut self, message: Message) -> bool {
        use Message::*;
        match message {
            AddNewFaction => {
                self.world.factions.push(Faction::random());
                // Select the newly generated faction
                self.faction_idx = self.world.factions.len() - 1;
                self.message_immediate(Message::WorldModelUpdated);
            }

            AddNewWorld => {
                self.subsector.insert_random_world(&self.point);
                self.message_immediate(Message::ConfirmHexGridClicked {
                    new_point: self.point,
                });
                self.message_immediate(Message::SubsectorModelUpdated);
            }

            ApplyConfirmHexGridClicked { new_point } => {
                self.message_immediate(Message::ApplyWorldChanges);
                self.message_immediate(Message::ConfirmHexGridClicked { new_point });
            }

            ApplyWorldChanges => {
                if self.world_selected && self.world_edited {
                    self.subsector.insert_world(&self.point, &mut self.world);
                    self.message_immediate(Message::SubsectorModelUpdated);
                }
            }

            CancelHexGridClicked => {}
            CancelImportJson => {}

            CancelLocUpdate => {
                self.location = self.point.to_string();
            }

            CancelRegenSubsector => {}
            CancelRegenWorld => {}
            CancelRemoveWorld => {}
            CancelRenameSubsector => {}

            CancelUnsavedExit => self.can_exit = false,

            ConfigRegenSubsector => {
                let popup = SubsectorRegenPopup::default();
                self.add_popup(popup);
            }

            ConfirmHexGridClicked { new_point } => {
                self.point_selected = true;
                self.point = new_point;
                self.faction_idx = 0;
                let world = self.subsector.get_world(&self.point);
                if let Some(world) = world {
                    self.world_selected = true;
                    self.world = world.clone();
                    self.location = self.point.to_string();
                    self.diameter = self.world.diameter.to_string();
                    self.berthing_cost = self.world.starport.berthing_cost.to_string();
                } else {
                    self.world_selected = false;
                }
            }

            ConfirmImportJson => {
                let result = load_file_to_string(&self.directory, "JSON", &["json"]);

                let (path, json) = match result {
                    Ok(Some((path, json))) => (path, json),
                    Ok(None) => return false,
                    Err(err) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error: Failed to Read JSON")
                            .set_text(&format!("{}", err)[..])
                            .show_alert()
                            .unwrap();
                        return false;
                    }
                };

                let subsector = match Subsector::try_from_json(&json) {
                    Ok(subsector) => subsector,
                    Err(err) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error: Failed to Load Subsector from JSON")
                            .set_text(&format!("{}", err)[..])
                            .show_alert()
                            .unwrap();
                        return false;
                    }
                };

                let directory = path.parent().unwrap().to_str().unwrap().to_string();
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                *self = Self {
                    subsector,
                    directory,
                    filename,
                    ..Self::default()
                };
                self.message(Message::RedrawSubsectorImage);
            }

            ConfirmLocUpdate { location } => {
                self.subsector.move_world(&self.point, &location);
                self.point = location;
                self.location = self.point.to_string();
                self.message_immediate(Message::WorldModelUpdated);
                self.message_immediate(Message::SubsectorModelUpdated);
            }

            ConfirmRegenSubsector { world_abundance_dm } => {
                let directory = self.directory.clone();
                *self = Self {
                    subsector: Subsector::new(world_abundance_dm),
                    directory,
                    ..Self::default()
                };
                self.message(Message::RedrawSubsectorImage);
            }

            ConfirmRegenWorld => {
                self.subsector.insert_random_world(&self.point);
                self.world_selected = false;
                self.message_immediate(Message::ConfirmHexGridClicked {
                    new_point: self.point.clone(),
                });
                self.message_immediate(Message::SubsectorModelUpdated);
            }

            ConfirmRemoveWorld => {
                self.world_selected = false;
                self.subsector.remove_world(&self.point);
                self.message_immediate(Message::SubsectorModelUpdated);
            }

            ConfirmRenameSubsector { new_name } => {
                self.subsector.set_name(new_name);
                self.message_immediate(Message::SubsectorModelUpdated);
            }

            ConfirmUnsavedExit => self.can_exit = true,

            ExportPlayerSafeSubsectorJson => {
                let filename = format!("{}_Subsector-Player_Safe.json", self.subsector.name());
                let result = save_file_dialog(
                    &self.directory,
                    &filename,
                    "JSON",
                    &["json"],
                    self.subsector.player_safe().to_json(),
                );

                match result {
                    Ok(Some(_)) => (),
                    Ok(None) => (),
                    Err(err) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error: Failed to Save Player Safe JSON")
                            .set_text(&format!("{}", err)[..])
                            .show_alert()
                            .unwrap();
                    }
                }
            }

            ExportSubsectorMapSvg => {
                let filename = format!("{}_Subsector_Map.svg", self.subsector.name());
                let result = save_file_dialog(
                    &self.directory,
                    &filename,
                    "SVG",
                    &["svg"],
                    self.subsector.generate_svg(),
                );

                match result {
                    Ok(Some(_)) => (),
                    Ok(None) => (),
                    Err(err) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error: Failed to Save SVG")
                            .set_text(&format!("{}", err)[..])
                            .show_alert()
                            .unwrap();
                    }
                }
            }

            HexGridClicked { new_point } => {
                if self.world_edited {
                    let mut popup = ButtonPopup::new(
                        "Unapplied World Changes".to_string(),
                        format!(
                            "Do you want to apply your changes to '{}'?",
                            self.world.name
                        ),
                    );

                    popup.add_button(
                        "Apply".to_string(),
                        Message::ApplyConfirmHexGridClicked { new_point },
                    );
                    popup.add_button(
                        "Don't Apply".to_string(),
                        Message::ConfirmHexGridClicked { new_point },
                    );
                    popup.add_button("Cancel".to_string(), Message::CancelHexGridClicked);

                    self.add_popup(popup);
                } else {
                    self.message_immediate(Message::ConfirmHexGridClicked { new_point });
                }
            }

            NewFactionGovSelected { new_code } => {
                let fac_index = self.faction_idx;
                let old_code = self.world.factions[fac_index].government.code as usize;
                let old_description = &mut self.world.factions[fac_index].government.description;

                // Replace existing description iff the user hasn't changed it from the default
                if *old_description == TABLES.gov_table[old_code].description {
                    *old_description = TABLES.gov_table[new_code as usize].description.clone();
                }

                self.world.factions[fac_index].government.code = new_code;
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewStarportClassSelected => {
                let starport = TABLES
                    .starport_table
                    .iter()
                    .find(|starport| starport.class == self.world.starport.class)
                    .unwrap();

                self.world.starport.code = starport.code;
                self.world.generate_berthing_cost();
                self.berthing_cost = self.world.starport.berthing_cost.to_string();
                self.world.starport.fuel = starport.fuel.clone();
                self.world.starport.facilities = starport.facilities.clone();
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewWorldCultureSelected { new_code } => {
                let old_code = self.world.culture.code as usize;
                let old_description = &mut self.world.culture.description;

                // Replace existing description iff the user hasn't changed it from the default
                if *old_description == TABLES.culture_table[old_code].description {
                    *old_description = TABLES.culture_table[new_code as usize].description.clone();
                }

                self.world.culture.code = new_code;
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewWorldGovSelected { new_code } => {
                let old_code = self.world.government.code as usize;
                let old_description = &mut self.world.government.description;

                // Replace existing description iff the user hasn't changed it from the default
                if *old_description == TABLES.gov_table[old_code].description {
                    *old_description = TABLES.gov_table[new_code as usize].description.clone();
                }

                self.world.government.code = new_code;
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewWorldTagSelected { index, new_code } => {
                let world_tag = &mut self.world.world_tags[index];
                let old_code = world_tag.code as usize;

                world_tag.code = new_code;

                // Replace existing description iff the user hasn't changed it from the default
                if world_tag.description == TABLES.world_tag_table[old_code].description {
                    world_tag.description = TABLES.world_tag_table[new_code as usize]
                        .description
                        .clone();
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            Open => {
                if self.unsaved_changes() {
                    let popup = ButtonPopup::unsaved_changes_dialog(
                        format!(
                            "Do you want to save changes to Subsector {}?",
                            self.subsector.name()
                        ),
                        Message::SaveConfirmImportJson,
                        Message::ConfirmImportJson,
                        Message::CancelImportJson,
                    );
                    self.add_popup(popup);
                } else {
                    self.message(Message::ConfirmImportJson);
                }
            }

            RedrawSubsectorImage => {
                let subsector_svg = self.subsector.generate_svg();
                self.subsector_image =
                    generate_subsector_image(self.subsector.name(), &subsector_svg).unwrap();
            }

            RegenSelectedFaction => {
                let index = self.faction_idx;
                if let Some(faction) = self.world.factions.get_mut(index) {
                    let old_code = faction.government.code as usize;
                    let name = faction.name.clone();
                    let old_description = faction.government.description.clone();
                    *faction = Faction::random();

                    faction.name = name;
                    if old_description != TABLES.gov_table[old_code].description {
                        faction.government.description = old_description;
                    }
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenSelectedWorld => {
                let mut popup = ButtonPopup::new(
                    "Regenerating World".to_string(),
                    format!(
                        "Do you want to completely regenerate '{}'? This can not be undone.",
                        self.world.name
                    ),
                );
                popup.add_confirm_buttons(Message::ConfirmRegenWorld, Message::CancelRegenWorld);

                self.add_popup(popup);
            }

            RegenSubsector => {
                if self.unsaved_changes() {
                    let popup = ButtonPopup::unsaved_changes_dialog(
                        format!(
                            "Do you want to save changes to Subsector {}?",
                            self.subsector.name()
                        ),
                        Message::SaveConfigRegenSubsector,
                        Message::ConfigRegenSubsector,
                        Message::CancelRegenSubsector,
                    );

                    self.add_popup(popup);
                } else {
                    self.message(Message::ConfigRegenSubsector);
                }
            }

            RegenWorldAtmosphere => {
                self.world.generate_atmosphere();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldCulture => {
                let old_code = self.world.culture.code as usize;
                let old_description = self.world.culture.description.clone();
                self.world.generate_culture();

                if old_description != TABLES.culture_table[old_code].description {
                    self.world.culture.description = old_description;
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldGovernment => {
                let old_code = self.world.government.code as usize;
                let old_description = self.world.government.description.clone();
                let old_contraband = self.world.government.contraband.clone();
                self.world.generate_government();

                // If description or contraband have been changed from the default, keep them;
                // otherwise, allow them to be overwritten by the new government's default
                if old_description != TABLES.gov_table[old_code].description {
                    self.world.government.description = old_description;
                }
                if old_contraband != TABLES.gov_table[old_code].contraband {
                    self.world.government.contraband = old_contraband;
                }

                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldHydrographics => {
                self.world.generate_hydrographics();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldLawLevel => {
                self.world.generate_law_level();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldPopulation => {
                self.world.generate_population();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldSize => {
                self.world.generate_size();
                self.diameter = self.world.diameter.to_string();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldStarport => {
                self.world.generate_starport();
                self.berthing_cost = self.world.starport.berthing_cost.to_string();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldTag { index } => {
                let world_tag = &mut self.world.world_tags[index];
                let old_code = world_tag.code as usize;

                let new_tag = WorldTagRecord::random();
                world_tag.code = new_tag.code;
                world_tag.tag = new_tag.tag.clone();

                // Replace existing description iff the user hasn't changed it from the default
                if world_tag.description == TABLES.world_tag_table[old_code].description {
                    world_tag.description = new_tag.description.clone();
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldTechLevel => {
                self.world.generate_tech_level();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldTemperature => {
                self.world.generate_temperature();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RemoveSelectedFaction => {
                let index = &mut self.faction_idx;
                let factions = &mut self.world.factions;

                factions.remove(*index);

                if factions.len() == 0 {
                    *index = 0;
                } else if *index >= factions.len() {
                    *index = factions.len() - 1;
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            RemoveSelectedWorld => {
                let mut popup = ButtonPopup::new(
                    "Removing World".to_string(),
                    format!(
                        "Do you want to completely remove '{}'? This can not be undone.",
                        self.world.name
                    ),
                );
                popup.add_confirm_buttons(Message::ConfirmRemoveWorld, Message::CancelRemoveWorld);

                self.add_popup(popup);
            }

            RenameSubsector => {
                let popup = SubsectorRenamePopup::new(self.subsector.name());
                self.add_popup(popup);
            }

            RevertWorldChanges => {
                if self.world_selected {
                    if let Some(world) = self.subsector.get_world(&self.point) {
                        self.world = world.clone();
                    }
                }
            }

            Save => {
                // Make sure any unapplied changes the selected world are also saved
                self.message_immediate(Message::ApplyWorldChanges);

                let directory: &Path = self.directory.as_ref();
                let filename: &Path = self.filename.as_ref();
                let path = directory.join(filename);

                if self.filename.is_empty() || !path.exists() {
                    return self.message_immediate(Message::SaveAs);
                } else {
                    let result =
                        save_file(&self.directory, &self.filename, self.subsector.to_json());
                    match result {
                        Ok(()) => {
                            self.subsector_edited = false;
                        }
                        Err(err) => {
                            MessageDialog::new()
                                .set_type(MessageType::Error)
                                .set_title("Error: Failed to Save JSON")
                                .set_text(&format!("{}", err)[..])
                                .show_alert()
                                .unwrap();
                            return false;
                        }
                    }
                }
            }

            SaveAs => {
                // Make sure any unapplied changes the selected world are also saved
                self.message_immediate(Message::ApplyWorldChanges);

                let default_filename = format!("{}_Subsector.json", self.subsector.name());
                let filename = if !self.filename.is_empty() {
                    &self.filename
                } else {
                    &default_filename
                };

                let result = save_file_dialog(
                    &self.directory,
                    filename,
                    "JSON",
                    &["json"],
                    self.subsector.to_json(),
                );

                match result {
                    Ok(Some(path)) => {
                        self.directory = path.parent().unwrap().to_str().unwrap().to_string();
                        self.filename = path.file_name().unwrap().to_str().unwrap().to_string();
                        self.subsector_edited = false;
                    }
                    Ok(None) => return false,
                    Err(err) => {
                        MessageDialog::new()
                            .set_type(MessageType::Error)
                            .set_title("Error: Failed to Save JSON")
                            .set_text(&format!("{}", err)[..])
                            .show_alert()
                            .unwrap();
                        return false;
                    }
                }
            }

            SaveConfigRegenSubsector => {
                self.message_immediate(Message::Save);
                self.message_immediate(Message::ConfigRegenSubsector);
            }

            SaveConfirmImportJson => {
                let save_success = self.message_immediate(Message::Save);
                match save_success {
                    true => return self.message_immediate(Message::ConfirmImportJson),
                    false => return false,
                }
            }

            SaveExit => {
                self.message_immediate(Message::Save);
                self.can_exit = true;
            }

            SubsectorModelUpdated => {
                self.subsector_edited = true;
                self.message(Message::RedrawSubsectorImage);
            }

            WorldBerthingCostsUpdated => {
                if let Ok(berthing_cost) = self.berthing_cost.parse::<u32>() {
                    self.world.starport.berthing_cost = berthing_cost;
                } else {
                    self.berthing_cost = self.world.starport.berthing_cost.to_string();
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            WorldDiameterUpdated => {
                if let Ok(diameter) = self.diameter.parse::<u32>() {
                    self.world.diameter = diameter;
                } else {
                    self.diameter = self.world.diameter.to_string();
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            WorldLocUpdated => {
                let location = Point::try_from(&self.location[..]);
                if let Ok(location) = location {
                    if location == self.point {
                        self.location = self.point.to_string();
                        return true;
                    }

                    if let Some(world) = self.subsector.get_world(&location).clone() {
                        let mut popup = ButtonPopup::new(
                            "Destination Hex Occupied".to_string(),
                            format!(
                                "'{}' is already at {}.\nWould you like to overwrite it?",
                                world.name,
                                location.to_string()
                            ),
                        );
                        popup.add_confirm_buttons(
                            Message::ConfirmLocUpdate { location },
                            Message::CancelLocUpdate,
                        );

                        self.add_popup(popup);
                    } else {
                        self.message_immediate(Message::ConfirmLocUpdate { location });
                    }
                } else {
                    self.location = self.point.to_string();
                }
            }

            WorldModelUpdated => {
                self.world.resolve_trade_codes();
            }
        }

        true
    }

    /** Add a `Popup` to the queue to be shown and awaiting response. */
    fn add_popup<T: 'static + Popup>(&mut self, popup: T) {
        self.popup_queue.push(Box::new(popup));
    }

    /** Display all `Popup`'s in the queue and process any messages they return. */
    fn show_popups(&mut self, ctx: &Context) {
        let mut responded = Vec::new();
        for (i, popup) in self.popup_queue.iter_mut().enumerate() {
            if let Some(message) = popup.show(ctx) {
                responded.push((i, message));
            }
        }

        for (i, message) in responded {
            if self.popup_queue.get(i).is_some() {
                self.popup_queue.remove(i);
            }
            self.message(message);
        }
    }

    /** Displays the top panel of the app.

    Currently just a menu bar.
    */
    fn top_panel(&mut self, ctx: &Context) {
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
                            self.message(Message::Open);
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

    /** Handles displaying the overall central panel of the app.

    Shows the map of the `Subsector` on the left half of the panel and any information of the
    selected `Point` and/or `World` on the right half.
    If there is no `World` at the selected `Point`, it shows a button to add a new world at there.
    If there is a `World` there, displays the data associated with that `World`.
    */
    fn central_panel(&mut self, ctx: &Context) {
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

    /** Displays the `Subsector` map and handles any clicks or `Point` selections on it. */
    fn subsector_map_display(&mut self, ctx: &Context, ui: &mut Ui) {
        let max_size = ui.available_size();
        ui.set_min_size(Self::SUBSECTOR_IMAGE_MIN_SIZE);
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
                    self.message_immediate(Message::HexGridClicked { new_point });
                }
            }
        }
    }

    /** Displays information and fields associated with the selected `Point` and/or `World`.

    Contains and handles most of the data viewing and editing aspects of the app.
    Shows a summarizing `World` "profile" at the top and several, more detailed, selectable tabs
    beneath.
    */
    fn world_data_display(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.profile_display(ui);
            ui.add_space(Self::FIELD_SPACING);

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

    /** Displays a summarizing "profile" of the selected world.

    This profile shows the name, location, Universal World Profile (UWP), trade codes, travel code,
    and gas giant presence of the selected `World`. Also handles the `World` regeneration and
    removal buttons at the top right of the `world_data_display`.
    */
    fn profile_display(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(TextEdit::singleline(&mut self.world.name).font(TextStyle::Heading));
            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add_space(12.5);
                let header_font = TextStyle::Heading.resolve(&Style::default());
                let world_removal_button =
                    Button::new(RichText::new(Self::X_ICON).font(header_font.clone()))
                        .fill(Self::NEGATIVE_RED);
                if ui.add(world_removal_button).clicked() {
                    self.message(Message::RemoveSelectedWorld);
                }

                if ui
                    .button(RichText::new(Self::DICE_ICON).font(header_font))
                    .clicked()
                {
                    self.message(Message::RegenSelectedWorld);
                }
            });
        });

        Grid::new("world_profile_grid")
            .spacing([Self::FIELD_SPACING / 2.0, Self::LABEL_SPACING])
            .min_col_width(100.0)
            .max_col_width(200.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Location")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("World Profile")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Trade Codes")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Travel Code")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.end_row();

                // Location
                if ui
                    .add(
                        TextEdit::singleline(&mut self.location)
                            .desired_width(Self::SHORT_SELECTION_WIDTH),
                    )
                    .clicked()
                {
                    self.message(Message::WorldLocUpdated);
                }

                // World profile
                let profile = self.world.profile();
                if ui.add(Label::new(&profile).sense(Sense::click())).clicked() {
                    ui.output().copied_text = profile;
                }

                // Trade codes
                let trade_codes = self.world.trade_code_str();
                if ui
                    .add(Label::new(&trade_codes).sense(Sense::click()))
                    .clicked()
                {
                    ui.output().copied_text = trade_codes;
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
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
            });
    }

    /** Displays a row of selectable "tabs" of data for the user to look through. */
    fn tab_labels(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for tab_label in TabLabel::TAB_LABELS {
                let text = tab_label.to_string();
                ui.selectable_value(&mut self.tab, tab_label, text);
            }
        });
    }

    /** Tab displaying `World` survey data such as info about the planetology and the starport. */
    fn world_survey_display(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            self.planetary_data_display(&mut columns[0]);
            self.starport_information_display(&mut columns[1]);
        });
    }

    /** Tab displaying information about the government and law level of the `World`. */
    fn government_law_display(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            self.government_display(&mut columns[0]);
            self.law_level_display(&mut columns[1]);
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

    /** Tab displaying the non-government factions that exist on this `World`. */
    fn factions_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Factions")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

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
                    ui.set_width(Self::FIELD_SELECTION_WIDTH * 2.5);

                    // Regenerate and remove faction buttons
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(), |ui| {
                            let faction_removal_button = Button::new(
                                RichText::new(Self::X_ICON)
                                    .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                            )
                            .fill(Self::NEGATIVE_RED);

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
                                    RichText::new(Self::DICE_ICON)
                                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
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
                            .desired_width(Self::FIELD_SELECTION_WIDTH),
                    );
                    ui.add_space(Self::LABEL_SPACING * 1.5);

                    ui.label(
                        RichText::new("Relative Strength")
                            .font(Self::LABEL_FONT)
                            .color(Self::LABEL_COLOR),
                    );
                    ui.add_space(Self::LABEL_SPACING);

                    // Faction strength dropdown
                    let strength_code = self.world.factions[self.faction_idx].code as usize;
                    ComboBox::from_id_source("faction_strength_selection")
                        .selected_text(format!(
                            "{}: {}",
                            strength_code, TABLES.faction_table[strength_code].strength
                        ))
                        .width(Self::FIELD_SELECTION_WIDTH)
                        .show_ui(ui, |ui| {
                            for faction in TABLES.faction_table.iter() {
                                let Faction { code, strength, .. } =
                                    &mut self.world.factions[self.faction_idx];

                                if ui
                                    .selectable_value(
                                        strength,
                                        faction.strength.clone(),
                                        format!("{}: {}", faction.code, faction.strength),
                                    )
                                    .clicked()
                                {
                                    *code = faction.code;
                                }
                            }
                        });
                    ui.add_space(Self::LABEL_SPACING * 1.5);

                    ui.label(
                        RichText::new("Leadership")
                            .font(Self::LABEL_FONT)
                            .color(Self::LABEL_COLOR),
                    );
                    ui.add_space(Self::LABEL_SPACING);

                    // Faction government/leadership dropdown
                    let gov_code = self.world.factions[self.faction_idx].government.code as usize;
                    ComboBox::from_id_source("faction_government_selection")
                        .selected_text(format!("{}: {}", gov_code, TABLES.gov_table[gov_code].kind))
                        .width(Self::FIELD_SELECTION_WIDTH)
                        .show_ui(ui, |ui| {
                            for gov in TABLES.gov_table.iter() {
                                let GovRecord {
                                    kind: fac_gov_kind, ..
                                } = &mut self.world.factions[self.faction_idx].government;

                                if ui
                                    .selectable_value(
                                        fac_gov_kind,
                                        gov.kind.clone(),
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
                    ui.add_space(Self::LABEL_SPACING * 1.5);

                    ui.label(
                        RichText::new("Description")
                            .font(Self::LABEL_FONT)
                            .color(Self::LABEL_COLOR),
                    );
                    ui.add_space(Self::LABEL_SPACING);

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

    fn planetary_data_display(&mut self, ui: &mut Ui) {
        ui.heading("Planetary Data");
        ui.add_space(Self::LABEL_SPACING);

        self.size_display(ui);
        ui.add_space(Self::FIELD_SPACING);

        self.atmosphere_display(ui);
        ui.add_space(Self::FIELD_SPACING);

        self.temperature_display(ui);
        ui.add_space(Self::FIELD_SPACING);

        self.hydrographics_display(ui);
        ui.add_space(Self::FIELD_SPACING);

        self.population_display(ui);
        ui.add_space(Self::FIELD_SPACING);

        self.tech_level_display(ui);
    }

    fn size_display(&mut self, ui: &mut Ui) {
        Grid::new("world_size_grid")
            .spacing([Self::FIELD_SPACING, Self::LABEL_SPACING])
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Size")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Diameter (km)")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.end_row();

                // Size code
                ComboBox::from_id_source("size_selection")
                    .selected_text(self.world.size.to_string())
                    .width(Self::SHORT_SELECTION_WIDTH)
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
                        TextEdit::singleline(&mut self.diameter)
                            .desired_width(Self::SHORT_SELECTION_WIDTH),
                    )
                    .lost_focus()
                {
                    self.message(Message::WorldDiameterUpdated);
                }

                if ui
                    .button(
                        RichText::new(Self::DICE_ICON)
                            .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                    )
                    .clicked()
                {
                    self.message(Message::RegenWorldSize);
                }
            });
    }

    fn atmosphere_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Atmosphere")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("atmosphere_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.atmosphere.code,
                    TABLES.atmo_table[self.world.atmosphere.code as usize].composition
                ))
                .width(Self::FIELD_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldAtmosphere);
            }
        });
    }

    fn temperature_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Temperature")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("temperature_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.temperature.code,
                    TABLES.temp_table[self.world.temperature.code as usize].kind
                ))
                .width(Self::FIELD_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldTemperature);
            }
        });
    }

    fn hydrographics_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Hydrographics")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("hydrographics_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.hydrographics.code,
                    TABLES.hydro_table[self.world.hydrographics.code as usize].description
                ))
                .width(Self::FIELD_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldHydrographics);
            }
        });
    }

    fn population_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Population")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("population_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.population.code,
                    TABLES.pop_table[self.world.population.code as usize].inhabitants
                ))
                .width(Self::FIELD_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldPopulation);
            }
        });
    }

    fn tech_level_display(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Technology Level")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("tech_level_selection")
                .selected_text(self.world.tech_level.to_string())
                .width(Self::SHORT_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldTechLevel);
            }
        });
    }

    fn starport_information_display(&mut self, ui: &mut Ui) {
        ui.heading("Starport Information");
        ui.add_space(Self::LABEL_SPACING);

        ui.label(
            RichText::new("Class")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("starport_class_selection")
                .selected_text(self.world.starport.class.to_string())
                .width(Self::SHORT_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldStarport);
            }
        });
        ui.add_space(Self::FIELD_SPACING);

        Grid::new("starport_grid")
            .spacing([Self::FIELD_SPACING, Self::LABEL_SPACING])
            .min_col_width(Self::SHORT_SELECTION_WIDTH * 1.5)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Berthing Costs")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Fuel")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Facilities")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.end_row();

                if ui
                    .add(
                        TextEdit::singleline(&mut self.berthing_cost)
                            .desired_width(Self::SHORT_SELECTION_WIDTH),
                    )
                    .lost_focus()
                {
                    self.message(Message::WorldBerthingCostsUpdated);
                }

                ui.label(&self.world.starport.fuel);
                ui.label(&self.world.starport.facilities);
            });
        ui.add_space(Self::FIELD_SPACING);

        Grid::new("bases_grid")
            .spacing([Self::FIELD_SPACING, Self::LABEL_SPACING])
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Bases")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.end_row();

                ui.checkbox(
                    &mut self.world.has_naval_base,
                    RichText::new("Naval")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );

                ui.checkbox(
                    &mut self.world.has_scout_base,
                    RichText::new("Scout")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );

                ui.checkbox(
                    &mut self.world.has_research_base,
                    RichText::new("Research")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );

                ui.checkbox(
                    &mut self.world.has_tas,
                    RichText::new("TAS")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
            });
    }

    fn government_display(&mut self, ui: &mut Ui) {
        ui.heading("Government");
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("government_selection")
                .selected_text(format!(
                    "{}: {}",
                    self.world.government.code,
                    TABLES.gov_table[self.world.government.code as usize].kind
                ))
                .width(Self::FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for gov in TABLES.gov_table.iter() {
                        let GovRecord {
                            kind: world_gov_kind,
                            ..
                        } = &mut self.world.government;

                        if ui
                            .selectable_value(
                                world_gov_kind,
                                gov.kind.clone(),
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldGovernment);
            }
        });

        ui.add_space(Self::LABEL_SPACING * 1.5);
        ui.label(
            RichText::new("Contraband")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ui.add(
            TextEdit::singleline(&mut self.world.government.contraband)
                .desired_width(Self::FIELD_SELECTION_WIDTH),
        )
        .on_hover_text(format!(
            "Common contraband: {}",
            TABLES.gov_table[self.world.government.code as usize].contraband
        ));

        ui.add_space(Self::LABEL_SPACING * 1.5);
        ui.label(
            RichText::new("Description")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ScrollArea::vertical()
            .id_source("government_description")
            .max_height(ui.available_height() * 0.9)
            .show(ui, |ui| {
                ui.add(TextEdit::multiline(&mut self.world.government.description));
            });
    }

    fn law_level_display(&mut self, ui: &mut Ui) {
        ui.heading("Law Level");
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            ComboBox::from_id_source("law_level_selection")
                .selected_text(format!("{}", self.world.law_level.code))
                .width(Self::SHORT_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldLawLevel);
            }
        });

        Grid::new("banned_equipment_grid")
            .spacing([Self::FIELD_SPACING / 2.0, Self::LABEL_SPACING])
            .min_col_width(Self::FIELD_SELECTION_WIDTH)
            .max_col_width(Self::FIELD_SELECTION_WIDTH)
            .striped(true)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Banned Weapons")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
                );
                ui.label(
                    RichText::new("Banned Armor")
                        .font(Self::LABEL_FONT)
                        .color(Self::LABEL_COLOR),
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

    fn culture_display(&mut self, ui: &mut Ui) {
        ui.heading("Culture");
        ui.add_space(Self::LABEL_SPACING);

        ui.horizontal(|ui| {
            let code = self.world.culture.code as usize;
            ComboBox::from_id_source("culture_selection")
                .selected_text(&TABLES.culture_table[code].cultural_difference)
                .width(Self::FIELD_SELECTION_WIDTH)
                .show_ui(ui, |ui| {
                    for item in TABLES.culture_table.iter() {
                        let CulturalDiffRecord {
                            cultural_difference,
                            ..
                        } = &mut self.world.culture;

                        if ui
                            .selectable_value(
                                cultural_difference,
                                item.cultural_difference.clone(),
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldCulture);
            }
        });
        ui.add_space(Self::LABEL_SPACING * 1.5);

        ui.label(
            RichText::new("Description")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        ui.add_space(Self::LABEL_SPACING);

        ScrollArea::vertical()
            .id_source("culture_description")
            .max_height(ui.available_height() * 0.9)
            .show(ui, |ui| {
                ui.add(TextEdit::multiline(&mut self.world.culture.description));
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
        columns[index].add_space(Self::LABEL_SPACING);
        columns[index].horizontal(|ui| {
            let code = self.world.world_tags[index].code as usize;
            ComboBox::from_id_source(format!("world_tag_{}_selection", index))
                .selected_text(&TABLES.world_tag_table[code].tag)
                .width(Self::FIELD_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldTag { index });
            }
        });
        columns[index].add_space(Self::LABEL_SPACING * 1.5);

        columns[index].label(
            RichText::new("Description")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        columns[index].add_space(Self::LABEL_SPACING);

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
        columns[index].add_space(Self::LABEL_SPACING);
        columns[index].horizontal(|ui| {
            let code = self.world.world_tags[index].code as usize;
            ComboBox::from_id_source(format!("world_tag_{}_selection", index))
                .selected_text(&TABLES.world_tag_table[code].tag)
                .width(Self::FIELD_SELECTION_WIDTH)
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
                .button(
                    RichText::new(Self::DICE_ICON)
                        .font(FontId::proportional(Self::BUTTON_FONT_SIZE)),
                )
                .clicked()
            {
                self.message(Message::RegenWorldTag { index });
            }
        });
        columns[index].add_space(Self::LABEL_SPACING * 1.5);

        columns[index].label(
            RichText::new("Description")
                .font(Self::LABEL_FONT)
                .color(Self::LABEL_COLOR),
        );
        columns[index].add_space(Self::LABEL_SPACING);

        ScrollArea::vertical()
            .id_source(format!("world_tag_{}_description", index))
            .max_height(columns[index].available_height() * 0.9)
            .show(&mut columns[index], |ui| {
                ui.add(TextEdit::multiline(
                    &mut self.world.world_tags[index].description,
                ));
            });
    }

    fn apply_revert_buttons(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
            ui.add_space(12.5);
            ui.horizontal(|ui| {
                ui.add_space(12.5);

                let header_font = TextStyle::Heading.resolve(&Style::default());
                let apply_button = Button::new(
                    RichText::new(Self::SAVE_ICON.to_string() + " Apply").font(header_font.clone()),
                )
                .fill(Self::POSITIVE_BLUE);
                let revert_button = Button::new(
                    RichText::new(Self::X_ICON.to_string() + " Revert").font(header_font),
                )
                .fill(Self::NEGATIVE_RED);

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

    fn new_world_dialog(&mut self, ui: &mut Ui) {
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
}

impl Default for GeneratorApp {
    fn default() -> Self {
        let subsector = Subsector::default();
        let subsector_svg = subsector.generate_svg();
        let subsector_image = generate_subsector_image(subsector.name(), &subsector_svg).unwrap();

        Self {
            can_exit: false,
            directory: "~".to_string(),
            filename: String::new(),
            subsector,
            subsector_edited: false,
            subsector_image,
            message_queue: VecDeque::new(),
            popup_queue: Vec::new(),
            point_selected: false,
            world_selected: false,
            world_edited: false,
            point: Point::default(),
            world: World::empty(),
            tab: TabLabel::WorldSurvey,
            faction_idx: 0,
            location: String::new(),
            diameter: String::new(),
            berthing_cost: String::new(),
        }
    }
}

impl App for GeneratorApp {
    fn on_exit_event(&mut self) -> bool {
        let can_exit = !self.unsaved_changes() || self.can_exit;
        if !can_exit {
            let popup = ButtonPopup::unsaved_changes_dialog(
                format!(
                    "Do you want to save changes to Subsector {}?",
                    self.subsector.name()
                ),
                Message::SaveExit,
                Message::ConfirmUnsavedExit,
                Message::CancelUnsavedExit,
            );

            self.add_popup(popup);
        }
        can_exit
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.can_exit {
            frame.quit();
        }

        self.check_world_edited();
        self.process_hotkeys(ctx);

        // GUI elements
        self.process_message_queue();
        self.top_panel(ctx);
        self.central_panel(ctx);
        self.show_popups(ctx);
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

# Errors
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

/** Save `contents` directly to the file described by `directory` and `filename` *without* a dialog.

# Returns
- `Err` if there was an error while trying to write to the file
- `Ok(())` if the file was successfully written to
*/
fn save_file<P, C>(
    directory: &P,
    filename: &P,
    contents: C,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let directory: &Path = directory.as_ref();
    let filename: &Path = filename.as_ref();
    let path = directory.join(filename);
    std::fs::write(path, contents)?;
    Ok(())
}

/** Open a `FileDialog` and save `contents` to the selected file.

# Arguments
- `directory`: Directory to which the `FileDialog` initially opens
- `filename`: Filename to be pre-filled into the `FileDialog`
- `description`: Description of the file type to be filtered
- `extensions`: Array of file extensions to filter
- `contents`: Contents of the file to write to the file system

# Returns
- `Err` if there was an error while trying to save the file
- `Ok(save_file)` with the path to the selected file if it was able to save successfully
- `Ok(None)` if there was no error but no directory was selected and no save occurred; usually means
  the "Cancel" button was selected
*/
fn save_file_dialog<P, C>(
    directory: &P,
    filename: &str,
    description: &str,
    extensions: &[&str],
    contents: C,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = FileDialog::new()
        .set_location(directory)
        .set_filename(filename)
        .add_filter(description, extensions)
        .show_save_single_file()?;

    let save_path = match path {
        Some(path) => {
            std::fs::write(path.clone(), contents)?;
            Some(path)
        }

        None => None,
    };

    Ok(save_path)
}

/** Open a `FileDialog` and read in the selected file.

# Arguments
- `directory`: Directory to which the `FileDialog` initially opens
- `description`: Description of the file type to be filtered
- `extensions`: Array of file extensions to filter

# Returns
- `Err` if there was an error while trying to read the file
- `Ok((loaded_path, contents))` with the path to the loaded file if it was able to save successfully
- `Ok(None)` if there was no error but no file was selected and no contents were loaded; usually
  means the "Cancel" button was selected
*/
fn load_file_to_string<P: AsRef<Path>>(
    directory: &P,
    description: &str,
    extensions: &[&str],
) -> Result<Option<(PathBuf, String)>, Box<dyn std::error::Error>> {
    let path = FileDialog::new()
        .set_location(directory)
        .add_filter(description, extensions)
        .show_open_single_file()?;

    let loaded_file = match path {
        Some(path) => {
            let contents = std::fs::read_to_string(path.clone())?;
            Some((path, contents))
        }
        None => None,
    };

    Ok(loaded_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod message_tests {
        use super::*;

        fn empty_app() -> GeneratorApp {
            GeneratorApp {
                subsector: Subsector::empty(),
                ..Default::default()
            }
        }

        #[test]
        fn add_new_faction() {
            let mut app = GeneratorApp::default();
            let occupied_points: Vec<_> = app.subsector.get_map().keys().cloned().collect();
            assert!(occupied_points.get(0).is_some());
            let point = occupied_points[0];
            app.message_immediate(Message::HexGridClicked { new_point: point });
            match app.subsector.get_world(&point) {
                Some(world) => assert_eq!(app.world.factions, world.factions),
                None => panic!("Empty point got in somehow"),
            }

            app.message_immediate(Message::AddNewFaction);
            match app.subsector.get_world(&point) {
                Some(world) => {
                    assert_ne!(app.world.factions, world.factions);
                    assert_eq!(app.world.factions.len(), world.factions.len() + 1);
                }
                None => panic!("Empty point got in somehow"),
            }
        }

        #[test]
        fn add_new_world() {
            let mut app = empty_app();

            let unoccupied_point = Point { x: 1, y: 1 };
            assert!(app.subsector.get_world(&unoccupied_point).is_none());

            app.message_immediate(Message::HexGridClicked {
                new_point: unoccupied_point,
            });
            assert!(app.point_selected);
            assert_eq!(app.point, unoccupied_point);
            assert!(!app.world_selected);

            app.message_immediate(Message::AddNewWorld);
            assert!(app.subsector.get_world(&unoccupied_point).is_some());
            assert!(app.point_selected);
            assert_eq!(app.point, unoccupied_point);
            assert!(app.world_selected);

            assert!(app.unsaved_changes());
        }

        #[test]
        fn apply_world_changes() {
            let mut app = empty_app();
            let point = Point { x: 1, y: 1 };
            assert!(app.subsector.get_world(&point).is_none());

            app.message_immediate(Message::HexGridClicked { new_point: point });
            app.message_immediate(Message::AddNewWorld);
            app.check_world_edited();
            assert!(app.subsector.get_world(&point).is_some());
            assert_eq!(app.world, *app.subsector.get_world(&point).unwrap());
            assert!(!app.world_edited);

            let blah = "Blah blah blah".to_string();
            app.world.notes = blah.clone();
            app.check_world_edited();
            assert_ne!(app.world, *app.subsector.get_world(&point).unwrap());
            assert!(app.world_edited);

            app.message_immediate(Message::ApplyWorldChanges);
            app.check_world_edited();
            assert_eq!(app.world, *app.subsector.get_world(&point).unwrap());
            assert!(!app.world_edited);
        }

        #[test]
        fn hex_grid_clicked() {
            let mut app = GeneratorApp::default();

            // Test hex clicking on all points with no world changes
            for x in 1..=Subsector::COLUMNS {
                for y in 1..=Subsector::ROWS {
                    let point = Point {
                        x: x as u16,
                        y: y as u16,
                    };

                    app.message_immediate(Message::HexGridClicked { new_point: point });
                    assert!(app.point_selected);
                    assert_eq!(app.point, point);
                    match app.subsector.get_world(&point) {
                        Some(world) => {
                            assert!(app.world_selected);
                            assert_eq!(app.world, *world);
                            assert_eq!(app.location, point.to_string());
                            assert_eq!(app.diameter, world.diameter.to_string());
                            assert_eq!(app.berthing_cost, world.starport.berthing_cost.to_string());
                        }

                        None => {
                            assert!(!app.world_selected);
                        }
                    }
                }
            }

            // Test hex clicking after making changes to selected world
            let occupied_points: Vec<_> = app.subsector.get_map().keys().cloned().collect();
            assert!(occupied_points.get(0).is_some());
            let point = occupied_points[0];
            assert!(app.subsector.get_world(&point).is_some());

            let other_x = if point.x == Subsector::COLUMNS as u16 {
                point.x - 1
            } else {
                point.x + 1
            };

            let other_y = if point.y == Subsector::ROWS as u16 {
                point.y - 1
            } else {
                point.y + 1
            };

            let new_point = Point {
                x: other_x,
                y: other_y,
            };

            let blah = "Blah blah blah blah".to_string();

            app.message_immediate(Message::HexGridClicked { new_point: point });

            // Just making some/any change to the now selected world
            app.world.notes = blah.clone();
            app.check_world_edited();
            assert!(app.world_edited);

            app.message_immediate(Message::HexGridClicked { new_point });
            assert!(app.popup_queue.get(0).is_some());
            app.popup_queue.remove(0);

            // Nothing should change if the "cancel" button was hit on the popup
            app.message_immediate(Message::CancelHexGridClicked);
            assert_eq!(app.point, point);

            // Repeat as if the user had pressed the "don't apply" button
            app.message_immediate(Message::HexGridClicked { new_point });
            assert!(app.popup_queue.get(0).is_some());
            app.popup_queue.remove(0);

            app.message_immediate(Message::ConfirmHexGridClicked { new_point });
            assert_eq!(app.point, new_point);

            app.check_world_edited();
            assert!(!app.world_edited);

            // Confirm that the change was not kept
            app.message_immediate(Message::HexGridClicked { new_point: point });
            assert_eq!(app.world.notes, String::new());

            // Repeat as if the "apply" button had been pressed
            app.world.notes = blah.clone();
            app.check_world_edited();
            assert!(app.world_edited);

            app.message_immediate(Message::HexGridClicked { new_point });
            assert!(app.popup_queue.get(0).is_some());
            app.popup_queue.remove(0);
            app.message_immediate(Message::ApplyConfirmHexGridClicked { new_point });
            assert_eq!(app.point, new_point);

            app.check_world_edited();
            assert!(!app.world_edited);

            // Confirm that the change was kept
            assert_eq!(app.subsector.get_world(&point).unwrap().notes, blah);
        }
    }
}
