// Copyright (c) 2026 Nikolaus Einhauser <nikolaus.einhauser@web.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Module for reading and writing offset arrays.
//!
//! An offset array consists of an array of offsets, followed by data at those offsets.
//! The offsets can be either u8 or u16, specified by the `OffsetSize` enum.
//! The inner type `T` is constructed from an N-array of `U`, read from the positions at
//! N offsets. The offsets are read/written in little-endian format.
//! For example, an `OffsetArrayContainer<T, 3>` where `T` implements `OffsetArrayItems`
//! will read/write 3 offsets, followed by 3 instances of `T::Item` at the offsets which
//! are used to construct an inner `T`.
//!
//! Example:
//! ```
//! # use binrw::{binrw, BinRead, BinWrite};
//! # use rekordcrate::pdb::offset_array::{OffsetArrayContainer, OffsetArrayItems};
//! #[derive(Debug, PartialEq)]
//! struct SingleTarget<T: BinRead + BinWrite>(T);
//!
//! impl<T: BinRead + BinWrite> OffsetArrayItems<1> for SingleTarget<T> {
//!     type Item = T;
//!
//!     fn as_items(&self) -> [&Self::Item; 1] {
//!         [&self.0]
//!     }
//!
//!     fn from_items(items: [Self::Item; 1]) -> Self {
//!      let [v] = items;
//!         Self(v)
//!     }
//! }
//!
//!
//! let near_u8_tail = OffsetArrayContainer {
//!     offsets: [1u8].into(),
//!     inner: SingleTarget(42u8),
//! };
//! ```

use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite};

/// Specifies whether the offsets are stored as u8 or u16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetSize {
    /// Offsets are stored as u8.
    U8,
    /// Offsets are stored as u16.
    U16,
}

/// A container for an array of offsets and the indirectly-addressed data at those offsets.
/// See the module documentation for an example.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct OffsetArrayContainer<T, const N: usize> {
    /// The offsets, either u8 or u16.
    pub offsets: OffsetArray<N>,

    /// The inner value, read/written with the offsets.
    pub inner: T,
}

impl<T, const N: usize> std::ops::Deref for OffsetArrayContainer<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &(self.inner)
    }
}

impl<T, U, const N: usize> binrw::meta::WriteEndian for OffsetArrayContainer<T, N>
where
    T: OffsetArrayItems<N, Item = U>,
    U: BinRead + BinWrite,
    for<'a> <U as BinRead>::Args<'a>: Clone,
    for<'a> <U as BinWrite>::Args<'a>: Clone,
{
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
}
impl<T, U, const N: usize> binrw::meta::ReadEndian for OffsetArrayContainer<T, N>
where
    T: OffsetArrayItems<N, Item = U>,
    U: BinRead + BinWrite,
    for<'a> <U as BinRead>::Args<'a>: Clone,
    for<'a> <U as BinWrite>::Args<'a>: Clone,
{
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
}

impl<T, const N: usize> OffsetArrayContainer<T, N> {
    fn calculate_base(start: u64, offset: u64) -> BinResult<u64> {
        start
            .checked_sub(offset)
            .ok_or_else(|| binrw::Error::AssertFail {
                pos: start,
                message: format!("Stream position underflow: {start}-{offset}"),
            })
    }
}

impl<T, U, const N: usize, IA> BinRead for OffsetArrayContainer<T, N>
where
    T: OffsetArrayItems<N, Item = U>,
    U: for<'a> BinRead<Args<'a> = IA>,
    IA: Clone,
{
    type Args<'a> = (u64, OffsetSize, IA);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (offset, offset_size, args): Self::Args<'_>,
    ) -> BinResult<Self> {
        let start = reader.stream_position()?;
        let offsets = OffsetArray::<N>::read_options(reader, endian, (offset_size,))?;
        if !offsets.assert_offset_size_matches(offset_size) {
            return Err(binrw::Error::AssertFail {
                pos: start,
                message: format!("offsetsize mismatch! {offset_size:?}"),
            });
        }
        let base = Self::calculate_base(start, offset)?;
        reader.seek(SeekFrom::Start(base))?;
        let seeks = offsets.as_seeks();
        use crate::util::ArrayPolyfills;
        let parsed = seeks.try_map_polyfill(|seek| {
            reader.seek(seek)?;
            let v = U::read_options(reader, endian, args.clone())?;
            // Restore position after each item.
            reader.seek(SeekFrom::Start(base))?;
            Ok::<U, binrw::Error>(v)
        })?;
        Ok(Self {
            offsets,
            inner: T::from_items(parsed),
        })
    }
}

