mod gui;
mod pipe;

use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use eframe::{App, Frame};
use egui::{Context, Key, Modifiers};
use egui_extras::RetainedImage;
use native_dialog::{FileDialog, MessageDialog, MessageType};

use crate::astrography::{
    table::TABLES,
    world::{Faction, World},
    Point, Subsector,
};

use gui::popup::Popup;

// TODO: calls to `Subsector::generate_svg` using this variable need to have their logic of when to
// have the svg colored updated once proper svg coloring has been implemented. This `const` is just
// part of the proof of concept commit; set to true to have the hexes of generated svg's be rainbow
// colored. Make sure to commit only with this set to `false`.
const COLORED: bool = false;

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
    ConfirmRemoveWorld { point: Point },
    ConfirmRenameSubsector { new_name: String },
    ConfirmUnsavedExit,
    ExportPlayerSafeSubsectorJson,
    ExportSubsectorMapSvg,
    HexGridClicked { new_point: Point },
    NewFactionGovSelected { new_code: u16 },
    NewFactionStrengthSelected { new_code: u16 },
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

pub struct GeneratorApp {
    /// Buffer for `String` representation of the selected world's starport berthing cost
    berthing_cost: String,
    /// Flag used to ensure the program is not closed without a save prompt
    can_exit: bool,
    /// Buffer for `String` representation of the selected world's diameter in km
    diameter: String,
    /// Index of selected [`Faction`]
    faction_idx: usize,
    /// Receive internal and external messages
    message_rx: pipe::Receiver<Message>,
    /// Send internal and external messages; cloned by external GUI structs (e.g. [`Popups`]s)
    message_tx: pipe::Sender<Message>,
    /// Currently selected [`Point`] on the hex grid
    point: Point,
    /// Whether a [`Point`] on the hex grid is currently selected or not
    point_selected: bool,
    /// Buffer for `String` representation of the selected world's [`Point`] location
    point_str: String,
    /// List of blocking popups
    popup_queue: Vec<Box<dyn Popup>>,
    /// Path to directory that was last saved to
    save_directory: String,
    /// Name of the file that was last saved to
    save_filename: String,
    subsector: Subsector,
    /// Whether the loaded [`Subsector`] has unsaved changes
    subsector_edited: bool,
    /// Image of the subsector map, rasterized from the generated svg
    subsector_image: Option<RetainedImage>,
    /// Whether the loaded [`Subsector`]'s name changed and the app window needs a title update
    subsector_name_changed: bool,
    /// Selected display [`TabLabel`]
    tab: gui::TabLabel,
    /// `Receiver` for the subsector image worker thread
    worker_rx: mpsc::Receiver<RetainedImage>,
    /// `Sender` for the subsector image worker thread
    worker_tx: mpsc::Sender<String>,
    /// Selected `World`
    world: World,
    /// Whether the selected [`World`] has unapplied changes
    world_edited: bool,
    /// Whether a [`World`] is at the selected [`Point`] or not
    world_selected: bool,
}

impl GeneratorApp {
    fn empty() -> Self {
        let subsector = Subsector::empty();
        let (message_tx, message_rx) = pipe::channel();

        let (worker_tx, boss_rx) = mpsc::channel::<String>();
        let (boss_tx, worker_rx) = mpsc::channel::<RetainedImage>();

        // Spawn worker thread to process SVG asynchronously
        thread::spawn(move || loop {
            while let Ok(svg) = boss_rx.recv() {
                match boss_tx.send(gui::generate_subsector_image(svg)) {
                    Ok(_) => (),
                    Err(_) => break,
                }
            }
        });

        Self {
            berthing_cost: String::new(),
            can_exit: false,
            diameter: String::new(),
            faction_idx: 0,
            message_rx,
            message_tx,
            point: Point::default(),
            point_selected: false,
            point_str: String::new(),
            popup_queue: Vec::new(),
            save_directory: "~".to_string(),
            save_filename: String::new(),
            subsector,
            subsector_edited: false,
            subsector_image: None,
            subsector_name_changed: true,
            tab: gui::TabLabel::WorldSurvey,
            worker_rx,
            worker_tx,
            world: World::empty(),
            world_edited: false,
            world_selected: false,
        }
    }

