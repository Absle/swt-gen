# Known TODOs

The purpose of this file is to track known TODOs before they're made into Github issues.
TODOs will be removed from this file when made into Github issues and have ~~strikethroughs~~ if they are completed before being made into Github issues.
~~Struckthrough~~ TODOs should probably should still have an issue created and completed before being removed, but at the very least will be cleared out periodically.

## Open TODOs

### General Issues
- Make executable portable by using `include_str!` and `include!` macros
- Improve unit test coverage
- More doc-comments where useful

### App/GUI Issues
- Refactor older "description saving" messages to minimize cloning
- Finish adding displays for all `Subsector` fields
- Implement world movement to the GUI using the `WorldLocUpdated` message
- Implement new world generation on empty hexes
- Implement world deletion
- Implement `Subsector` json saving and loading
    - "Player-safe" versions of files would also be nice
- "Player-safe" version of the GUI, probably using different build targets

## Completed TODOs

### General Issues
- ~~Make this TODO list~~