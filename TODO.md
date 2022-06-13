# Known TODOs

The purpose of this file is to track known TODOs before they're made into Github issues.
I often work on this code while I don't have access to the actual Github repo itself, so this is an easy way for me to track TODOs as find them when working.

TODOs will be removed from this file when made into Github issues and have ~~strikethroughs~~ if they are completed before being made into Github issues.
TODOs completed in this way should probably should still have an issue created and completed before being removed, but at the very least will be cleared out periodically.

## Open TODOs

### General TODOs
- Update project name
- Make executable portable by using `include_str!` and `include!` macros
- Improve unit test coverage
- More doc-comments where useful
- Remove redundant serialization and deserializing code once GUI is in a good state

### App/GUI TODOs
- Refactor older "description saving" messages to minimize cloning
- Add "save" and "revert" buttons to world data GUI to trigger saving world changes and redrawing SVG instead of clicking to a different grid point entirely
- Implement `Subsector` json saving and loading
    - "Player-safe" versions of files would also be nice
- Implement `Subsector` renaming
- Implement whole `Subsector` regenerating functionality with configurable `world_abundance_dm`
- "Player-safe" version of the GUI, probably using different build targets
- Add support for creating Stellar alliances and trade connections in GUI once backend groundwork is completed for it
- Add dark mode support... somehow

### Backened/Astrography TODOs
- Add a background color to the subsector grid svg template, rather than just transparent and unreadable on renderers with a dark background
- Move world generation over to be more in line with the Cepheus Engine SRD, though it still may not be full compatible
- Rename project and repo to "Stars With Travellers Generator" (swt-gen) or something like that
- Add support for colored "stellar alliances" and trade/diplomatic connection lines; ***actually creating these alliances and connections*** will be done by the user in the GUI
    - Update svg generation to display these

### Far Future TODOs
- Create a web-app version
- Create a demo website using the web-app
- Create an Obsidian plugin; this is ***only worth doing if*** making the web-app takes you 95% of the way towards to displaying the GUI in an Electron application like Obsidian

## Completed TODOs

### General Issues
- ~~Make this TODO list~~

### App/GUI TODOs
- ~~Finish adding displays for all `Subsector` fields~~
- ~~Implement world movement to the GUI using the `WorldLocUpdated` message~~
- ~~Implement new world generation on empty hexes~~
- ~~Implement whole world regeneration~~
- ~~Implement world deletion~~