    fn check_world_edited(&mut self) {
        self.world_edited = match self.subsector.get_world(&self.point) {
            Some(stored_world) => self.world != *stored_world,
            None => false,
        };
    }

    fn process_hotkeys(&mut self, ctx: &Context) {
        let hotkeys = [
            (Modifiers::CTRL, Key::N, Message::RenameSubsector),
            (Modifiers::CTRL, Key::O, Message::Open),
            (Modifiers::CTRL, Key::S, Message::Save),
            (Modifiers::CTRL | Modifiers::SHIFT, Key::S, Message::SaveAs),
        ];

        for (modifiers, key, message) in hotkeys {
            if ctx.input_mut().consume_key(modifiers, key) {
                self.message(message);
            }
        }
    }

    fn has_unsaved_changes(&self) -> bool {
        self.subsector_edited || self.world_edited
    }

    /** Queue a message to be handled at the beginning of the next frame. */
    fn message(&self, message: Message) {
        self.message_tx.send(message);
    }

    /** Process all messages in the queue. */
    fn process_message_queue(&mut self) {
        while !self.message_rx.is_empty() {
            let message = self.message_rx.receive().unwrap();
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
                self.faction_idx = self.world.add_faction();
                self.message_immediate(Message::WorldModelUpdated);
            }

            AddNewWorld => match self.subsector.insert_random_world(&self.point) {
                Ok(_) => {
                    self.message_immediate(Message::ConfirmHexGridClicked {
                        new_point: self.point,
                    });
                    self.message_immediate(Message::SubsectorModelUpdated);
                }
                Err(_) => return false,
            },

            ApplyConfirmHexGridClicked { new_point } => {
                self.message_immediate(Message::ApplyWorldChanges);
                self.message_immediate(Message::ConfirmHexGridClicked { new_point });
            }

            ApplyWorldChanges => {
                if self.world_selected && self.world_edited {
                    match self.subsector.insert_world(&self.point, self.world.clone()) {
                        Ok(_) => {
                            self.message_immediate(Message::SubsectorModelUpdated);
                        }
                        Err(_) => return false,
                    }
                }
            }

            CancelHexGridClicked => {}
            CancelImportJson => {}

            CancelLocUpdate => {
                self.point_str = self.point.to_string();
            }

            CancelRegenSubsector => {}
            CancelRegenWorld => {}
            CancelRemoveWorld => {}
            CancelRenameSubsector => {}

            CancelUnsavedExit => self.can_exit = false,

            ConfigRegenSubsector => {
                self.subsector_regen_popup();
            }

            ConfirmHexGridClicked { new_point } => {
                self.point_selected = true;
                self.point = new_point;
                self.faction_idx = 0;
                let world = self.subsector.get_world(&self.point);
                if let Some(world) = world {
                    self.world_selected = true;
                    self.world = world.clone();
                    self.point_str = self.point.to_string();
                    self.diameter = self.world.diameter.to_string();
                    self.berthing_cost = self.world.starport.berthing_cost.to_string();
                } else {
                    self.world_selected = false;
                }
            }

            ConfirmImportJson => {
                let result = load_file_to_string(&self.save_directory, "JSON", &["json"]);

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
                    save_directory: directory,
                    save_filename: filename,
                    ..Self::from(subsector)
                };
            }

            ConfirmLocUpdate { location } => {
                let success = match self.subsector.move_world(&self.point, &location) {
                    Ok(_) => {
                        self.point = location;
                        self.message_immediate(Message::WorldModelUpdated);
                        self.message_immediate(Message::SubsectorModelUpdated);
                        true
                    }

                    Err(_) => false,
                };
                self.point_str = self.point.to_string();
                return success;
            }

            ConfirmRegenSubsector { world_abundance_dm } => {
                let directory = self.save_directory.clone();
                *self = Self {
                    save_directory: directory,
                    ..Self::with_world_abundance(world_abundance_dm)
                };
            }

