# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Features

- setting: Add method to construct default setting objects
- setting: Derive `Clone` and `Copy` traits for all setting values
- setting: Derive `Clone` for all setting data structs
- setting: Add `Display` implementation for setting values

## [0.2.0] - 2022-10-09

- Switch from `nom` to `binrw` to pave the way for serialization support in the future
- Improve documentation and add contribution guide
- Add new command-line interface, which now also offers a way list the playlist tree
- Add support for reading and serializing `*SETTING.DAT` files
- Add more tests
- Various small bug fixes and improvements (e.g., using 8-bit color indices consistently)

## [0.1.0] - 2022-02-10

Initial release.
