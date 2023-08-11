// Copyright (c) 2023 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Wrapper types for dealing with XOR encryption.

use binrw::io::{Read, Result, Seek, SeekFrom, Write};
use std::iter::Cycle;

/// Stream cipher wrapper around another [`Read`]- or [`Write`]-able stream that XOR's all bytes
/// after reading/before writing.
#[derive(Debug)]
pub struct XorStream<T> {
    stream: T,
    key: Cycle<std::vec::IntoIter<u8>>,
    key_size: u64,
}

impl<T> XorStream<T> {
    /// Create a new XOR wrapper around `stream` that passes XOR's all data with `key` before
    /// forwarding it.
    pub fn with_key(stream: T, key: Vec<u8>) -> Self {
        let key = if key.is_empty() { vec![0] } else { key };
        let key_size = key.len() as u64;
        let key = key.into_iter().cycle();
        Self {
            stream,
            key,
            key_size,
        }
    }
}

impl<R: Read> Read for XorStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes_read = self.stream.read(buf)?;

        for byte in buf {
            let key_byte = self.key.next().unwrap_or(0);
            *byte ^= key_byte;
        }

        Ok(bytes_read)
    }
}

impl<W: Write> Write for XorStream<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let encrypted_buffer: Vec<u8> = buf
            .iter()
            .map(|x| {
                let key_byte = self.key.next().unwrap_or(0);
                x ^ key_byte
            })
            .collect();

        self.stream.write(&encrypted_buffer)
    }

    fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }
}

impl<S: Seek> Seek for XorStream<S> {
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        let old_position = self.stream.stream_position()?;
        let new_position = self.stream.seek(position)?;

        // Calculate how many bytes we need to move forward in the key stream to match the seek in
        // the actual buffer.
        let offset = if new_position > old_position {
            (new_position - old_position) % self.key_size
        } else {
            self.key_size - ((old_position - new_position) % self.key_size)
        };

        for _ in 0..offset {
            self.key.next();
        }

        Ok(new_position)
    }
}