impl<T, U, const N: usize, IA> BinWrite for OffsetArrayContainer<T, N>
where
    T: OffsetArrayItems<N, Item = U>,
    U: for<'a> BinWrite<Args<'a> = IA>,
    IA: Clone,
{
    type Args<'a> = (u64, OffsetSize, IA);

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        (offset, offset_size, args): Self::Args<'_>,
    ) -> BinResult<()> {
        let start = writer.stream_position()?;
        if !self.offsets.assert_offset_size_matches(offset_size) {
            return Err(binrw::Error::AssertFail {
                pos: start,
                message: format!("offsetsize mismatch! {offset_size:?}"),
            });
        }
        self.offsets.write_options(writer, endian, ())?;

        let base = Self::calculate_base(start, offset)?;
        writer.seek(SeekFrom::Start(base))?;
        let seeks = self.offsets.as_seeks();
        let items = self.inner.as_items();
        use crate::util::ArrayPolyfills;
        items.zip_polyfill(seeks).try_map_polyfill(|(v, seek)| {
            writer.seek(seek)?;
            v.write_options(writer, endian, args.clone())?;
            // Restore position after each item.
            writer.seek(SeekFrom::Start(base))?;
            Ok::<(), binrw::Error>(())
        })?;
        Ok(())
    }
}

/// Inner type that can be stored in an OffsetArrayContainer.
pub trait OffsetArrayItems<const N: usize> {
    /// Type of the items pointed to by the offsets.
    type Item;

    /// Returns the items as an array of references.
    fn as_items(&self) -> [&Self::Item; N];

    /// Constructs the composite type from an array of items.
    fn from_items(items: [Self::Item; N]) -> Self;
}

impl OffsetArrayItems<0> for () {
    type Item = ();

    fn as_items(&self) -> [&Self::Item; 0] {
        []
    }

    fn from_items(_items: [Self::Item; 0]) -> Self {}
}

