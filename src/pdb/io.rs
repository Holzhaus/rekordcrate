// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
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
//!
//! # Editing existing rows
//!
//! `Database::open` + `iter_rows` + `close` round-trips edits to disk, but each row is written
//! back at its **original fixed offset** — the page heap is never repacked.
//!
//! - **Size-stable edits are safe.** Mutating a fixed-width scalar field leaves the row's
//!   serialized length unchanged, so it lands cleanly in its slot.
//! - **Length-changing string edits corrupt the page**, and this is *not* detected: a longer
//!   string overwrites the next row, a shorter one leaves stale bytes. Grow or add strings by
//!   appending a new row via [`Database::add_row`] or [`crate::DeviceExportWriter`] instead.
//!
//! `flush`/`close` only re-check the 221-byte minimum track row size; they cannot detect heap
//! overflow.

use super::*;
use crate::util::{RekordcrateError, RekordcrateResult, TableIndex};
use binrw::{binrw, io::SeekFrom, BinRead, BinResult, BinWrite, Endian};
use fallible_iterator::{FallibleIterator, IteratorExt};
use std::io::{Read, Seek, Write};

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
    #[br(calc = vec![LazyPage::NotLoaded; header.next_unused_page.0.saturating_sub(1) as usize])]
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

fn read_page<IO: Read + Seek>(
    io: &mut IO,
    page_index: PageIndex,
    page_size: u32,
    db_type: DatabaseType,
) -> RekordcrateResult<Page> {
    let endian = Endian::Little;
    let page_offset = SeekFrom::Start(page_index.offset(page_size));
    io.seek(page_offset).map_err(binrw::Error::Io)?;
    let page = Page::read_options(io, endian, (page_size, db_type))?;
    Ok(page)
}

/// A PDB database opened for reading or writing.
#[derive(Debug)]
pub struct Database<IO> {
    io: IO,
    db_type: DatabaseType,
    content: LazyDatabase,
}

