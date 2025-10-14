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
- pdb: WIP pass artists_page test
- Add `ofs_name` field to Album row and correctly (de-)serialize it
- pdb: Set fixed 4-byte alignment on labels
- pdb: Set 4-byte alignment for Artwork rows
- pdb: Increase struct fields visibility in ext.rs for use in tests
- pdb: Add padding field to tag struct
- pdb: Set fixed 4-byte alignment for tag rows
- pdb: Add mapping for optional NonZero<u32> in ParentId struct
- Fix various compiler or clippy warnings with Rust 1.90

### Documentation

- pdb: Add doc comments for struct fields in ext.rs
- Link to upstream Tag row docs in docs

### Features

- Use "clean" buffer in `track_page` test & refactor `DeviceSQLString` construction
- Add `artist_page_long` test to cover the 0x64 artist subtype
- Get `labels_page` passing
- Add `VarOffsetTail` as an abstraction for trailing var-len data
- Replace `VarOffsetTail` with `OffsetArray` and add ExplicitPadding
- Parse rekordbox exportExt.pdb format
- Convert dump-ext-pdb into flag instead with some guessing magic

### Refactor

- Simplify `Page` and `RowGroup` parsing
- Fix test `assert_eq!(result, expected)` parameter order
- Move `Row::Artist`-specific padding to `Artist` struct
- Remove length `assert_eq` from roundtrip tests
- Use `VarOffsetTail` for Artist rows
- Use `VarOffsetTail` for `Album` struct
- Move `VarOffsetTail` to its own module
- Add separate row `Subtype` type
- Remodel OffsetArray so it can be used with (almost) any type
- Use OffsetArray for Track rows
- Cleanup `pdb/offset_array.rs` a little by reducing duplication
- OffsetArray->OffsetArrayContainer, OffsetArrayImpl->OffsetArray
- Outline page test buffers
- pdb: Remove unneeded explicit padding from tag rows

### Testing

- pdb: Add genres_page test
- pdb: Add artists_page test
- pdb: Move tests to sepparate file
- pdb: Fix mistake in artists_page test
- pdb: Corrections after moving tests
- pdb: WIP added albums_page test
- pdb: Add labels_page test
- pdb: Add keys_page test
- pdb: Add colors_page test and fix colors padding to pass test
- pdb: Add playlist entry row and page tests
- pdb: Add playlist tree row and page tests
- pdb: Add artwork page test
- pdb: Add tag and track_tag page tests
- pdb: Add history playlists and entries page tests

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
