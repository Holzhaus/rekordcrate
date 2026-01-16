# rekordcrate - Library for parsing Pioneer Rekordbox device exports

[![Version](https://img.shields.io/crates/v/rekordcrate)](https://crates.io/crates/rekordcrate)
[![License](https://img.shields.io/github/license/Holzhaus/rekordcrate)](https://github.com/Holzhaus/rekordcrate/blob/main/COPYING)
[![Build Status](https://img.shields.io/github/actions/workflow/status/Holzhaus/rekordcrate/build.yml?branch=main)](https://github.com/Holzhaus/rekordcrate/actions?query=branch%3Amain)

*rekordcrate* is library to parse device exports for the CDJ/XDJ series players
(usually exported from the Pioneer Rekordbox DJ software), written in Rust.

**Note:** This library is currently still under heavy development and might
have breaking API changes in the future.

## Command Line Usage

This library includes a command line tool named `rekordcrate` to inspect
database exports (i.e. `PIONEER/rekordbox/export.pdb` files):

    $ cargo run -- dump-pdb data/complete_export/demo-tracks/PIONEER/rekordbox/export.pdb

Analysis files (`.DAT`, `.EXT` and `.2EX` files in the `PIONEER/USBANLZ`
directory) can also be viewed:

    $ cargo run -- dump-anlz -- data/complete_export/demo_tracks/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT

The tool is also able to display the contents of `*SETTING.DAT` files
(`DEVSETTING.DAT`, `DJMMYSETTING.DAT`, `MYSETTING.DAT` and `MYSETTING2.DAT`
files in the `PIONEER` directory):

    $ cargo run -- dump-setting -- data/complete_export/demo_tracks/PIONEER/MYSETTING.DAT

Information about additional commands can be accessed using the `--help` flag.

## FAQ

### Is this software affiliated with Pioneer Corp. or its related companies?

No, this library has been written independently.

### Is the official documentation on the file format?

There isn't any official documentation publicly available, but [James
Elliott](https://github.com/brunchboy), [Henry
Betts](https://github.com/henrybetts), [Fabian
Lesniak](https://github.com/flesniak) and others reverse-engineered and
documented it on
[djl-analysis.deepsymmetry.org](https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html).

## License

This software is licensed under the terms of the [Mozilla Public License
2.0](https://www.mozilla.org/en-US/MPL/2.0/). Please also have a look at the
[license FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).
