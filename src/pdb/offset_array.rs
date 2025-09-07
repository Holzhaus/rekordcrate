// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetSize {
    U8,
    U16,
}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[br(import(start_offset: usize, offset_size: OffsetSize, inner_args: <T as BinRead>::Args<'_>))]
#[bw(import(start_offset: usize, offset_size: OffsetSize, inner_args: <T as BinWrite>::Args<'_>))]
pub struct OffsetArray<T: BinRead + BinWrite, const N: usize>
where
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
{
    #[br(args(offset_size))]
    #[brw(assert(offsets.assert_offset_size_matches(offset_size)))]
    pub offsets: OffsetArrayImpl<N>,
    #[br(parse_with = OffsetArrayImpl::<N>::read_offsets, args(&offsets, start_offset, inner_args))]
    #[bw(write_with = OffsetArrayImpl::<N>::write_with_offsets, args(offsets, start_offset, inner_args))]
    pub inner: Vec<T>, // This has always N elements, but making it a proper array is difficult
}

// impl<T: BinRead + BinWrite, const N: usize> binrw::meta::WriteEndian for OffsetArray<T, N>
// where
//     for<'a> <T as BinRead>::Args<'a>: Clone,
//     for<'a> <T as BinWrite>::Args<'a>: Clone,
// {
//     const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
// }
// impl<T: BinRead + BinWrite, const N: usize> binrw::meta::ReadEndian for OffsetArray<T, N>
// where
//     for<'a> <T as BinRead>::Args<'a>: Clone,
//     for<'a> <T as BinWrite>::Args<'a>: Clone,
// {
//     const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
// }

#[binrw]
#[derive(Debug, Clone, PartialEq, Eq)]
#[brw(little)]
#[br(import(size: OffsetSize))]
pub enum OffsetArrayImpl<const N: usize> {
    #[br(pre_assert(size == OffsetSize::U8))]
    U8([u8; N]),
    #[br(pre_assert(size == OffsetSize::U16))]
    U16([u16; N]),
}

impl<const N: usize> OffsetArrayImpl<N> {
    pub fn byte_size(&self) -> usize {
        match self {
            OffsetArrayImpl::U8(_) => N,
            OffsetArrayImpl::U16(_) => N * 2,
        }
    }
    pub(crate) fn assert_offset_size_matches(&self, offset_size: OffsetSize) -> bool {
        matches!(
            (self, offset_size),
            (Self::U8(_), OffsetSize::U8) | (Self::U16(_), OffsetSize::U16)
        )
    }
    #[binrw::parser(reader, endian)]
    pub fn read_offsets<T: BinRead>(
        offsets: &Self,
        start_offset: usize,
        inner_args: T::Args<'_>,
    ) -> BinResult<Vec<T>>
    where
        for<'a> T::Args<'a>: Clone,
    {
        let stream_pos = reader.stream_position()?;
        let size_byte = offsets.byte_size();
        let base =
            stream_pos
                .checked_sub(size_byte as u64)
                .ok_or_else(|| binrw::Error::AssertFail {
                    pos: stream_pos,
                    message: format!("Stream position overflow: {stream_pos}-{size_byte}"),
                })?;
        match offsets {
            OffsetArrayImpl::U8(offsets) => {
                // if we had this: https://doc.rust-lang.org/std/array/fn.try_from_fn.html
                // we could return an array instead
                offsets
                    .iter()
                    .map(|&offset| {
                        reader.seek(SeekFrom::Start(
                            base + u64::from(offset) - (start_offset as u64),
                        ))?;
                        T::read_options(reader, endian, inner_args.clone())
                    })
                    .collect()
            }
            OffsetArrayImpl::U16(offsets) => {
                // if we had this: https://doc.rust-lang.org/std/array/fn.try_from_fn.html
                // we could return an array instead
                offsets
                    .iter()
                    .map(|&offset| {
                        reader.seek(SeekFrom::Start(
                            base + u64::from(offset) - (start_offset as u64),
                        ))?;
                        T::read_options(reader, endian, inner_args.clone())
                    })
                    .collect()
            }
        }
    }

    #[binrw::writer(writer, endian)]
    pub fn write_with_offsets<T: BinWrite>(
        elements: &Vec<T>,
        offsets: &Self,
        start_offset: usize,
        inner_args: T::Args<'_>,
    ) -> BinResult<()>
    where
        for<'a> T::Args<'a>: Clone,
    {
        let stream_pos = writer.stream_position()?;
        let size_byte = offsets.byte_size();
        dbg!(start_offset, offsets);
        let base =
            stream_pos
                .checked_sub(size_byte as u64)
                .ok_or_else(|| binrw::Error::AssertFail {
                    pos: stream_pos,
                    message: format!("Stream position overflow: {stream_pos}-{size_byte}"),
                })?;
        match offsets {
            OffsetArrayImpl::U8(offsets) => {
                for (&offset, elem) in offsets.iter().zip(elements) {
                    writer.seek(SeekFrom::Start(
                        base + u64::from(offset) - (start_offset as u64),
                    ))?;
                    elem.write_options(writer, endian, inner_args.clone())?;
                }
            }
            OffsetArrayImpl::U16(offsets) => {
                for (&offset, elem) in offsets.iter().zip(elements) {
                    writer.seek(SeekFrom::Start(
                        base + u64::from(offset) - (start_offset as u64),
                    ))?;
                    elem.write_options(writer, endian, inner_args.clone())?;
                }
            }
        }
        Ok(())
    }
}

impl<const N: usize> From<[u8; N]> for OffsetArrayImpl<N> {
    fn from(arr: [u8; N]) -> Self {
        Self::U8(arr)
    }
}

impl<const N: usize> From<[u16; N]> for OffsetArrayImpl<N> {
    fn from(arr: [u16; N]) -> Self {
        Self::U16(arr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::testing::test_roundtrip_with_args;
    #[test]
    fn empty() {
        let empty_offset_tail_u8 = OffsetArray {
            offsets: OffsetArrayImpl::U8([]),
            inner: vec![(); 0],
        };
        test_roundtrip_with_args(
            &[],
            empty_offset_tail_u8,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
        let empty_offset_tail_u16 = OffsetArray {
            offsets: OffsetArrayImpl::U16([]),
            inner: vec![(); 0],
        };
        test_roundtrip_with_args(
            &[],
            empty_offset_tail_u16,
            (0, OffsetSize::U16, ()),
            (0, OffsetSize::U16, ()),
        );
    }
    #[test]
    fn near_u8() {
        let near_u8_tail = OffsetArray {
            offsets: [1u8].into(),
            inner: vec![42u8],
        };
        test_roundtrip_with_args(
            &[0x01, 42],
            near_u8_tail,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn buffer() {
        let buffer = OffsetArray {
            offsets: [1u8].into(),
            inner: vec![0xDEADBEEF_u32.to_be_bytes()],
        };
        test_roundtrip_with_args(
            &[0x01, 0xDE, 0xAD, 0xBE, 0xEF],
            buffer,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn near_remote() {
        let near_remote = OffsetArray {
            offsets: [4u8].into(),
            inner: vec![42u8],
        };
        test_roundtrip_with_args(
            &[0x04, 0x00, 0x00, 0x00, 42],
            near_remote,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn far_remote() {
        let far_remote = OffsetArray {
            offsets: [3u16].into(),
            inner: vec![42u8],
        };
        test_roundtrip_with_args(
            &[0x03, 0x00, 0x00, 42],
            far_remote,
            (0, OffsetSize::U16, ()),
            (0, OffsetSize::U16, ()),
        );
    }
    #[test]
    fn near_offset() {
        let near_offset = OffsetArray {
            offsets: [4u8].into(),
            inner: vec![42u8],
        };
        let offset = 3;
        test_roundtrip_with_args(
            &[0x04, 42],
            near_offset,
            (offset, OffsetSize::U8, ()),
            (offset, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn multiple() {
        let multiple = OffsetArray {
            offsets: [2u8, 3u8].into(),
            inner: vec![0xC0u8, 0xDEu8],
        };
        test_roundtrip_with_args(
            &[0x02, 0x03, 0xC0, 0xDE],
            multiple,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
    #[test]
    fn switched_ordering() {
        let multiple = OffsetArray {
            offsets: [3u8, 2u8].into(),
            inner: vec![0xC0u8, 0xDEu8],
        };
        test_roundtrip_with_args(
            &[0x03, 0x02, 0xDE, 0xC0],
            multiple,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
}
