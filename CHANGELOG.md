# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Refactoring

- Use binrw instead of nom to parse `ANLZ*.DAT` files #47 #49 #51
- Use binrw instead of nom to parse `*SETTING.DAT` files #40 #42 #50

### Bug Fixes

- anlz: Add length checks for `ANLZ*.DAT` parser #15 #33
- pdb: Fix Table RowGroup parsing #20
- pdb: Fix Row::parse_album string offset #22
- pdb: Fix DeviceSQLString::parse_long_utf16le and DeviceSQLString::parse_long_ascii #28

### Features

- setting: Add parser for `*SETTING.DAT` files #27 #32 #34
- pdb: Implement ISRC-string parsing
- pdb: Implement proper PageIndex iteration #24

### Testing

- Generate smoke tests for setting files #30 #31
- Add tests for individual setting values #43

### Documentation

- Improve documentation for `anlz` parser #19 #29

## [0.1.0] - 2022-02-10

Initial release.
