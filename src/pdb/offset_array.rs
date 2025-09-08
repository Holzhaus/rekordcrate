// Copyright (c) 2025 Nikolaus Einhauser <nikolaus.einhauser@web.de>
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

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct OffsetArray<T, const N: usize> {
    pub offsets: OffsetArrayImpl<N>,
    pub inner: T,
}

impl<T, const N: usize> std::ops::Deref for OffsetArray<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &(self.inner)
    }
}

impl<T: BinRead + BinWrite, const N: usize> binrw::meta::WriteEndian for OffsetArray<T, N>
where
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
{
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
}
impl<T: BinRead + BinWrite, const N: usize> binrw::meta::ReadEndian for OffsetArray<T, N>
where
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
{
    const ENDIAN: binrw::meta::EndianKind = binrw::meta::EndianKind::Endian(binrw::Endian::Little);
}

impl<T, const N: usize, IA> BinRead for OffsetArray<T, N>
where
    for<'a> T: BinRead<Args<'a> = (i64, &'a OffsetArrayImpl<N>, IA)>,
{
    type Args<'a> = (usize, OffsetSize, IA);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (offset, offset_size, args): Self::Args<'_>,
    ) -> BinResult<Self> {
        let start = reader.stream_position()?;
        let offsets = OffsetArrayImpl::<N>::read_options(reader, endian, (offset_size,))?;
        if !offsets.assert_offset_size_matches(offset_size) {
            return Err(binrw::Error::AssertFail {
                pos: start,
                message: format!("offsetsize mismatch! {offset_size:?}"),
            });
        }
        let base = i64::try_from(start).map_err(|err| binrw::Error::AssertFail {
            pos: start,
            message: format!("{err}"),
        })?;
        let offset = i64::try_from(offset).map_err(|err| binrw::Error::AssertFail {
            pos: start,
            message: format!("{err}"),
        })?;
        let base = base
            .checked_sub(offset)
            .ok_or_else(|| binrw::Error::AssertFail {
                pos: start,
                message: format!("Stream position underflow: {start}-{offset}"),
            })?;
        let inner = T::read_options(reader, endian, (base, &offsets, args))?;
        Ok(Self { offsets, inner })
    }
}

