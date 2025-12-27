// Copyright (c) 2025 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Parser for Pioneer DeviceSQL database exports (PDB).
//!
//! The Rekordbox DJ software uses writes PDB files to `/PIONEER/rekordbox/export.pdb`.
//!
//! Most of the file format has been reverse-engineered by Henry Betts, Fabian Lesniak and James
//! Elliott.
//!
//! - <https://github.com/Deep-Symmetry/crate-digger/blob/master/doc/Analysis.pdf>
//! - <https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html>
//! - <https://github.com/henrybetts/Rekordbox-Decoding>
//! - <https://github.com/flesniak/python-prodj-link/tree/master/prodj/pdblib>

use super::*;
use crate::util::{RekordcrateError, RekordcrateResult, TableIndex};
use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite, Endian};
use fallible_iterator::{FallibleIterator, IteratorExt};
use std::{
    collections::HashSet,
    io::{Read, Seek, Write},
};

/// A lazily loaded PDB database.
#[binrw]
#[brw(little)]
#[br(import(db_type: DatabaseType))]
#[derive(Debug, PartialEq)]
struct LazyDatabase {
    /// The PDB header.
    #[br(args(db_type))]
    #[bw(pad_size_to = header.page_size as usize)]
    header: Header,
    /// The pages of the database, initially not loaded.
    #[br(calc = vec![LazyPage::NotLoaded; (header.next_unused_page.0 - 1) as usize])]
    #[bw(args(header.page_size))]
    pages: Vec<LazyPage>,
}

#[derive(Debug, PartialEq, Clone)]
enum LazyPage {
    NotLoaded,
    Loaded(Page),
}

impl BinWrite for LazyPage {
    type Args<'a> = (u32,);

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (page_size,): Self::Args<'_>,
    ) -> BinResult<()> {
        match self {
            LazyPage::NotLoaded => {
                // Just seek forward without writing anything.
                writer.seek(SeekFrom::Current(page_size as i64))?;
                Ok(())
            }
            LazyPage::Loaded(page) => page.write_options(writer, endian, (page_size,)),
        }
    }
}

/// A PDB database opened for reading or writing.
#[derive(Debug)]
pub struct Database<IO> {
    io: IO,
    db_type: DatabaseType,
    content: LazyDatabase,
}

impl<R: Read + Seek> Database<R> {
    /// Opens a PDB database without writing back to disk.
    /// Still allows modifying data in memory.
    pub fn open_non_persistent(mut io: R, db_type: DatabaseType) -> RekordcrateResult<Self> {
        let endian = Endian::Little;
        let content = LazyDatabase::read_options(&mut io, endian, (db_type,))?;
        Ok(Self {
            io,
            db_type,
            content,
        })
    }

    /// Loads a page into memory.
    pub fn load_page(&mut self, index: PageIndex) -> RekordcrateResult<&mut Page> {
        let endian = Endian::Little;
        let page_entry = self
            .content
            .pages
            .get_mut(index.0 as usize - 1)
            .ok_or_else(|| RekordcrateError::PageNotPresent(index))?;
        if let LazyPage::NotLoaded = page_entry {
            // Load the page from the file
            let page_offset = SeekFrom::Start(index.offset(self.content.header.page_size));
            self.io.seek(page_offset).map_err(binrw::Error::Io)?;
            let page = Page::read_options(
                &mut self.io,
                endian,
                (self.content.header.page_size, self.db_type),
            )?;
            *page_entry = LazyPage::Loaded(page);
        }
        match page_entry {
            LazyPage::Loaded(page) => Ok(page),
            _ => unreachable!(),
        }
    }

    /// Loads all pages for a table into memory and iterates over them.
    pub fn iter_pages_for_table<'db>(
        &'db mut self,
        table_index: TableIndex,
    ) -> RekordcrateResult<PageIterator<'db, R>> {
        let table = self
            .get_header()
            .tables
            .get(table_index.0)
            .ok_or_else(|| RekordcrateError::TableNotPresent(table_index))?;
        let (first_page, last_page) = (table.first_page, table.last_page);

        Ok(PageIterator {
            db: self,
            next_page: Some(first_page),
            last_page,
            seen_pages: HashSet::new(),
        })
    }

    /// Loads all pages for a page type into memory and iterates over them.
    pub fn iter_pages(&mut self, page_type: PageType) -> RekordcrateResult<PageIterator<'_, R>> {
        let (_, table) = self
            .get_header()
            .find_table(page_type)
            .ok_or_else(|| RekordcrateError::TableTypeNotPresent(page_type))?;
        let (first_page, last_page) = (table.first_page, table.last_page);

        Ok(PageIterator {
            db: self,
            next_page: Some(first_page),
            last_page,
            seen_pages: HashSet::new(),
        })
    }

    /// Loads all pages for a page type into memory and iterates over their data rows.
    pub fn iter_rows<'a, RowT: RowVariant + 'a>(
        &'a mut self,
    ) -> RekordcrateResult<impl FallibleIterator<Item = &'a mut RowT, Error = RekordcrateError>>
    {
        Ok(self
            .iter_pages(RowT::PAGE_TYPE)?
            .filter_map(|page| Ok(page.content.as_data_mut()))
            .flat_map(|dpc| {
                Ok(dpc
                    .rows
                    .values_mut()
                    .into_fallible()
                    .map_err(|_: core::convert::Infallible| unreachable!()))
            })
            // The parsed row type is determined from the page type, so if we find an unexpected
            // variant then there is a code bug (not simply a corrupt DB).
            .map(|row| Ok(row.as_variant_mut().expect("unexpected row type"))))
    }

    /// Returns a reference to the PDB header.
    #[must_use]
    pub fn get_header(&self) -> &Header {
        &self.content.header
    }

    /// Returns a mutable reference to the PDB header.
    #[must_use]
    pub fn get_header_mut(&mut self) -> &mut Header {
        &mut self.content.header
    }
}