/// Reference to a row stored in a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowRef {
    /// Index of the page that contains the row.
    pub page_index: PageIndex,
    /// Byte offset of the row inside the page data section.
    pub row_offset: u16,
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
        let page_entry = self
            .content
            .pages
            .get_mut(index.0 as usize - 1)
            .ok_or_else(|| RekordcrateError::PageNotPresent(index))?;
        if let LazyPage::NotLoaded = page_entry {
            let page = read_page(
                &mut self.io,
                index,
                self.content.header.page_size,
                self.db_type,
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
            db_pages: self.content.pages.as_mut_slice(),
            db_pages_offset: 1, // Page indices are 1-based, so the first page is at offset 0 in the slice.
            db_io: &mut self.io,
            db_page_size: self.content.header.page_size,
            db_type: self.db_type,
            next_page: Some(first_page),
            last_page,
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
            db_pages: self.content.pages.as_mut_slice(),
            db_pages_offset: 1, // Page indices are 1-based, so the first page is at offset 0 in the slice.
            db_io: &mut self.io,
            db_page_size: self.content.header.page_size,
            db_type: self.db_type,
            next_page: Some(first_page),
            last_page,
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
    const DEFAULT_PAGE_SIZE: u32 = 4096;
    const PAGE_CHAIN_END: PageIndex = PageIndex(0x03FF_FFFF);
    const MIN_TRACK_ALLOCATED_SIZE: u16 = 221;

    fn allocated_row_size(row_size: u16) -> u16 {
        row_size.next_multiple_of(4)
    }

    pub(crate) fn validate_track_row_size(track: &Track) -> RekordcrateResult<()> {
        let allocated = Self::allocated_row_size(track.heap_bytes_required(()));
        if allocated < Self::MIN_TRACK_ALLOCATED_SIZE {
            return Err(RekordcrateError::TrackRowTooSmall {
                track_id: track.id.0,
                allocated,
                minimum: Self::MIN_TRACK_ALLOCATED_SIZE,
            });
        }
        Ok(())
    }

    fn validate_all_track_rows(&mut self) -> RekordcrateResult<()> {
        if self.db_type != DatabaseType::Plain {
            return Ok(());
        }

        let mut pages = self.iter_pages(PageType::Plain(PlainPageType::Tracks))?;
        while let Some(page) = pages.next()? {
            let data = match &page.content {
                PageContent::Data(data) => data,
                PageContent::Index(_) => continue,
            };

            for row in data.rows.values() {
                if let Row::Plain(PlainRow::Track(track)) = row {
                    Self::validate_track_row_size(track)?;
                }
            }
        }

        Ok(())
    }

    /// Points the previous page's `next_page` field at `current_page_index`.
    ///
    fn relink_chain_end(
        &mut self,
        previous_page_index: PageIndex,
        current_page_index: PageIndex,
    ) -> RekordcrateResult<()> {
        let previous_page = self.load_page(previous_page_index)?;
        previous_page.header.next_page = current_page_index;
        // Keep both copies of `next_page` in index pages in sync.
        if let PageContent::Index(ref mut index_content) = previous_page.content {
            index_content.header.next_page = current_page_index;
        }
        Ok(())
    }

    /// Creates a new empty PDB database.
    ///
    /// Initializes each table with an index page followed by an empty data page.
    pub fn create(
        io: RW,
        db_type: DatabaseType,
        table_page_types: &[PageType],
    ) -> RekordcrateResult<Self> {
        let mut pages = Vec::with_capacity(table_page_types.len() * 2);
        let mut tables = Vec::with_capacity(table_page_types.len());

        let mut next_page = PageIndex::try_from(1)?;
        for &page_type in table_page_types {
            let index_page_index = next_page;
            next_page = PageIndex::try_from(next_page.0 + 1)?;

            let data_page_index = next_page;
            next_page = PageIndex::try_from(next_page.0 + 1)?;

            let mut index_page = Page::new_index(index_page_index, page_type, Self::PAGE_CHAIN_END);
            if let PageContent::Index(ref mut index_content) = index_page.content {
                index_content.header.next_page = Self::PAGE_CHAIN_END;
            }
            let data_page = Page::new_data(
                Self::DEFAULT_PAGE_SIZE,
                data_page_index,
                page_type,
                Self::PAGE_CHAIN_END,
            );

            tables.push(Table {
                page_type,
                empty_candidate: data_page_index.0,
                first_page: index_page_index,
                // Empty table: logical chain tail is the index page.
                last_page: index_page_index,
            });

            pages.push(LazyPage::Loaded(index_page));
            pages.push(LazyPage::Loaded(data_page));
        }

        let num_tables = table_page_types
            .len()
            .try_into()
            .map_err(|_| RekordcrateError::IntegrityError("too many tables"))?;
        let header = Header {
            page_size: Self::DEFAULT_PAGE_SIZE,
            num_tables,
            next_unused_page: next_page,
            unknown: 5,
            sequence: 1,
            tables,
        };

        Ok(Self {
            io,
            db_type,
            content: LazyDatabase { header, pages },
        })
    }

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
        self.validate_all_track_rows()?;
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

    /// Allocates a new empty data page and returns its page index.
    fn alloc_data_page(&mut self, page_type: PageType) -> RekordcrateResult<PageIndex> {
        let page_index = self.content.header.next_unused_page;
        self.content.header.next_unused_page = PageIndex::try_from(page_index.0 + 1)?;

        let page = Page::new_data(
            self.content.header.page_size,
            page_index,
            page_type,
            Self::PAGE_CHAIN_END,
        );
        self.content.pages.push(LazyPage::Loaded(page));

        Ok(page_index)
    }

    /// Tries to append a row to an existing page's heap. Returns `None` if the page
    /// can't hold it (index page, or no free space).
    fn try_insert_row(
        &mut self,
        page_index: PageIndex,
        row_size: u16,
        row: &mut Option<Row>,
    ) -> RekordcrateResult<Option<RowRef>> {
        let page = self.load_page(page_index)?;
        let row_offset = page.header.used_size;
        let insert = match page.allocate_row(row_size) {
            Some(insert) => insert,
            None => return Ok(None),
        };
        insert(row.take().expect("row should still be pending"));
        Ok(Some(RowRef {
            page_index,
            row_offset,
        }))
    }

    /// Adds a row to the corresponding table, allocating a new page when needed.
    pub fn add_row(&mut self, row: Row) -> RekordcrateResult<RowRef> {
        if let Row::Plain(PlainRow::Track(track)) = &row {
            Self::validate_track_row_size(track)?;
        }

        let page_type = row.page_type()?;
        let row_size = row.heap_bytes_required(());
        let mut pending_row = Some(row);

        let (_, table) = self
            .content
            .header
            .find_table(page_type)
            .ok_or_else(|| RekordcrateError::TableTypeNotPresent(page_type))?;
        let old_last_page = table.last_page;
        let empty_candidate = PageIndex::try_from(table.empty_candidate)?;

        // Try the chain tail first.
        if let Some(row_ref) = self.try_insert_row(old_last_page, row_size, &mut pending_row)? {
            return Ok(row_ref);
        }

        // Tail was full or not a data page. Here we check if for some reason empty_candidate
        // is a usable (i.e. has enough space) data page of the right type, and if so we use it.
        if empty_candidate != old_last_page
            && self.load_page(empty_candidate).is_ok_and(|page| {
                page.header.page_type == page_type && matches!(page.content, PageContent::Data(_))
            })
        {
            if let Some(row_ref) =
                self.try_insert_row(empty_candidate, row_size, &mut pending_row)?
            {
                self.relink_chain_end(old_last_page, empty_candidate)?;
                let (_, table) = self
                    .content
                    .header
                    .find_table_mut(page_type)
                    .ok_or_else(|| RekordcrateError::TableTypeNotPresent(page_type))?;
                table.last_page = empty_candidate;
                return Ok(row_ref);
            }
        }

        // No existing page fit: allocate a fresh one and link it onto the tail.
        let new_page_index = self.alloc_data_page(page_type)?;
        self.relink_chain_end(old_last_page, new_page_index)?;

        let (_, table) = self
            .content
            .header
            .find_table_mut(page_type)
            .ok_or_else(|| RekordcrateError::TableTypeNotPresent(page_type))?;
        table.last_page = new_page_index;

        self.try_insert_row(new_page_index, row_size, &mut pending_row)?
            .ok_or(RekordcrateError::IntegrityError(
                "newly allocated page has no room for row",
            ))
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
#[derive(Debug)]
pub struct PageIterator<'db, IO> {
    db_pages: &'db mut [LazyPage],
    db_pages_offset: usize,
    db_io: &'db mut IO,
    db_page_size: u32,
    db_type: DatabaseType,

    next_page: Option<PageIndex>,
    last_page: PageIndex,
}

impl<'db, R: Read + Seek> FallibleIterator for PageIterator<'db, R> {
    type Item = &'db mut Page;
    type Error = RekordcrateError;

    /// Loads the next page in the iterator.
    fn next(&mut self) -> RekordcrateResult<Option<&'db mut Page>> {
        match self.next_page {
            None => Ok(None),
            Some(page_index) => {
                // Throw away references to pages lower than the next page index,
                // leaving our target page at the start of `pages`.
                // ASSUMPTION: pages in a table are linked in increasing order by index.
                let slice_index = (page_index.0 as usize)
                    .checked_sub(self.db_pages_offset)
                    .ok_or(RekordcrateError::PageOrderViolation(page_index))?;
                let db_pages: &'db mut [LazyPage] = std::mem::take(&mut self.db_pages);
                let (_, pages): (_, &'db mut [LazyPage]) = db_pages
                    .split_at_mut_checked(slice_index)
                    .ok_or(RekordcrateError::PageNotPresent(page_index))?;
                // Pull out the target page and leave the rest in `self.db_pages`.
                let (page_entry, pages): (&'db mut LazyPage, &'db mut [LazyPage]) = pages
                    .split_first_mut()
                    .ok_or(RekordcrateError::PageNotPresent(page_index))?;
                self.db_pages = pages;
                self.db_pages_offset = page_index.0 as usize + 1;

                if let LazyPage::NotLoaded = page_entry {
                    let page = read_page(self.db_io, page_index, self.db_page_size, self.db_type)?;
                    *page_entry = LazyPage::Loaded(page);
                }
                let page: &'db mut Page = match page_entry {
                    LazyPage::Loaded(page) => page,
                    _ => unreachable!(),
                };

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::pdb::string::DeviceSQLString;
    use crate::util::MaybeCalculated;
    use std::fs::File;
    use std::io::Cursor;

    fn open_test_db_rw() -> Database<Cursor<Vec<u8>>> {
        let bytes = include_bytes!("../../data/pdb/num_rows/export.pdb");
        Database::open(Cursor::new(bytes.to_vec()), DatabaseType::Plain).unwrap()
    }

    fn get_table_row_count<RowT: RowVariant>(
        db: &mut Database<impl std::io::Read + std::io::Seek>,
    ) -> usize {
        db.iter_rows::<RowT>()
            .expect("Failed to load rows")
            .count()
            .expect("Failed to count rows")
    }

    #[test]
    fn test_pageiterator_safety() {
        // This was written when PageIterator used unsafe.
        // It's a small test and provides value in case we ever want to use unsafe again.
        // Run with `MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test test_pageiterator_safety`.
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let mut iter = db
            .iter_pages(PageType::Plain(PlainPageType::Tracks))
            .unwrap();

        let first = iter.next().unwrap().unwrap();
        let second = iter.next().unwrap().unwrap();

        // Should be disallowed since `db` is still borrowed by `iter` until all pages go out of scope.
        // let _iter2 = db
        //     .iter_pages(PageType::Plain(PlainPageType::Tracks))
        //     .unwrap();

        assert_eq!(
            first.header.page_type,
            PageType::Plain(PlainPageType::Tracks)
        );
        assert_eq!(
            second.header.page_type,
            PageType::Plain(PlainPageType::Tracks)
        );

        // Should be allowed since the `db` borrow can now be released.
        let _iter3 = db
            .iter_pages(PageType::Plain(PlainPageType::Tracks))
            .unwrap();
    }

    #[test]
    fn test_allocate_page_updates_header_and_storage() {
        let mut db = open_test_db_rw();
        let next_unused_before = db.content.header.next_unused_page;
        let page_count_before = db.content.pages.len();

        let new_page_index = db
            .alloc_data_page(PageType::Plain(PlainPageType::Keys))
            .unwrap();

        assert_eq!(new_page_index, next_unused_before);
        assert_eq!(db.content.pages.len(), page_count_before + 1);
        assert_eq!(
            db.content.header.next_unused_page,
            PageIndex::try_from(next_unused_before.0 + 1).unwrap()
        );

        let expected_next_page = Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END;
        let new_page = db.load_page(new_page_index).unwrap();
        assert_eq!(new_page.header.page_index, new_page_index);
        assert_eq!(
            new_page.header.page_type,
            PageType::Plain(PlainPageType::Keys)
        );
        assert_eq!(new_page.header.next_page, expected_next_page);
        assert!(matches!(new_page.content, PageContent::Data(_)));
    }

    #[test]
    fn test_add_row_updates_index_inner_next_page_when_linking_new_page() {
        let mut db = open_test_db_rw();
        let tracks_page_type = PageType::Plain(PlainPageType::Tracks);

        let (_, tracks_table) = db.content.header.find_table(tracks_page_type).unwrap();
        let tracks_first_page = tracks_table.first_page;
        let original_last_page = tracks_table.last_page;

        let row = {
            let page = db.load_page(original_last_page).unwrap();
            let data_content = page.content.as_data().expect("expected data page");
            data_content
                .rows
                .values()
                .next()
                .expect("expected existing row")
                .clone()
        };

        {
            let first_page = db.load_page(tracks_first_page).unwrap();
            assert!(matches!(first_page.content, PageContent::Index(_)));
        }

        {
            let first_page = db.load_page(tracks_first_page).unwrap();
            first_page.header.next_page = Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END;
            match &mut first_page.content {
                PageContent::Index(index_content) => {
                    index_content.header.next_page = Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END;
                }
                _ => panic!("expected index page"),
            }
        }

        let (_, tracks_table_mut) = db.content.header.find_table_mut(tracks_page_type).unwrap();
        tracks_table_mut.last_page = tracks_first_page;

        let row_ref = db.add_row(row).unwrap();
        assert_eq!(row_ref.row_offset, 0);

        let first_page = db.load_page(tracks_first_page).unwrap();
        assert_eq!(first_page.header.next_page, row_ref.page_index);
        match &first_page.content {
            PageContent::Index(index_content) => {
                assert_eq!(index_content.header.next_page, row_ref.page_index);
            }
            _ => panic!("expected index page"),
        }

        let (_, tracks_table_after) = db.content.header.find_table(tracks_page_type).unwrap();
        assert_eq!(tracks_table_after.last_page, row_ref.page_index);
    }

    #[test]
    fn test_add_row_allocates_reachable_page() {
        let mut data = Vec::from(include_bytes!("../../data/pdb/num_rows/export.pdb"));

        // Capture state before modification (read-only snapshot via open_non_persistent).
        let next_unused_before = {
            let db =
                Database::open_non_persistent(Cursor::new(&data[..]), DatabaseType::Plain).unwrap();
            db.get_header().next_unused_page
        };
        let entries_before = get_table_row_count::<HistoryEntry>(
            &mut Database::open_non_persistent(Cursor::new(&data[..]), DatabaseType::Plain)
                .unwrap(),
        );

        let added = {
            // Owned, growable buffer: allocating pages extends the DB past its original length.
            let mut db = Database::open(Cursor::new(data.clone()), DatabaseType::Plain).unwrap();

            // Grab an existing HistoryEntry to clone as a template for the appended rows.
            let template_row = db
                .iter_pages(PageType::Plain(PlainPageType::HistoryEntries))
                .expect("failed to load HistoryEntries pages")
                .find_map(|page| {
                    Ok(page
                        .content
                        .as_data()
                        .and_then(|d| d.rows.values().next().cloned()))
                })
                .expect("no HistoryEntry rows found")
                .expect("expected a HistoryEntry row");

            // The last page has 8 bytes free; a HistoryEntry is larger, so this forces allocation.
            db.add_row(template_row.clone())
                .expect("failed to append HistoryEntry row");
            // A second append surely lands on the freshly allocated page (exercises link-following too).
            db.add_row(template_row.clone())
                .expect("failed to append second HistoryEntry row");

            // Test-private field access: flush then move the cursor out without a public accessor.
            db.flush().expect("failed to flush database");
            data = db.io.into_inner();
            2
        };

        // A new page should have been allocated.
        let next_unused_after = {
            let db =
                Database::open_non_persistent(Cursor::new(&data[..]), DatabaseType::Plain).unwrap();
            db.get_header().next_unused_page
        };
        assert!(
            next_unused_after > next_unused_before,
            "expected next_unused_page to advance after appending ({:?} -> {:?})",
            next_unused_before,
            next_unused_after
        );

        // Re-open read-only and confirm the appended rows are reachable through the chain.
        let mut db = Database::open_non_persistent(Cursor::new(&data[..]), DatabaseType::Plain)
            .expect("failed to reopen database");
        let entries_after = get_table_row_count::<HistoryEntry>(&mut db);
        assert_eq!(
            entries_after,
            entries_before + added,
            "appended rows not reachable on re-read; the new page was allocated but never linked into the chain"
        );
    }

    #[test]
    fn test_create_initializes_index_data_pairs() {
        let table_page_types = [
            PageType::Plain(PlainPageType::Tracks),
            PageType::Plain(PlainPageType::Artists),
        ];
        let db = Database::create(
            Cursor::new(Vec::new()),
            DatabaseType::Plain,
            &table_page_types,
        )
        .unwrap();

        assert_eq!(db.content.header.page_size, 4096);
        assert_eq!(db.content.header.num_tables, 2);
        assert_eq!(
            db.content.header.next_unused_page,
            PageIndex::try_from(5).unwrap()
        );
        assert_eq!(db.content.pages.len(), 4);

        let tracks_table = &db.content.header.tables[0];
        assert_eq!(tracks_table.first_page, PageIndex::try_from(1).unwrap());
        assert_eq!(tracks_table.last_page, PageIndex::try_from(1).unwrap());

        let artists_table = &db.content.header.tables[1];
        assert_eq!(artists_table.first_page, PageIndex::try_from(3).unwrap());
        assert_eq!(artists_table.last_page, PageIndex::try_from(3).unwrap());

        let tracks_index = match &db.content.pages[0] {
            LazyPage::Loaded(page) => page,
            LazyPage::NotLoaded => panic!("expected loaded page"),
        };
        assert!(matches!(tracks_index.content, PageContent::Index(_)));
        assert_eq!(
            tracks_index.header.next_page,
            Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END
        );
        match &tracks_index.content {
            PageContent::Index(index_content) => {
                assert_eq!(
                    index_content.header.next_page,
                    Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END
                );
            }
            _ => panic!("expected index page"),
        }

        let tracks_data = match &db.content.pages[1] {
            LazyPage::Loaded(page) => page,
            LazyPage::NotLoaded => panic!("expected loaded page"),
        };
        assert!(matches!(tracks_data.content, PageContent::Data(_)));
        assert_eq!(
            tracks_data.header.next_page,
            Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END
        );
    }

    #[test]
    fn test_create_uses_chain_end_sentinel_for_all_new_data_pages() {
        let table_page_types = [
            PageType::Plain(PlainPageType::Tracks),
            PageType::Plain(PlainPageType::Genres),
            PageType::Plain(PlainPageType::Artists),
        ];
        let db = Database::create(
            Cursor::new(Vec::new()),
            DatabaseType::Plain,
            &table_page_types,
        )
        .unwrap();

        for page_entry in &db.content.pages {
            let page = match page_entry {
                LazyPage::Loaded(page) => page,
                LazyPage::NotLoaded => panic!("expected loaded page"),
            };

            if matches!(page.content, PageContent::Data(_)) {
                assert_eq!(
                    page.header.next_page,
                    Database::<Cursor<Vec<u8>>>::PAGE_CHAIN_END
                );
            }
        }
    }

    #[test]
    fn test_add_row_rejects_undersized_track_row() {
        let table_page_types = [PageType::Plain(PlainPageType::Tracks)];
        let mut db = Database::create(
            Cursor::new(Vec::new()),
            DatabaseType::Plain,
            &table_page_types,
        )
        .unwrap();

        let track = Track {
            subtype: Subtype(0x24),
            index_shift: 0x0000,
            bitmask: 0,
            sample_rate: 0,
            composer_id: ArtistId(0),
            file_size: 0,
            unknown2: 0,
            unknown3: 0,
            unknown4: 0,
            unknown5: 0,
            artwork_id: ArtworkId(0),
            key_id: KeyId(0),
            bitrate: 0,
            color: ColorIndex::None,
            orig_artist_id: ArtistId(0),
            disc_number: 0,
            duration: 0,
            file_type: FileType::Unknown,
            label_id: LabelId(0),
            genre_id: GenreId(0),
            play_count: 0,
            rating: 0,
            remixer_id: ArtistId(0),
            track_number: 0,
            tempo: 0,
            year: 0,
            sample_depth: 0,
            id: TrackId(1),
            artist_id: ArtistId(1),
            album_id: AlbumId(1),
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TrackStrings {
                    title: DeviceSQLString::new("Music").unwrap(),
                    filename: DeviceSQLString::new("02 - Music.mp3").unwrap(),
                    file_path: DeviceSQLString::new("/Contents/02 - Music.mp3").unwrap(),
                    isrc: DeviceSQLString::empty(),
                    lyricist: DeviceSQLString::empty(),
                    unknown_string2: DeviceSQLString::empty(),
                    unknown_string3: DeviceSQLString::empty(),
                    unknown_string4: DeviceSQLString::empty(),
                    unknown_string5: DeviceSQLString::empty(),
                    unknown_string6: DeviceSQLString::empty(),
                    unknown_string7: DeviceSQLString::empty(),
                    unknown_string8: DeviceSQLString::empty(),
                    message: DeviceSQLString::empty(),
                    publish_track_information: DeviceSQLString::empty(),
                    autoload_hotcues: DeviceSQLString::empty(),
                    date_added: DeviceSQLString::empty(),
                    release_date: DeviceSQLString::empty(),
                    mix_name: DeviceSQLString::empty(),
                    analyze_date: DeviceSQLString::empty(),
                    analyze_path: DeviceSQLString::empty(),
                    comment: DeviceSQLString::empty(),
                },
            },
        };

        let err = db
            .add_row(Row::Plain(PlainRow::Track(track)))
            .expect_err("expected undersized track row to be rejected");

        match err {
            RekordcrateError::TrackRowTooSmall {
                track_id,
                allocated,
                minimum,
            } => {
                assert_eq!(track_id, 1);
                assert!(allocated < minimum);
                assert_eq!(minimum, 221);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
