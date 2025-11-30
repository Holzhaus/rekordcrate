// Copyright (c) 2025 Nikolaus Einhauser <nikolaus.einhauser@web.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Module for reading and writing offset arrays.
//! An offset array consists of an array of offsets, followed by data at those offsets.
//! The offsets can be either u8 or u16, specified by the `OffsetSize` enum.
//! The inner type `T` is read/written with the array of absolute seek positions passed as an argument.
//! The inner type `T` must implement `BinRead` and `BinWrite`, and its `Args` must be a tuple of
//! `(&SeekArray<N>, IA)`, where the last argument is any additional
//! arguments required by `T`.
//! The `OffsetArrayContainer` itself takes as arguments a tuple of `(i64, OffsetSize, IA)`, where the first argument is the
//! offset of the start of the offsets themselves used to find the base position, the second argument is the
//! `OffsetSize`, and the third argument is any additional arguments required by `T`.
//! The number of offsets is determined by the const generic parameter `N`.
//! For example, an `OffsetArrayContainer<T, 3>` will read/write 3 offsets, followed by the data for `T`.
//! The offsets are read/written in little-endian format.
//! The inner type `T` is also read/written in little-endian format.
//! This struct implements `Deref` to `T`, so the inner value can be accessed directly.
//!! Example:
//! ```
//! # use binrw::{binrw, BinRead, BinWrite};
//! # use rekordcrate::pdb::offset_array::{OffsetArrayContainer, SeekArray, OffsetSize};
//! #[binrw]
//! #[brw(little)]
//! #[br(import(seeks: &SeekArray<1>, args: <T as BinRead>::Args<'_>))]
//! #[bw(import(seeks: &SeekArray<1>, args: <T as BinWrite>::Args<'_>))]
//! #[derive(Debug, PartialEq)]
//! struct SingleTarget<T: BinRead + BinWrite>(
//!     #[brw(args_raw = args)]
//!     #[br(parse_with = seeks.parser(0))]
//!     #[bw(write_with = seeks.writer(0))]
//!     T,
//! );
//!
//! #[binrw]
//! #[brw(little)]
//! #[derive(Debug, PartialEq)]
//! struct Toplevel {
//!    // Some initial data.
//!    initial: u32,
//!    // Offsets to the actual data, located 0x04 bytes into the structure.
//!    #[brw(args(0x04, OffsetSize::U8, ()))]
//!    data: OffsetArrayContainer<SingleTarget<u32>, 1>,
//! };
//! ```

use std::cell::Cell;

use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite};

/// Specifies whether the offsets are stored as u8 or u16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetSize {
    /// Offsets are stored as u8.
    U8,
    /// Offsets are stored as u16.
    U16,
}
/// An array of offsets, followed by data at those offsets.
/// The offsets are relative to the start of the offset array, minus an optional offset.
/// The offsets can be either u8 or u16, specified by the `OffsetSize`
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct OffsetArrayContainer<T, const N: usize>(T);

impl<T, const N: usize> From<T> for OffsetArrayContainer<T, N> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T, const N: usize> OffsetArrayContainer<T, N> {
    /// Consumes the container and returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Creates a new `OffsetArrayContainer` containing the given value.
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T, const N: usize> std::ops::Deref for OffsetArrayContainer<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

impl<T, const N: usize> std::ops::DerefMut for OffsetArrayContainer<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (self.0)
    }
}

impl<T: BinRead + BinWrite, const N: usize> binrw::meta::WriteEndian for OffsetArrayContainer<T, N>
where
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
{
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
}
impl<T: BinRead + BinWrite, const N: usize> binrw::meta::ReadEndian for OffsetArrayContainer<T, N>
where
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
{
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
}

impl<T, const N: usize, IA> BinRead for OffsetArrayContainer<T, N>
where
    for<'a> T: BinRead<Args<'a> = (&'a SeekArray<N>, IA)>,
{
    type Args<'a> = (i64, OffsetSize, IA);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (offsets_offset, offset_size, args): Self::Args<'_>,
    ) -> BinResult<Self> {
        let offsets_pos = reader.stream_position()?;
        let base = (offsets_pos as i64 - offsets_offset) as u64;
        let offsets = OffsetArray::<N>::read_options(reader, endian, (offset_size,))?;
        if !offsets.assert_offset_size_matches(offset_size) {
            return Err(binrw::Error::AssertFail {
                pos: offsets_pos,
                message: format!("offsetsize mismatch! {offset_size:?}"),
            });
        }
        let seeks = offsets.into_absolute(base);
        let inner = T::read_options(reader, endian, (&seeks, args))?;
        Ok(Self(inner))
    }
}

