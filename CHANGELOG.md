# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Bug Fixes

- Plain Hotcues being rejected due to cuetype mismatch
- pdb: Change padding logic for track rows
- pdb: Pass parsing stage of track_page test
- pdb: Apply review feedback for 3d5d57c
- pdb: Validate row count in RowGroup to avoid silent overflow
- pdb: Allow unused method to pass CI
- pdb: Pass track_page test by not reversing in present_rows
- pdb: Re-add comments removed during merge
- pdb: Pass 5 more test_pdb_num_rows* tests
- Doctest broken by previous string refactor commit
- pdb: All tests pass + clean up comments
- pdb: Apply review feedback for 78ee51c
- pdb: Accept review feedback for 969d507
- `genres_page` test offset padding and rowgroup ordering

### Features

- Use "clean" buffer in `track_page` test & refactor `DeviceSQLString` construction

### Refactor

- Simplify `Page` and `RowGroup` parsing
- Fix test `assert_eq!(result, expected)` parameter order

### Testing

- pdb: Add genres_page test

## [0.3.0] - 2025-01-23

### Bug Fixes

- Always read all 16 rows potential from each row group

### Documentation

- changelog: Fix typo

### Features

- xml: Add support for Rekordbox XML format

### Refactor

- Avoid temporary Vec allocation in assert_pdb_row_count
- Use div_ceil instead of handrolled checked arithmatic
- Improve `BinRead` impl of `RowGroup`

### Testing

- Add regression tests to ensure all rows are read

## [0.2.1] - 2023-11-30

### Bug Fixes

- pdb: Skip reading rows of invalid pages
- pdb: Adhere to rows alignment to type alignment when writing

### Features

- setting: Add method to construct default setting objects
- setting: Derive `Clone` and `Copy` traits for all setting values
- setting: Derive `Clone` for all setting data structs
- setting: Add `Display` implementation for setting values
- pdb: Add Columns table
- pdb: Mark table rows as serializable
- pdb: Implement support for serialization of table pages

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