impl<T, const N: usize, IA> BinWrite for OffsetArray<T, N>
where
    for<'a> T: BinWrite<Args<'a> = (i64, &'a OffsetArrayImpl<N>, IA)>,
{
    type Args<'a> = (usize, OffsetSize, IA);

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
        let base = i64::try_from(start).map_err(|err| binrw::Error::AssertFail {
            pos: start,
            message: format!("{err}"),
        })?;
        let offset = i64::try_from(offset).map_err(|err| binrw::Error::AssertFail {
            pos: start,
            message: format!("{err}"),
        })?;
        let base = base
            .checked_sub(offset)
            .ok_or_else(|| binrw::Error::AssertFail {
                pos: start,
                message: format!("Stream position underflow: {start}-{offset}"),
            })?;
        self.inner
            .write_options(writer, endian, (base, &self.offsets, args))?;
        Ok(())
    }
}

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
    fn assert_offset_size_matches(&self, offset_size: OffsetSize) -> bool {
        matches!(
            (self, offset_size),
            (Self::U8(_), OffsetSize::U8) | (Self::U16(_), OffsetSize::U16)
        )
    }

    pub fn read_offset<'a, T: BinRead, R: binrw::io::Read + binrw::io::Seek>(
        &'a self,
        index: usize,
    ) -> impl FnOnce(&mut R, binrw::Endian, (i64, T::Args<'_>)) -> BinResult<T> + 'a {
        move |reader, endian, (base, inner_args)| match self {
            OffsetArrayImpl::U8(offsets) => {
                let offset = offsets.get(index).ok_or_else(|| binrw::Error::AssertFail {
                    pos: base.try_into().unwrap_or_default(),
                    message: format!("can't get offset at index {index} for offsets {offsets:?}"),
                })?;
                let start = (base + i64::from(*offset)).try_into().map_err(|err| {
                    binrw::Error::AssertFail {
                        pos: base.try_into().unwrap_or_default(),
                        message: format!("{err}: {base}+{offset}"),
                    }
                })?;
                reader.seek(SeekFrom::Start(start))?;
                T::read_options(reader, endian, inner_args)
            }
            OffsetArrayImpl::U16(offsets) => {
                let offset = offsets.get(index).ok_or_else(|| binrw::Error::AssertFail {
                    pos: base.try_into().unwrap_or_default(),
                    message: format!("can't get offset at index {index} for offsets {offsets:?}"),
                })?;
                let start = (base + i64::from(*offset)).try_into().map_err(|err| {
                    binrw::Error::AssertFail {
                        pos: base.try_into().unwrap_or_default(),
                        message: format!("{err}: {base}+{offset}"),
                    }
                })?;
                reader.seek(SeekFrom::Start(start))?;
                T::read_options(reader, endian, inner_args)
            }
        }
    }

    pub fn write_offset<'a, T: BinWrite, R: binrw::io::Write + binrw::io::Seek>(
        &'a self,
        index: usize,
    ) -> impl FnOnce(&T, &mut R, binrw::Endian, (i64, T::Args<'_>)) -> BinResult<()> + 'a {
        move |element, writer, endian, (base, inner_args)| match self {
            OffsetArrayImpl::U8(offsets) => {
                let offset = offsets.get(index).ok_or_else(|| binrw::Error::AssertFail {
                    pos: base.try_into().unwrap_or_default(),
                    message: format!("can't get offset at index {index} for offsets {offsets:?}"),
                })?;
                let start = (base + i64::from(*offset)).try_into().map_err(|err| {
                    binrw::Error::AssertFail {
                        pos: base.try_into().unwrap_or_default(),
                        message: format!("{err}: {base}+{offset}"),
                    }
                })?;
                writer.seek(SeekFrom::Start(start))?;
                element.write_options(writer, endian, inner_args)
            }
            OffsetArrayImpl::U16(offsets) => {
                let offset = offsets.get(index).ok_or_else(|| binrw::Error::AssertFail {
                    pos: base.try_into().unwrap_or_default(),
                    message: format!("can't get offset at index {index} for offsets {offsets:?}"),
                })?;
                let start = (base + i64::from(*offset)).try_into().map_err(|err| {
                    binrw::Error::AssertFail {
                        pos: base.try_into().unwrap_or_default(),
                        message: format!("{err}: {base}+{offset}"),
                    }
                })?;
                writer.seek(SeekFrom::Start(start))?;
                element.write_options(writer, endian, inner_args)
            }
        }
    }
    // pub fn read_offsets<'a, T: BinRead, R: binrw::io::Read + binrw::io::Seek>(&'a self)
    //     -> impl FnOnce(&mut R, binrw::Endian, (i64, T::Args<'_>)) -> BinResult<[T; N]> + 'a
    // where
    //     for<'b> T::Args<'b>: Clone
    // {
    //     move |reader, endian, args| {
    //         (0..N).map(|i| self.read_offset(i)(reader, endian, args.clone())).collect().map()
    //     }
    // }

    // pub fn write_offsets<'a, T: BinWrite, R: binrw::io::Write + binrw::io::Seek>(&'a self)
    //     -> impl FnOnce(&[T], &mut R, binrw::Endian, (i64, T::Args<'_>)) -> BinResult<()> + 'a
    // where
    //     for<'b> T::Args<'b>: Clone
    // {
    //     move |elems, reader, endian, args| {
    //         (0..N).zip(elems).map(|(i, elem)|
    //             self.write_offset(i)(elem, reader, endian, args.clone())
    //         ).collect()
    //     }
    // }
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

    use binrw::VecArgs;

    use super::*;
    use crate::util::testing::test_roundtrip_with_args;

    #[binrw]
    #[brw(little)]
    #[br(import(_base: i64, _offsets: &OffsetArrayImpl<N>, args: <T as BinRead>::Args<'_>))]
    #[bw(import(_base: i64, _offsets: &OffsetArrayImpl<N>, args: <T as BinWrite>::Args<'_>))]
    #[derive(Debug, PartialEq)]
    struct IgnoreArgs<T: BinRead + BinWrite, const N: usize>(#[brw(args_raw = args)] T);

    #[binrw]
    #[brw(little)]
    #[br(import(base: i64, offsets: &OffsetArrayImpl<1>, args: <T as BinRead>::Args<'_>))]
    #[bw(import(base: i64, offsets: &OffsetArrayImpl<1>, args: <T as BinWrite>::Args<'_>))]
    #[derive(Debug, PartialEq)]
    // This could also be used outside of tests, it just isn't yet (though a version is, called "TrailingName")
    pub struct SingleTarget<T: BinRead + BinWrite>(
        #[brw(args(base, args))]
        #[br(parse_with = offsets.read_offset(0))]
        #[bw(write_with = offsets.write_offset(0))]
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
        let empty_offset_tail_u8 = OffsetArray {
            offsets: OffsetArrayImpl::U8([]),
            inner: IgnoreArgs(()),
        };
        test_roundtrip_with_args(
            &[],
            empty_offset_tail_u8,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
        let empty_offset_tail_u16 = OffsetArray {
            offsets: OffsetArrayImpl::U16([]),
            inner: IgnoreArgs(vec![(); 0]),
        };
        test_roundtrip_with_args(
            &[],
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
        let near_u8_tail = OffsetArray {
            offsets: [1u8].into(),
            inner: SingleTarget(42u8),
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
            inner: SingleTarget(0xDEADBEEF_u32.to_be_bytes()),
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
            inner: SingleTarget(42u8),
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
            inner: SingleTarget(42u8),
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
            inner: SingleTarget(42u8),
        };
        let offset = 3;
        test_roundtrip_with_args(
            &[0x04, 42],
            near_offset,
            (offset, OffsetSize::U8, ()),
            (offset, OffsetSize::U8, ()),
        );
    }

    #[binrw]
    #[brw(little)]
    #[br(import(base: i64, offsets: &OffsetArrayImpl<N>, args: <T as BinRead>::Args<'_>))]
    #[bw(import(base: i64, offsets: &OffsetArrayImpl<N>, args: <T as BinWrite>::Args<'_>))]
    #[derive(Debug, PartialEq)]
    struct Multiple<T: BinRead + BinWrite, const N: usize>
    where
        for<'a> <T as BinRead>::Args<'a>: Clone,
        for<'a> <T as BinWrite>::Args<'a>: Clone,
    {
        #[brw(args(base, args.clone()))]
        #[br(parse_with = offsets.read_offset(0))]
        #[bw(write_with = offsets.write_offset(0))]
        a: T,
        #[brw(args(base, args))]
        #[br(parse_with = offsets.read_offset(1))]
        #[bw(write_with = offsets.write_offset(1))]
        b: T,
    }

    #[test]
    fn multiple() {
        let multiple = OffsetArray {
            offsets: [2u8, 3u8].into(),
            inner: Multiple {
                a: 0xC0u8,
                b: 0xDEu8,
            },
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
            inner: Multiple {
                a: 0xC0u8,
                b: 0xDEu8,
            },
        };
        test_roundtrip_with_args(
            &[0x03, 0x02, 0xDE, 0xC0],
            multiple,
            (0, OffsetSize::U8, ()),
            (0, OffsetSize::U8, ()),
        );
    }
}
