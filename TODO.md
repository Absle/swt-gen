# Known TODOs

The purpose of this file is to track known TODOs before they're made into Github issues.
I often work on this code while I don't have access to the actual Github repo itself, so this is an easy way for me to track TODOs as find them when working.

TODOs will be removed from this file when made into Github issues and have ~~strikethroughs~~ if they are completed before being made into Github issues.
TODOs completed in this way should probably should still have an issue created and completed before being removed, but at the very least will be cleared out periodically.

## Open TODOs

### Bugs
- Large world names overflow hex on a single line

### General TODOs
- Improve unit test coverage
- More doc-comments where useful
- Some kind of logging schema
- Auto saving, probably to some `*.json~` backup files
- Some kind of user preference saving
- Refactor the player-safe gui into a separate binary rather than a feature
- Make an install/release script to zip up both the default binary and player-safe binary

### App/GUI TODOs
- Add support for creating Stellar alliances and trade connections in GUI once backend groundwork is completed for it
- Add dark mode support... somehow
- Implement a framework to more structurally link `Message`s with their hotkeys
- Clicking to new planet should just apply the changes by default rather than having a popup
- Tech level should have some indication of what the number means, not just a number
- Size should have some comparison with Earth or list the gravity, not just a number
- Add a way to reorder factions
- Adjust faction tab GUI spacing
- Rework whole-world-regeneration to allow reverting and get rid of warning popup
- Refactor calls to `TextStyle::Heading.resolve(&Style::default())` into one `rich_text_heading` function; consider using lazy static for it
- Add a "TAS Description" or "GM Description" that is viewable but not editable in player-safe GUI
- Consider refactoring the subsector map rendering pipeline. Currently it manually paints world symbols on top of a (blurry) hex grid rendered from an SVG; these should probably be unified into some kind of hex grid GUI.

### Backened/Astrography TODOs
- Move world generation over to be more in line with the Cepheus Engine SRD, though it still may not be full compatible
- Add a background color to the subsector grid svg template, rather than just transparent and unreadable on renderers with a dark background
- Add support for colored "stellar alliances" and trade/diplomatic connection lines; ***actually creating these alliances and connections*** will be done by the user in the GUI
    - Update svg generation to display these

### Far Future TODOs
- Some kind of search/query/filtering system
- Create a web-app version
- Create a demo website using the web-app
- Create an Obsidian plugin; this is ***only worth doing if*** making the web-app takes you 95% of the way towards to displaying the GUI in an Electron application like Obsidian
- Add Markdown syntax support in the notes area of the app

## Completed TODOs

### Bugs
- ~~When regenerating the subsector with an unsaved, no-file one already loaded, pressing "Cancel" on the save dialog still lets the regeneration continue when it should stop. Uncertain if this affects file loading in the same way.~~
- ~~Pressing the revert button doesn't correctly reset the displayed diameter of the world. Reverting and changing away from the world and back makes it display the original value, so it's mostly likely just the text box not updating properly~~

### General TODOs
- ~~Make this TODO list~~
- ~~Rename `mod.rs` files~~
- ~~Refactor implementations of `ToString` to `std::fmt::Display`~~
- ~~Update project name~~
- ~~Remove redundant serialization and deserializing code once GUI is in a good state~~
- ~~Fix up interface and module interdependencies to make things more usable externally and less monolithic~~
- ~~Make executable portable by using `include_str!` and `include!` macros~~

### App/GUI TODOs
- ~~Finish adding displays for all `Subsector` fields~~
- ~~Implement world movement to the GUI using the `WorldLocUpdated` message~~
- ~~Implement new world generation on empty hexes~~
- ~~Implement whole world regeneration~~
- ~~Implement world deletion~~
- ~~Implement `Subsector` renaming~~
- ~~Implement whole `Subsector` regenerating functionality with configurable `world_abundance_dm`~~
- ~~Implement `Subsector` json serialization~~
- ~~Implement `Subsector` json deserialization~~
- ~~Add "apply" and "revert" buttons to world data GUI to trigger saving world changes and redrawing SVG instead of clicking to a different grid point entirely~~
- ~~Maybe change up menus to more conventional "File > Save/Load" and "Edit > Regenerate/Rename Subsector"~~
    - ~~Distinguish between "Save" and "Save As..."~~
- ~~Add hotkeys for "Save", "Open", etc.~~
- ~~Add "unsaved changes, are you sure you want to close?" popup of some kind~~
- ~~Add subsector map exporting~~
- ~~"Player-safe" version of the GUI, probably using different build targets~~
    - ~~Implement "player-safe" version of json serialization~~
- ~~Find some way to get `Popup`s to appear in the middle of the screen, rather than the top right corner initially~~
- ~~Add some margins to the notes editing tab for better reading~~
- ~~Refactor older "description saving" `Message`s to minimize cloning~~
- ~~Break up GUI elements in `app/mod.rs` into multiple files similar to `polity_display.rs`~~
- ~~Refactor all gui elements into different files~~
- ~~Refactor popups to use message pipes instead~~
- ~~Refactor popup processing into `gui` code~~
- ~~Refactor `ButtonPopup` to use builder-pattern `add_button` calls rather than requiring `ButtonPopup` to always be mut. Then `add_button` might be able to use the `must_call` tag~~
- ~~Refactor svg generation to use a proper xml editor~~
- ~~Consider moving from `self.message(Message)` to using the `pipe` system to take advantage of non-mutable borrowing~~
- ~~Remove `Message::Cancel*` messages that don't do anything anything and replace with `Message::NoOp`~~
