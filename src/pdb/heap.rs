// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::io::{self, Read, Seek, Write};

#[derive(PartialEq, Debug)]
enum HeapError {
    InsufficientSpace,
}

#[derive(Debug)]
struct Heap<'a> {
    top_size: usize,
    bottom_size: usize,
    data: &'a mut [u8],
}

impl<'a> Heap<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        Heap {
            top_size: 0,
            bottom_size: 0,
            data,
        }
    }

    fn with_size(data: &'a mut [u8], bottom_size: usize, top_size: usize) -> Self {
        Heap {
            top_size,
            bottom_size,
            data,
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn free_size(&self) -> usize {
        self.size() - self.bottom_size() - self.top_size()
    }

    fn bottom_size(&self) -> usize {
        self.bottom_size
    }

    fn top_size(&self) -> usize {
        self.top_size
    }

    fn position_bottom(&self) -> usize {
        self.bottom_size
    }

    fn position_top(&self) -> usize {
        self.size() - self.top_size
    }

    fn push_bottom(&mut self, buf: &[u8]) -> Result<usize, HeapError> {
        let start = self.position_bottom();
        let end = self.position_bottom() + buf.len();
        if end > self.position_top() {
            return Err(HeapError::InsufficientSpace);
        }
        self.data[start..end].copy_from_slice(buf);
        self.bottom_size = end;
        Ok(buf.len())
    }

    fn push_top(&mut self, buf: &[u8]) -> Result<usize, HeapError> {
        let start = self.position_top() - buf.len();
        let end = self.position_top();
        if start < self.position_bottom() {
            return Err(HeapError::InsufficientSpace);
        }
        self.data[start..end].copy_from_slice(buf);
        self.top_size += buf.len();
        Ok(buf.len())
    }

    fn pop_bottom(&mut self, size: usize) -> Result<(), HeapError> {
        if size > self.bottom_size() {
            return Err(HeapError::InsufficientSpace);
        }
        self.bottom_size = self.bottom_size() - size;
        Ok(())
    }

    fn pop_top(&mut self, size: usize) -> Result<(), HeapError> {
        if size > self.top_size() {
            return Err(HeapError::InsufficientSpace);
        }
        self.top_size = self.top_size() - size;
        Ok(())
    }

    pub fn bottom(&'a mut self) -> HeapBottom<'a> {
        HeapBottom {
            offset: 0,
            heap: self,
        }
    }
}

struct HeapBottom<'a> {
    offset: usize,
    heap: &'a mut Heap<'a>,
}

impl<'a> io::Read for HeapBottom<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset > self.heap.bottom_size() {
            return Ok(0);
        }

        let bytes_to_read = buf.len().min(self.heap.bottom_size() - self.offset);
        let new_offset = self.offset + bytes_to_read;

        buf[..bytes_to_read].copy_from_slice(&self.heap.data[self.offset..new_offset]);
        self.offset = new_offset;

        Ok(bytes_to_read)
    }
}

impl<'a> io::Write for HeapBottom<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_to_write = buf
            .len()
            .min(self.heap.bottom_size() + self.heap.free_size() - self.offset);
        if bytes_to_write == 0 {
            return Ok(0);
        }

        let new_offset = self.offset + bytes_to_write;
        self.heap.data[self.offset..new_offset].copy_from_slice(&buf[..bytes_to_write]);
        self.offset = new_offset;
        self.heap.bottom_size = self.heap.bottom_size.max(self.offset);

        Ok(bytes_to_write)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> io::Seek for HeapBottom<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        debug_assert!(self.offset <= self.heap.bottom_size());
        let result = match pos {
            io::SeekFrom::Start(offset) => {
                usize::try_from(offset).map_err(|_| io::ErrorKind::InvalidInput.into())
            }
            io::SeekFrom::End(offset) => isize::try_from(offset)
                .map_err(|_| io::ErrorKind::InvalidInput.into())
                .and_then(|ofs| {
                    self.heap
                        .bottom_size()
                        .checked_add_signed(ofs)
                        .ok_or(io::ErrorKind::InvalidInput.into())
                }),
            io::SeekFrom::Current(offset) => isize::try_from(offset)
                .map_err(|_| io::ErrorKind::InvalidInput.into())
                .and_then(|ofs| {
                    self.offset
                        .checked_add_signed(ofs)
                        .ok_or(io::ErrorKind::InvalidInput.into())
                }),
        }
        .map(|new_offset| new_offset.min(self.heap.bottom_size()));

        if let Ok(new_offset) = result {
            self.offset = new_offset;
        }

        result.and_then(|offset| {
            offset
                .try_into()
                .map_err(|_| io::ErrorKind::InvalidInput.into())
        })
    }
}

struct HeapTop<'a> {
    offset: usize,
    heap: &'a mut Heap<'a>,
}

impl<'a> HeapTop<'a> {
    fn resize(&mut self, size: usize) -> Result<u64, HeapError> {
        let old_size =
            u64::try_from(self.heap.top_size()).map_err(|_| HeapError::InsufficientSpace)?;
        let new_size = i64::try_from(size).map_err(|_| HeapError::InsufficientSpace)?;
        match new_size.checked_sub_unsigned(old_size) {
            Some(0) => Ok(0),
            Some(diff) => {
                self.heap.top_size = diff.try_into().and_then(|d| {
                    self.heap
                        .top_size
                        .checked_add_signed(d)
                        .ok_or(HeapError::InsufficientSpace)
                })?;
                self.offset.checked_add_signed(diff).or(0usize);
                Ok(diff)
            }
            None => Err(HeapError::InsufficientSpace),
        }
    }
}