/// The implementation of the offset array, which can be either u8 or u16.
/// This is a private implementation detail, use `OffsetArrayContainer` instead.
/// This enum is used to read/write the offsets, and to provide the `read_offset` and
/// `write_offset` methods to read/write the inner type `T` at the specified offsets.
/// The offsets are stored in little-endian format.
///
/// Offsets are always preceded by a magic number 0x03u8/0x0003u16.
#[binrw]
#[derive(Debug, Clone, PartialEq, Eq)]
#[brw(little)]
#[br(import(size: OffsetSize))]
pub enum OffsetArray<const N: usize> {
    /// Offsets are stored as u8.
    #[br(pre_assert(size == OffsetSize::U8))]
    U8(#[brw(magic(0x03u8))] [u8; N]),
    /// Offsets are stored as u16.
    #[br(pre_assert(size == OffsetSize::U16))]
    U16(#[brw(magic(0x0003u16))] [u16; N]),
}

impl<const N: usize> OffsetArray<N> {
    fn assert_offset_size_matches(&self, offset_size: OffsetSize) -> bool {
        matches!(
            (self, offset_size),
            (Self::U8(_), OffsetSize::U8) | (Self::U16(_), OffsetSize::U16)
        )
    }

    fn as_seeks(&self) -> [SeekFrom; N] {
        match self {
            OffsetArray::U8(offsets) => offsets.map(|offset| SeekFrom::Current(offset.into())),
            OffsetArray::U16(offsets) => offsets.map(|offset| SeekFrom::Current(offset.into())),
        }
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

    use super::*;
    use crate::util::testing::test_roundtrip_with_args;

    #[derive(Debug, PartialEq)]
    // This could also be used outside of tests, it just isn't yet (though a version is, called "TrailingName")
    pub struct SingleTarget<T: BinRead + BinWrite>(T);

    impl<T: BinRead + BinWrite> OffsetArrayItems<1> for SingleTarget<T> {
        type Item = T;

        fn as_items(&self) -> [&Self::Item; 1] {
            [&self.0]
        }

        fn from_items(items: [Self::Item; 1]) -> Self {
            let [v] = items;
            Self(v)
        }
    }

    impl<T: BinRead + BinWrite> std::ops::Deref for SingleTarget<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &(self.0)
        }
    }

    #[test]
    fn empty() {
        let empty_offset_tail_u8 = OffsetArrayContainer {
            offsets: OffsetArray::U8([]),
            inner: (),
        };
        test_roundtrip_with_args(
            &[0x03],
            empty_offset_tail_u8,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
        let empty_offset_tail_u16 = OffsetArrayContainer {
            offsets: OffsetArray::U16([]),
            inner: (),
        };
        test_roundtrip_with_args(
            &[0x03, 0x00],
            empty_offset_tail_u16,
            (0, OffsetSize::U16, ()),
            (0, OffsetSize::U16, ()),
        );
    }
    #[test]
    fn near_u8() {
        let near_u8_tail = OffsetArrayContainer {
            offsets: [2u8].into(),
            inner: SingleTarget(42u8),
        };
        test_roundtrip_with_args(
            &[0x03, 0x02, 42],
            near_u8_tail,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn buffer() {
        let buffer = OffsetArrayContainer {
            offsets: [2u8].into(),
            inner: SingleTarget(0xDEADBEEF_u32.to_be_bytes()),
        };
        test_roundtrip_with_args(
            &[0x03, 0x02, 0xDE, 0xAD, 0xBE, 0xEF],
            buffer,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn near_remote() {
        let near_remote = OffsetArrayContainer {
            offsets: [5u8].into(),
            inner: SingleTarget(42u8),
        };
        test_roundtrip_with_args(
            &[0x03, 0x05, 0x00, 0x00, 0x00, 42],
            near_remote,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn far_remote() {
        let far_remote = OffsetArrayContainer {
            offsets: [5u16].into(),
            inner: SingleTarget(42u8),
        };
        test_roundtrip_with_args(
            &[0x03, 0x00, 0x05, 0x00, 0x00, 42],
            far_remote,
            (0, OffsetSize::U16, ()),
            (0, OffsetSize::U16, ()),
        );
    }
    #[test]
    fn near_offset() {
        #[binrw]
        #[brw(little)]
        #[derive(Debug, PartialEq)]
        struct Data {
            padding: [u8; 3],
            #[brw(args(3, OffsetSize::U8, ()))]
            offsets: OffsetArrayContainer<SingleTarget<u8>, 1>,
        }
        let data = Data {
            padding: [0u8; 3],
            offsets: OffsetArrayContainer {
                offsets: [5u8].into(),
                inner: SingleTarget(42u8),
            },
        };
        test_roundtrip_with_args(&[0, 0, 0, 0x03, 0x05, 42], data, (), ());
    }

    #[derive(Debug, PartialEq)]
    struct Multiple<T: BinRead + BinWrite>
    where
        for<'a> <T as BinRead>::Args<'a>: Clone,
        for<'a> <T as BinWrite>::Args<'a>: Clone,
    {
        a: T,
        b: T,
    }

    impl<T: BinRead + BinWrite> OffsetArrayItems<2> for Multiple<T>
    where
        for<'a> <T as BinRead>::Args<'a>: Clone,
        for<'a> <T as BinWrite>::Args<'a>: Clone,
    {
        type Item = T;

        fn as_items(&self) -> [&Self::Item; 2] {
            [&self.a, &self.b]
        }

        fn from_items(items: [Self::Item; 2]) -> Self {
            let [a, b] = items;
            Self { a, b }
        }
    }

    #[test]
    fn multiple() {
        let multiple = OffsetArrayContainer {
            offsets: [3u8, 4u8].into(),
            inner: Multiple {
                a: 0xC0u8,
                b: 0xDEu8,
            },
        };
        test_roundtrip_with_args(
            &[0x03, 0x03, 0x04, 0xC0, 0xDE],
            multiple,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn switched_ordering() {
        let multiple = OffsetArrayContainer {
            offsets: [4u8, 3u8].into(),
            inner: Multiple {
                a: 0xC0u8,
                b: 0xDEu8,
            },
        };
        test_roundtrip_with_args(
            &[0x03, 0x04, 0x03, 0xDE, 0xC0],
            multiple,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
}