impl<T, const N: usize, IA> BinWrite for OffsetArrayContainer<T, N>
where
    for<'a> T: BinWrite<Args<'a> = (&'a SeekArray<N>, IA)>,
{
    type Args<'a> = (i64, OffsetSize, IA);

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        (offsets_offset, offset_size, args): Self::Args<'_>,
    ) -> BinResult<()> {
        let offsets_pos = writer.stream_position()?;
        let base = (offsets_pos as i64 - offsets_offset) as u64;

        // Seek past the offsets to write the data first.
        writer.seek(SeekFrom::Current(
            OffsetArray::<N>::byte_count(offset_size) as i64
        ))?;
        let seeks = [0u64; N].into();
        self.0.write_options(writer, endian, (&seeks, args))?;
        let data_end_pos = writer.stream_position()?;

        // Now go back and write the offsets.
        writer.seek(SeekFrom::Start(offsets_pos))?;
        let offsets = seeks.into_relative(base, offset_size);
        if !offsets.assert_offset_size_matches(offset_size) {
            return Err(binrw::Error::AssertFail {
                pos: offsets_pos,
                message: format!("offsetsize mismatch! {offset_size:?}"),
            });
        }
        offsets.write_options(writer, endian, ())?;

        // Last, seek past the end of the data for further writing.
        writer.seek(SeekFrom::Start(data_end_pos))?;
        Ok(())
    }
}

/// An array of offsets stored as absolute positions in the stream,
/// passed to the inner type `T` of an `OffsetArrayContainer<T>` for parsing and writing
/// at the specified offsets.
///
/// Sadly contains a `Cell` to allow mutation during writing, ideally binrw would
/// let us use a mutable reference but the way parsers/writers work makes this difficult.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeekArray<const N: usize>([Cell<u64>; N]);

impl<const N: usize> SeekArray<N> {
    fn into_relative(self, base: u64, size: OffsetSize) -> OffsetArray<N> {
        match size {
            OffsetSize::U8 => {
                let rel_offsets = self.0.map(|offset_cell| {
                    let offset = offset_cell.get();
                    offset
                        .checked_sub(base)
                        .ok_or_else(|| binrw::Error::AssertFail {
                            pos: base,
                            message: format!("Stream position underflow: {base}-{offset}"),
                        })
                        .and_then(|v| {
                            u8::try_from(v).map_err(|err| binrw::Error::AssertFail {
                                pos: base,
                                message: format!("{err}"),
                            })
                        })
                        .expect("(pending try_map stabilization)")
                });
                OffsetArray::U8(rel_offsets)
            }
            OffsetSize::U16 => {
                let rel_offsets = self.0.map(|offset_cell| {
                    let offset = offset_cell.get();
                    offset
                        .checked_sub(base)
                        .ok_or_else(|| binrw::Error::AssertFail {
                            pos: base,
                            message: format!("Stream position underflow: {base}-{offset}"),
                        })
                        .and_then(|v| {
                            u16::try_from(v).map_err(|err| binrw::Error::AssertFail {
                                pos: base,
                                message: format!("{err}"),
                            })
                        })
                        .expect("(pending try_map stabilization)")
                });
                OffsetArray::U16(rel_offsets)
            }
        }
    }

    /// Returns a parser that reads a type `T` at the specified index.
    pub fn parser<'a, T: BinRead, R: binrw::io::Read + binrw::io::Seek>(
        &'a self,
        index: usize,
    ) -> impl FnOnce(&mut R, binrw::Endian, T::Args<'_>) -> BinResult<T> + 'a {
        let pos = &self.0[index];
        move |reader, endian, inner_args| {
            reader.seek(SeekFrom::Start(pos.get()))?;
            T::read_options(reader, endian, inner_args)
        }
    }

    /// Returns a writer that writes a type `T` and stores the written offset at the specified index.
    pub fn writer<'a, T: BinWrite, R: binrw::io::Write + binrw::io::Seek>(
        &'a self,
        index: usize,
    ) -> impl FnOnce(&T, &mut R, binrw::Endian, T::Args<'_>) -> BinResult<()> + 'a {
        let pos = &self.0[index];
        move |element, writer, endian, inner_args| {
            pos.set(writer.stream_position()?);
            element.write_options(writer, endian, inner_args)
        }
    }
}