            ConfirmRegenWorld => match self.subsector.insert_random_world(&self.point) {
                Ok(_) => {
                    self.world_selected = false;
                    self.message_immediate(Message::ConfirmHexGridClicked {
                        new_point: self.point,
                    });
                    self.message_immediate(Message::SubsectorModelUpdated);
                }
                Err(_) => return false,
            },

            ConfirmRemoveWorld { point } => {
                self.world_selected = false;
                match self.subsector.remove_world(&point) {
                    Ok(_) => {
                        self.message_immediate(Message::SubsectorModelUpdated);
                    }
                    Err(_) => return false,
                }
            }

            ConfirmRenameSubsector { new_name } => {
                self.subsector.set_name(new_name);
                self.subsector_name_changed = true;
                self.message_immediate(Message::SubsectorModelUpdated);
            }

            ConfirmUnsavedExit => self.can_exit = true,

            ExportPlayerSafeSubsectorJson => {
                let filename = format!("{} Subsector Player-Safe.json", self.subsector.name());
                let result = save_file_dialog(
                    &self.save_directory,
                    &filename,
                    "JSON",
                    &["json"],
                    self.subsector.copy_player_safe().to_json(),
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
                let filename = format!("{} Subsector Map.svg", self.subsector.name());
                let result = save_file_dialog(
                    &self.save_directory,
                    &filename,
                    "SVG",
                    &["svg"],
                    self.subsector.generate_svg(COLORED),
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
                    self.unapplied_world_popup(new_point);
                } else {
                    self.message_immediate(Message::ConfirmHexGridClicked { new_point });
                }
            }

            NewFactionGovSelected { new_code } => {
                if let Some(faction) = self.world.factions.get_mut(self.faction_idx) {
                    faction
                        .government
                        .safe_mutate(&TABLES.gov_table[new_code as usize]);
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewFactionStrengthSelected { new_code } => {
                if let Some(faction) = self.world.factions.get_mut(self.faction_idx) {
                    let faction_strength = &TABLES.faction_table[new_code as usize];
                    faction.code = faction_strength.code;
                    faction.strength = faction_strength.strength.clone();
                }
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
                self.world
                    .culture
                    .safe_mutate(&TABLES.culture_table[new_code as usize]);
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewWorldGovSelected { new_code } => {
                self.world
                    .government
                    .safe_mutate(&TABLES.gov_table[new_code as usize]);
                self.message_immediate(Message::WorldModelUpdated);
            }

            NewWorldTagSelected { index, new_code } => {
                if let Some(tag) = self.world.world_tags.get_mut(index) {
                    tag.safe_mutate(&TABLES.world_tag_table[new_code as usize]);
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            Open => {
                if self.has_unsaved_changes() {
                    self.unsaved_subsector_reload_popup();
                } else {
                    self.message(Message::ConfirmImportJson);
                }
            }

            RedrawSubsectorImage => {
                let svg = self.subsector.generate_svg(COLORED);
                self.worker_tx
                    .send(svg)
                    .expect("Subsector map worker thread should never hang up.");
            }

            RegenSelectedFaction => {
                let index = self.faction_idx;
                if let Some(faction) = self.world.factions.get_mut(index) {
                    let mut old_gov = faction.government.clone();
                    let name = faction.name.clone();
                    *faction = Faction::random();

                    faction.name = name;
                    old_gov.safe_mutate(&faction.government);
                    faction.government = old_gov;
                }
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenSelectedWorld => {
                self.regen_world_popup();
            }

            RegenSubsector => {
                if self.has_unsaved_changes() {
                    self.unsaved_subsector_regen_popup();
                } else {
                    self.message(Message::ConfigRegenSubsector);
                }
            }

            RegenWorldAtmosphere => {
                self.world.generate_atmosphere();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldCulture => {
                let mut old_culture = self.world.culture.clone();
                self.world.generate_culture();
                old_culture.safe_mutate(&self.world.culture);
                self.world.culture = old_culture;
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldGovernment => {
                let mut old_gov = self.world.government.clone();
                self.world.generate_government();
                old_gov.safe_mutate(&self.world.government);
                self.world.government = old_gov;
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

            RegenWorldTag { index } => match self.world.generate_world_tag(index) {
                Some(mut old_tag) => {
                    old_tag.safe_mutate(&self.world.world_tags[index]);
                    self.world.world_tags[index] = old_tag;
                    self.message_immediate(Message::WorldModelUpdated);
                }

                None => (),
            },

            RegenWorldTechLevel => {
                self.world.generate_tech_level();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RegenWorldTemperature => {
                self.world.generate_temperature();
                self.message_immediate(Message::WorldModelUpdated);
            }

            RemoveSelectedFaction => {
                self.faction_idx = self.world.remove_faction(self.faction_idx);
                self.message_immediate(Message::WorldModelUpdated);
            }

            RemoveSelectedWorld => {
                self.remove_world_popup();
            }

            RenameSubsector => {
                self.subsector_rename_popup();
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

                let directory: &Path = self.save_directory.as_ref();
                let filename: &Path = self.save_filename.as_ref();
                let path = directory.join(filename);

                if self.save_filename.is_empty() || !path.exists() {
                    return self.message_immediate(Message::SaveAs);
                } else {
                    let result = save_file(
                        &self.save_directory,
                        &self.save_filename,
                        self.subsector.to_json(),
                    );
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

                let default_filename = format!("{} Subsector.json", self.subsector.name());
                let filename = if !self.save_filename.is_empty() {
                    &self.save_filename
                } else {
                    &default_filename
                };

                let result = save_file_dialog(
                    &self.save_directory,
                    filename,
                    "JSON",
                    &["json"],
                    self.subsector.to_json(),
                );

                match result {
                    Ok(Some(path)) => {
                        self.save_directory = path.parent().unwrap().to_str().unwrap().to_string();
                        self.save_filename =
                            path.file_name().unwrap().to_str().unwrap().to_string();
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
                let location = Point::try_from(&self.point_str[..]);
                if let Ok(location) = location {
                    if location != self.point && Subsector::point_is_inbounds(&location) {
                        if let Some(world) = self.subsector.get_world(&location) {
                            self.occupied_hex_popup(world.name.clone(), location);
                        } else {
                            self.message_immediate(Message::ConfirmLocUpdate { location });
                        }
                    } else {
                        self.point_str = self.point.to_string();
                    }
                } else {
                    self.point_str = self.point.to_string();
                }
            }

            WorldModelUpdated => {
                self.world.resolve_trade_codes();
            }
        }
        true
    }

    fn with_world_abundance(world_abundance_dm: i16) -> Self {
        let subsector = Subsector::new(world_abundance_dm);
        Self {
            subsector,
            ..Self::empty()
        }
    }
}

impl App for GeneratorApp {
    fn on_exit_event(&mut self) -> bool {
        let can_exit = !self.has_unsaved_changes() || self.can_exit;
        if !can_exit {
            self.unsaved_exit_popup();
        }
        can_exit
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.can_exit {
            frame.quit();
        }

        self.check_world_edited();
        self.process_hotkeys(ctx);
        self.process_message_queue();
        if self.subsector_name_changed {
            frame.set_window_title(&(self.subsector.name().to_string() + " Subsector"));
        }

        self.show_gui(ctx);
    }
}

impl Default for GeneratorApp {
    fn default() -> Self {
        Self::with_world_abundance(0)
    }
}

impl From<Subsector> for GeneratorApp {
    fn from(subsector: Subsector) -> Self {
        Self {
            subsector,
            ..Self::empty()
        }
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

    fn empty_app() -> GeneratorApp {
        GeneratorApp {
            subsector: Subsector::empty(),
            ..Default::default()
        }
    }

    mod message_tests {
        use super::*;

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

            assert!(app.has_unsaved_changes());
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

            app.world.notes = "Blah blah blah".to_string();
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
                            assert_eq!(app.point_str, point.to_string());
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

        #[test]
        fn new_faction_gov_selected() {
            let mut app = empty_app();
            let point = Point { x: 1, y: 1 };
            app.message_immediate(Message::HexGridClicked { new_point: point });
            app.message_immediate(Message::AddNewWorld);

            if app.world.factions.is_empty() {
                app.message_immediate(Message::AddNewFaction);
            }
            assert!(!app.world.factions.is_empty());

            // Simulate selecting a new faction by simply changing the faction_idx
            app.faction_idx = 0;
            let faction = &mut app.world.factions[app.faction_idx];

            let gov_table = &TABLES.gov_table;
            let default_description = &gov_table
                .iter()
                .find(|g| g.kind == faction.government.kind)
                .unwrap()
                .description;
            assert_eq!(faction.government.description, *default_description);

            let new_gov = gov_table
                .iter()
                .find(|g| g.kind != faction.government.kind)
                .unwrap();
            assert_ne!(*new_gov, faction.government);

            // Simulate choosing a new faction government selectable value on the GUI by changing
            // government "kind" directly and messaging NewFactionGovSelected with the new code
            faction.government.kind = new_gov.kind.clone();
            app.message_immediate(Message::NewFactionGovSelected {
                new_code: new_gov.code,
            });

            let faction = &mut app.world.factions[app.faction_idx];
            assert_eq!(faction.government.code, new_gov.code);
            assert_eq!(faction.government.kind, new_gov.kind);
            // Because we didn't change the faction's government description from the default on the
            // gov_table, it should have updated to that of the newly selected government
            assert_eq!(faction.government.description, new_gov.description);
            // We don't change the contraband of the faction's government because we don't care or
            // display contraband in the context of factions

            // We repeat the same test, but this time we change the faction's government description
            // from the default and confirm that it is retained
            let default_description = &gov_table
                .iter()
                .find(|g| g.kind == faction.government.kind)
                .unwrap()
                .description;
            assert_eq!(faction.government.description, *default_description);

            let new_gov = gov_table
                .iter()
                .find(|g| g.kind != faction.government.kind)
                .unwrap();
            assert_ne!(*new_gov, faction.government);

            let blah = "Blah blah blah".to_string();
            faction.government.description = blah.clone();
            faction.government.kind = new_gov.kind.clone();
            app.message_immediate(Message::NewFactionGovSelected {
                new_code: new_gov.code,
            });

            let faction = &app.world.factions[app.faction_idx];
            assert_eq!(faction.government.code, new_gov.code);
            assert_eq!(faction.government.kind, new_gov.kind);
            // Because we changed the faction's government description from the default it should
            // have been retained
            assert_eq!(faction.government.description, blah);
        }

        #[test]
        fn new_starport_class_selected() {
            use crate::astrography::table::StarportClass;

            let mut app = empty_app();
            let point = Point { x: 1, y: 1 };
            app.message_immediate(Message::HexGridClicked { new_point: point });
            app.message_immediate(Message::AddNewWorld);

            let old_starport = app.world.starport.clone();
            let new_class = match app.world.starport.class {
                StarportClass::A => StarportClass::B,
                StarportClass::B => StarportClass::C,
                StarportClass::C => StarportClass::D,
                StarportClass::D => StarportClass::E,
                StarportClass::E => StarportClass::X,
                StarportClass::X => StarportClass::A,
            };
            let new_starport = TABLES
                .starport_table
                .iter()
                .find(|sp| sp.class == new_class)
                .unwrap();

            app.world.starport.class = new_class;
            app.message_immediate(Message::NewStarportClassSelected);
            assert_ne!(app.world.starport, old_starport);
            assert_eq!(app.world.starport.code, new_starport.code);
            assert_eq!(app.world.starport.class, new_starport.class);
            // Generated berthing costs are 1d6 * the "base" starport table berthing cost; just need
            // to account for when berthing costs are zero
            if new_starport.berthing_cost != 0 {
                assert!(app.world.starport.berthing_cost % new_starport.berthing_cost == 0);
            } else {
                assert_eq!(app.world.starport.berthing_cost, new_starport.berthing_cost);
            }
            assert_eq!(app.world.starport.fuel, new_starport.fuel);
            assert_eq!(app.world.starport.facilities, new_starport.facilities);
        }
    }
}