impl<'a> io::Read for HeapTop<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset > self.heap.top_size() {
            return Ok(0);
        }

        let bytes_to_read = buf.len().min(self.heap.top_size() - self.offset);
        let new_offset = self.offset + bytes_to_read;

        let start = self.heap.position_top() + self.offset;
        let end = self.heap.position_top() + new_offset;
        buf[..bytes_to_read].copy_from_slice(&self.heap.data[start..end]);
        self.offset = new_offset;

        Ok(bytes_to_read)
    }
}

impl<'a> io::Write for HeapTop<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_to_write = buf.len().min(self.heap.top_size() - self.offset);
        if bytes_to_write == 0 {
            return Ok(0);
        }

        let new_offset = self.offset + bytes_to_write;
        let start = self.heap.position_top() + self.offset;
        let end = self.heap.position_top() + new_offset;
        self.heap.data[start..end].copy_from_slice(&buf[..bytes_to_write]);
        self.offset = new_offset;
        self.heap.top_size = self.heap.top_size.max(self.offset);

        Ok(bytes_to_write)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> io::Seek for HeapTop<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        debug_assert!(self.offset <= self.heap.top_size());
        let result = match pos {
            io::SeekFrom::Start(offset) => {
                usize::try_from(offset).map_err(|_| io::ErrorKind::InvalidInput.into())
            }
            io::SeekFrom::End(offset) => isize::try_from(offset)
                .map_err(|_| io::ErrorKind::InvalidInput.into())
                .and_then(|ofs| {
                    self.heap
                        .top_size()
                        .checked_add_signed(ofs)
                        .ok_or(io::ErrorKind::InvalidInput.into())
                }),
            io::SeekFrom::Current(offset) => isize::try_from(offset)
                .map_err(|_| io::ErrorKind::InvalidInput.into())
                .and_then(|ofs| {
                    self.offset
                        .checked_add_signed(ofs)
                        .ok_or(io::ErrorKind::InvalidInput.into())
                }),
        }
        .map(|new_offset| new_offset.min(self.heap.top_size()));

        if let Ok(new_offset) = result {
            self.offset = new_offset;
        }

        result.and_then(|offset| {
            offset
                .try_into()
                .map_err(|_| io::ErrorKind::InvalidInput.into())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_array<const N: usize>() -> [u8; 256] {
        let mut data = [0u8; 256];
        let mut counter: u8 = 0;
        data.fill_with(|| {
            counter = counter.overflowing_add(1).0;
            counter
        });
        data
    }

    #[test]
    fn test_raw_heap() {
        let mut data = make_array::<256>();
        let mut heap = Heap::new(&mut data);
        assert_eq!(heap.top_size(), 0);
        assert_eq!(heap.bottom_size(), 0);

        const BUF: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

        assert_eq!(heap.push_top(&BUF), Ok(BUF.len()));
        assert_eq!(heap.top_size(), 8);

        assert_eq!(heap.push_top(&BUF), Ok(BUF.len()));
        assert_eq!(heap.top_size(), 16);

        assert_eq!(heap.push_bottom(&BUF), Ok(BUF.len()));
        assert_eq!(heap.bottom_size(), 8);

        assert_eq!(heap.pop_top(4), Ok(()));
        assert_eq!(heap.top_size(), 12);

        assert_eq!(heap.pop_bottom(2), Ok(()));
        assert_eq!(heap.bottom_size(), 6);

        assert_eq!(heap.size(), 256);
        assert_eq!(heap.free_size(), 238);
    }

    #[test]
    fn test_read_bottom() {
        let mut data = make_array::<256>();
        let mut heap = Heap::with_size(&mut data, 12, 0);

        let mut bottom = heap.bottom();
        let mut buf = [0u8; 8];

        assert_eq!(bottom.read(&mut buf).unwrap(), 8);
        assert_eq!(&buf, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(bottom.read(&mut buf).unwrap(), 4);
        assert_eq!(&buf[..4], &[9, 10, 11, 12]);
    }

    #[test]
    fn test_seek_bottom() {
        let mut data = make_array::<256>();
        let mut heap = Heap::with_size(&mut data, 12, 0);

        let mut bottom = heap.bottom();
        assert_eq!(bottom.seek(io::SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(bottom.seek(io::SeekFrom::Start(10)).unwrap(), 10);
        assert_eq!(bottom.seek(io::SeekFrom::End(0)).unwrap(), 12);
        assert_eq!(bottom.seek(io::SeekFrom::End(-2)).unwrap(), 10);
        assert_eq!(bottom.seek(io::SeekFrom::Current(-2)).unwrap(), 8);
        assert_eq!(bottom.seek(io::SeekFrom::Current(4)).unwrap(), 12);
        assert_eq!(bottom.seek(io::SeekFrom::Current(-12)).unwrap(), 0);
    }

    #[test]
    fn test_write_bottom() {
        let mut data = [0u8; 32];
        let mut heap = Heap::with_size(&mut data, 12, 16);

        let buf: [u8; 5] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        let mut bottom = heap.bottom();
        assert_eq!(bottom.write(&buf).unwrap(), 5);
        assert_eq!(bottom.write(&buf).unwrap(), 5);
        assert_eq!(bottom.write(&buf).unwrap(), 5);
        assert_eq!(bottom.write(&buf).unwrap(), 1);
        assert_eq!(bottom.write(&buf).unwrap(), 0);
        assert_eq!(
            &data[..17],
            &[
                0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xAA, 0xBB, 0xCC, 0xDD,
                0xEE, 0xAA, 0x00
            ]
        );
    }
}
