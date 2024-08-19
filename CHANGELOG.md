# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Bug Fixes

- Always read all 16 rows potential from each row group

### Refactor

- Avoid temporary Vec allocation in assert_pdb_row_count
- Use div_ceil instead of handrolled checked arithmatic
- Improve `BinRead` impl of `RowGroup`

### Testing

- Add regression tests to ensure all rows are read

## [0.2.1] - 2023-11-30

### Bug Fixes

- pdb: Skip reading rows of invalid pages

### Features

- setting: Add method to construct default setting objects
- setting: Derive `Clone` and `Copy` traits for all setting values
- setting: Derive `Clone` for all setting data structs
- setting: Add `Display` implementation for setting values
- pdb: Add Columns table
- pdb: Mark table rows as serializable

### Refactor

- cli: Return `Result` from main method instead of unwrapping

### Testing

- util: Add helper function for passing args to roundtrip tests
- util: Add additional length checks to roundtrip tests
- util: Print useful diffs when `assert_eq!` fails on large blobs

## [0.2.0] - 2022-10-09

- Switch from `nom` to `binrw` to pave the way for serialization support in the future
- Improve documentation and add contribution guide
- Add new command-line interface, which now also offers a way list the playlist tree
- Add support for reading and serializing `*SETTING.DAT` files
- Add more tests
- Various small bug fixes and improvements (e.g., using 8-bit color indices consistently)

## [0.1.0] - 2022-02-10

Initial release.