impl<RW: Read + Write + Seek> Database<RW> {
    /// Opens a PDB database for reading and writing.
    pub fn open(mut io: RW, db_type: DatabaseType) -> RekordcrateResult<Self> {
        let endian = Endian::Little;
        let content = LazyDatabase::read_options(&mut io, endian, (db_type,))?;
        Ok(Self {
            io,
            db_type,
            content,
        })
    }

    /// Flushes all changes to the underlying IO.
    pub fn flush(&mut self) -> RekordcrateResult<()> {
        let endian = Endian::Little;
        self.io.seek(SeekFrom::Start(0))?;
        self.content.write_options(&mut self.io, endian, ())?;
        Ok(())
    }

    /// Closes the database, flushing changes.
    pub fn close(mut self) -> RekordcrateResult<()> {
        self.flush()?;
        Ok(())
    }
}

/// An iterator over pages in a PDB database.
///
/// We use `FallibleIterator` instead of the standard `Iterator` trait
/// to improve the ergonomics of error handling while loading pages.
///
/// # Usage
///
/// ```no_run
/// # use rekordcrate::pdb::*;
/// # use rekordcrate::util::RekordcrateError;
/// # use rekordcrate::pdb::io::Database;
/// use fallible_iterator::FallibleIterator;
///
/// # let mut db: Database<std::fs::File> = unimplemented!();
/// // Loop over pages.
/// let mut page_iter = db.iter_pages(PageType::Plain(PlainPageType::Tracks))?;
/// while let Some(page) = page_iter.next()? {
///     // Process the page
/// }
///
/// // Iterate over pages using typical functional combinators.
/// // Note that combinators like `map` should return a `Result`.
/// let results: Vec<_> = db
///     .iter_pages(PageType::Plain(PlainPageType::Tracks))?
///     .map(|page| Ok(todo!()))
///     .collect()?;
/// # Ok::<(), RekordcrateError>(())
/// ```
///
/// # Safety
///
/// The unsafe code is necessary to achieve the required lifetime
/// bounds on the returned page references, despite the `next()` method
/// having a self lifetime only valid until the next call. This is
/// known as the "lending iterator" problem in Rust.
///
/// We have attempted to avoid this by using the `lending-iterator`,
/// `gat-lending-iterator` or `lender` crates, but they have
/// usability and soundness issues.
#[derive(Debug)]
pub struct PageIterator<'db, IO> {
    db: &'db mut Database<IO>,
    next_page: Option<PageIndex>,
    last_page: PageIndex,
    seen_pages: HashSet<PageIndex>,
}

impl<'db, R: Read + Seek> FallibleIterator for PageIterator<'db, R> {
    type Item = &'db mut Page;
    type Error = RekordcrateError;

    /// Loads the next page in the iterator.
    fn next<'call>(&'call mut self) -> RekordcrateResult<Option<&'db mut Page>> {
        match self.next_page {
            None => Ok(None),
            Some(page_index) => {
                if !self.seen_pages.insert(page_index) {
                    return Err(RekordcrateError::PageCycle(page_index));
                }
                let page: &'call mut Page = self.db.load_page(page_index)?;
                // SAFETY: extending the lifetime of `page` to `'db` is safe while
                // the following conditions are met:
                //
                // 1. We never attempt to load or create a reference to a page after
                //    we have already handed out a `&'db mut Page` to that page.
                //    This is ensured by `seen_pages` which ensures
                //    that we only touch unique pages.
                // 2. The `&'db mut Database` reference cannot be extracted from the iterator
                //    so that it cannot be used to create conflicting references with
                //    the `&'db mut Page` references that we hand out. This is ensured by
                //    the private field `db` and the lack of any methods returning it.
                let page: &'db mut Page = unsafe { std::mem::transmute(page) };
                if page_index == self.last_page {
                    self.next_page = None;
                } else {
                    self.next_page = Some(page.header.next_page);
                }
                Ok(Some(page))
            }
        }
    }
}