impl<const N: usize> From<[u64; N]> for SeekArray<N> {
    fn from(arr: [u64; N]) -> Self {
        Self(arr.map(Cell::new))
    }
}

/// The implementation of the offset array, which can be either u8 or u16.
/// This is a private implementation detail, use `OffsetArrayContainer` instead.
/// This enum is used to read/write the offsets, and can be converted into a
/// `SeekArray` which provides the actual binrw parsers/writers for each offset.
/// The offsets are stored in little-endian format and are prefixed by a magic value
/// 0x03u8/0x0003u16.
#[binrw]
#[derive(Debug, Clone, PartialEq, Eq)]
#[brw(little)]
#[br(import(size: OffsetSize))]
enum OffsetArray<const N: usize> {
    /// Offsets are stored as u8.
    /// First value (which isn't an offset) is always 0x03.
    #[br(pre_assert(size == OffsetSize::U8))]
    U8(#[brw(magic = 0x03u8)] [u8; N]),
    /// Offsets are stored as u16.
    /// First value (which isn't an offset) is always 0x0003.
    #[br(pre_assert(size == OffsetSize::U16))]
    U16(#[brw(magic = 0x0003u16)] [u16; N]),
}

impl<const N: usize> OffsetArray<N> {
    fn into_absolute(self, base: u64) -> SeekArray<N> {
        match self {
            OffsetArray::U8(offsets) => offsets.map(|o| o as u64),
            OffsetArray::U16(offsets) => offsets.map(|o| o as u64),
        }
        .map(|o| base + o)
        .into()
    }

    fn byte_count(size: OffsetSize) -> usize {
        (N + 1)
            * match size {
                OffsetSize::U8 => 1,
                OffsetSize::U16 => 2,
            }
    }

    fn assert_offset_size_matches(&self, offset_size: OffsetSize) -> bool {
        matches!(
            (self, offset_size),
            (Self::U8(_), OffsetSize::U8) | (Self::U16(_), OffsetSize::U16)
        )
    }
}

impl<const N: usize> From<[u8; N]> for OffsetArray<N> {
    fn from(arr: [u8; N]) -> Self {
        Self::U8(arr)
    }
}

impl<const N: usize> From<[u16; N]> for OffsetArray<N> {
    fn from(arr: [u16; N]) -> Self {
        Self::U16(arr)
    }
}

#[cfg(test)]
mod test {

    use binrw::VecArgs;

    use super::*;
    use crate::util::testing::{
        test_read_with_args, test_roundtrip_with_args, test_write_with_args,
    };

    #[binrw]
    #[brw(little)]
    #[br(import(_seeks: &SeekArray<N>, args: <T as BinRead>::Args<'_>))]
    #[bw(import(_seeks: &SeekArray<N>, args: <T as BinWrite>::Args<'_>))]
    #[derive(Debug, PartialEq)]
    struct IgnoreArgs<T: BinRead + BinWrite, const N: usize>(#[brw(args_raw = args)] T);

    #[binrw]
    #[brw(little)]
    #[br(import(seeks: &SeekArray<1>, args: <T as BinRead>::Args<'_>))]
    #[bw(import(seeks: &SeekArray<1>, args: <T as BinWrite>::Args<'_>))]
    #[derive(Debug, PartialEq)]
    // This could also be used outside of tests, it just isn't yet (though a version is, called "TrailingName")
    pub struct SingleTarget<T: BinRead + BinWrite>(
        #[brw(args_raw = args)]
        #[br(parse_with = seeks.parser(0))]
        #[bw(write_with = seeks.writer(0))]
        T,
    );

    impl<T: BinRead + BinWrite> std::ops::Deref for SingleTarget<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &(self.0)
        }
    }

    #[test]
    fn empty() {
        let empty_offset_tail_u8 = OffsetArrayContainer::new(IgnoreArgs::<_, 0>(()));
        test_roundtrip_with_args(
            &[0x03],
            empty_offset_tail_u8,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
        let empty_offset_tail_u16 = OffsetArrayContainer::new(IgnoreArgs::<_, 0>(vec![(); 0]));
        test_roundtrip_with_args(
            &[0x03, 0x00],
            empty_offset_tail_u16,
            (
                0,
                OffsetSize::U16,
                VecArgs {
                    count: 0,
                    inner: (),
                },
            ),
            (0, OffsetSize::U16, ()),
        );
    }

    #[test]
    fn near_u8() {
        let near_u8_tail = OffsetArrayContainer::new(SingleTarget(42u8));
        test_roundtrip_with_args(
            &[0x03, 0x02, 42],
            near_u8_tail,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn buffer() {
        let buffer = OffsetArrayContainer::new(SingleTarget(0xDEADBEEF_u32.to_be_bytes()));
        test_roundtrip_with_args(
            &[0x03, 0x02, 0xDE, 0xAD, 0xBE, 0xEF],
            buffer,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn near_remote() {
        let near_remote = OffsetArrayContainer::new(SingleTarget(42u8));
        test_read_with_args(
            &[0x03, 0x05, 0x00, 0x00, 0x00, 42],
            &near_remote,
            (0, OffsetSize::U8, ()),
        );
        // Write will always place the data immediately after the offsets.
        test_write_with_args(&[0x03, 0x02, 42], &near_remote, (0, OffsetSize::U8, ()));
    }
    #[test]
    fn far_remote() {
        let far_remote = OffsetArrayContainer::new(SingleTarget(42u8));
        test_read_with_args(
            &[0x03, 0x00, 0x05, 0x00, 0x00, 42],
            &far_remote,
            (0, OffsetSize::U16, ()),
        );
        // Write will always place the data immediately after the offsets.
        test_write_with_args(
            &[0x03, 0x00, 0x04, 0x00, 42],
            &far_remote,
            (0, OffsetSize::U16, ()),
        );
    }
    #[test]
    fn near_with_base() {
        #[binrw]
        #[brw(little)]
        #[derive(Debug, PartialEq)]
        struct TestStruct {
            some_values: [u8; 3],
            #[brw(args(3, OffsetSize::U8, ()))]
            offsets: OffsetArrayContainer<SingleTarget<u8>, 1>,
        }
        let data = TestStruct {
            some_values: [0xAA, 0xBB, 0xCC],
            offsets: OffsetArrayContainer::new(SingleTarget(42u8)),
        };
        test_roundtrip_with_args(&[0xAA, 0xBB, 0xCC, 0x03, 0x05, 42], data, (), ());
    }

    #[binrw]
    #[brw(little)]
    #[br(import(seeks: &SeekArray<2>, args: <T as BinRead>::Args<'_>))]
    #[bw(import(seeks: &SeekArray<2>, args: <T as BinWrite>::Args<'_>))]
    #[derive(Debug, PartialEq)]
    struct Multiple<T: BinRead + BinWrite>
    where
        for<'a> <T as BinRead>::Args<'a>: Clone,
        for<'a> <T as BinWrite>::Args<'a>: Clone,
    {
        #[brw(args_raw = args.clone())]
        #[br(parse_with = seeks.parser(0))]
        #[bw(write_with = seeks.writer(0))]
        a: T,
        #[brw(args_raw = args)]
        #[br(parse_with = seeks.parser(1))]
        #[bw(write_with = seeks.writer(1))]
        b: T,
    }

    #[test]
    fn multiple() {
        let multiple = OffsetArrayContainer::new(Multiple {
            a: 0xC0u8,
            b: 0xDEu8,
        });
        test_roundtrip_with_args(
            &[0x03, 0x03, 0x04, 0xC0, 0xDE],
            multiple,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn switched_ordering() {
        let multiple = OffsetArrayContainer::new(Multiple {
            a: 0xC0u8,
            b: 0xDEu8,
        });
        test_read_with_args(
            &[0x03, 0x04, 0x03, 0xDE, 0xC0],
            &multiple,
            (0, OffsetSize::U8, ()),
        );
        // Write will always place the data in sequential ordering.
        test_write_with_args(
            &[0x03, 0x03, 0x04, 0xC0, 0xDE],
            &multiple,
            (0, OffsetSize::U8, ()),
        );
    }
}
