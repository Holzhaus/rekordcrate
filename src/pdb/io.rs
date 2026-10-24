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

/// Records in a row the place it takes in its page, which some of its fields refer to.
fn index_shift_for(mut row: Row, index_in_page: u16) -> RekordcrateResult<Row> {
    let index_shift =
        index_in_page
            .checked_mul(INDEX_SHIFT_STEP)
            .ok_or(RekordcrateError::IntegrityError(
                "page holds too many rows to give one an index shift",
            ))?;
    row.set_index_shift(index_shift);
    Ok(row)
}

/// A PDB database opened for reading or writing.
#[derive(Debug)]
pub struct Database<IO> {
    io: IO,
    db_type: DatabaseType,
    content: LazyDatabase,
}

impl<IO> Database<IO> {
    /// Consumes the database and returns the underlying IO.
    ///
    /// Note that any unflushed changes are discarded.
    pub fn into_inner(self) -> IO {
        self.io
    }
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

    /// Adds a row to its table, allocating a new page if the table has no room for it.
    ///
    /// The row is appended to the last page of the table if it fits there, otherwise a new page
    /// is appended to the table via [`Database::append_page`] and the row is stored in that. Free
    /// space in earlier pages of the table is not reused.
    ///
    /// Returns the index of the page that the row was stored in.
    pub fn add_row<RowT: RowVariant>(&mut self, row: RowT) -> RekordcrateResult<PageIndex> {
        self.add_row_with_slack(row, 0)
    }

    /// Adds a row to its table, leaving `slack_bytes` of unused page heap after it.
    ///
    /// Rekordbox leaves spare space after the rows of some tables, which the row itself does not
    /// use. The rule behind it is not known, but the amount can be read off the exports it
    /// writes: eight bytes for the rows of the history and tag tables, and more for track rows.
    /// Players have been reported to depend on rows being given a certain amount of space, so
    /// this is not only a matter of writing the same bytes as Rekordbox.
    ///
    /// Returns the index of the page that the row was stored in.
    pub fn add_row_with_slack<RowT: RowVariant>(
        &mut self,
        row: RowT,
        slack_bytes: u16,
    ) -> RekordcrateResult<PageIndex> {
        let page_type = RowT::PAGE_TYPE;
        let row = row.into_row();
        let bytes_required = row.heap_bytes_required(()).saturating_add(slack_bytes);

        // Checked up front so that a row that can never be stored does not leave an empty page
        // behind in the table.
        let required = Page::heap_bytes_for_row(bytes_required);
        let available = Page::empty_page_heap_size(self.content.header.page_size);
        if required > available {
            return Err(RekordcrateError::RowTooLarge {
                required,
                available,
            });
        }

        let (_, table) = self
            .content
            .header
            .find_table(page_type)
            .ok_or(RekordcrateError::TableTypeNotPresent(page_type))?;
        let last_page = table.last_page;

        // The last page of a table is an index page while the table is still empty, in which case
        // the allocation fails here and we fall through to appending a data page below.
        let page = self.load_page(last_page)?;
        if let Some((index_in_page, insert)) = page.allocate_row(bytes_required) {
            insert(index_shift_for(row, index_in_page)?);
            return Ok(last_page);
        }

        let page_index = self.append_page(page_type)?;
        let page = self.load_page(page_index)?;
        let (index_in_page, insert) = page.allocate_row(bytes_required).ok_or(
            // Unreachable: we checked above that the row fits in an empty page.
            RekordcrateError::IntegrityError("freshly appended page cannot hold the row"),
        )?;
        insert(index_shift_for(row, index_in_page)?);
        Ok(page_index)
    }

    /// Whether the page at `page_index` has never been written.
    ///
    /// Rekordbox reserves pages ahead of time by pointing `Table::empty_candidate` at them, and
    /// such a page is either zeroed or past the end of the database. A page that is in use always
    /// stores its own non-zero page index, so it can be told apart from a reserved one.
    fn is_page_unused(&mut self, page_index: PageIndex) -> RekordcrateResult<bool> {
        let Some(entry) = usize::try_from(page_index.0)
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| self.content.pages.get(index))
        else {
            return Ok(false);
        };
        // A page that we have already parsed belongs to a table.
        if let LazyPage::Loaded(_) = entry {
            return Ok(false);
        }

        let offset = page_index.offset(self.content.header.page_size);
        self.io.seek(SeekFrom::Start(offset))?;
        let mut header = [0u8; 8];
        let mut bytes_read = 0;
        while bytes_read < header.len() {
            match self.io.read(&mut header[bytes_read..])? {
                // Past the end of the database, so the page cannot be in use.
                0 => return Ok(true),
                n => bytes_read += n,
            }
        }
        // The page magic and the page index of a page in use are never both zero.
        Ok(header == [0u8; 8])
    }

    /// Appends a new, empty data page to the end of a table's page chain.
    ///
    /// The page reserved for the table by `Table::empty_candidate` is used if it is still unused,
    /// which is how Rekordbox claims the page that follows a table's index page. A fresh page is
    /// then reserved for the next time the table grows. Otherwise the page is taken from the
    /// unused space past the end of the database, which grows the database by one page once the
    /// changes are flushed.
    ///
    /// The previous last page of the table is linked to the new page, the table's `last_page` is
    /// updated to point at it, and the database sequence number is incremented.
    ///
    /// Returns the index of the newly appended page.
    pub fn append_page(&mut self, page_type: PageType) -> RekordcrateResult<PageIndex> {
        let (table_index, table) = self
            .content
            .header
            .find_table(page_type)
            .ok_or(RekordcrateError::TableTypeNotPresent(page_type))?;
        let previous_last_page = table.last_page;
        let empty_candidate = table.empty_candidate;

        // `next_unused_page` points past the end of the database. Page indices are 1-based, hence
        // the `- 1`.
        let next_unused_page = self.content.header.next_unused_page;
        if usize::try_from(next_unused_page.0)
            .ok()
            .and_then(|i| i.checked_sub(1))
            != Some(self.content.pages.len())
        {
            return Err(RekordcrateError::IntegrityError(
                "next_unused_page does not point past the last page of the database",
            ));
        }
        // A page is always reserved for the next time the table grows, so the database gains a
        // page either way.
        let new_next_unused_page = PageIndex::try_from(next_unused_page.0 + 1)?;

        let reserved_page = match PageIndex::try_from(empty_candidate) {
            Ok(candidate) if self.is_page_unused(candidate)? => Some(candidate),
            _ => None,
        };
        // Claiming the reserved page of the table means reserving a fresh one in its place. The
        // last page of a table points at the page that the table will use next, which is that
        // reserved page. Where there is nothing to claim, `empty_candidate` is left alone, as it
        // still describes where the table has free space.
        let (page_index, next_page, new_empty_candidate) = match reserved_page {
            Some(candidate) => (candidate, next_unused_page, Some(next_unused_page)),
            None => (next_unused_page, new_next_unused_page, None),
        };

        let page_size = self.content.header.page_size;
        let sequence = self.content.header.sequence;
        let page = Page::new_empty_data(page_index, page_type, next_page, page_size, sequence);

        // Link the previous last page of the table to the new page. This has to happen before we
        // commit to the new state, so that a failure to load that page leaves the database
        // unchanged.
        let previous_page = self.load_page(previous_last_page)?;
        previous_page.header.next_page = page_index;
        if let Some(index_content) = previous_page.content.as_index_mut() {
            // The page header of an index page points at the page that physically follows it,
            // while the index content points at the next page of the table.
            index_content.header.next_page = page_index;
        }

        match reserved_page {
            // The reserved page already has an entry, and the page reserved in its place needs a
            // new one.
            Some(_) => {
                let entry = usize::try_from(page_index.0)
                    .ok()
                    .and_then(|index| index.checked_sub(1))
                    .and_then(|index| self.content.pages.get_mut(index))
                    .ok_or(RekordcrateError::IntegrityError(
                        "the page reserved for the table is not one the database holds",
                    ))?;
                *entry = LazyPage::Loaded(page);
                self.content.pages.push(LazyPage::NotLoaded);
            }
            None => self.content.pages.push(LazyPage::Loaded(page)),
        }

        let header = &mut self.content.header;
        header.next_unused_page = new_next_unused_page;
        header.sequence += 1;
        let table = header
            .tables
            .get_mut(table_index.0)
            .ok_or(RekordcrateError::TableNotPresent(table_index))?;
        table.last_page = page_index;
        if let Some(new_empty_candidate) = new_empty_candidate {
            table.empty_candidate = new_empty_candidate.0;
        }

        Ok(page_index)
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
    /// Page size that Rekordbox uses for the databases it writes.
    pub const DEFAULT_PAGE_SIZE: u32 = 4096;

    /// Creates a new, empty database with the default page size.
    ///
    /// See [`Database::create_with_page_size`] for details.
    pub fn create(io: RW, db_type: DatabaseType) -> RekordcrateResult<Self> {
        Self::create_with_page_size(io, db_type, Self::DEFAULT_PAGE_SIZE)
    }

    /// Creates a new, empty database, overwriting whatever the IO contained before.
    ///
    /// The database is laid out the way Rekordbox lays out an export it has not put any content
    /// into yet: a header page, followed by one index page per table at every odd page index,
    /// with the even page after each of those reserved for the first data page of that table.
    /// Every table is empty, so [`Database::add_row`] claims those reserved pages as rows are
    /// added. The page reserved by the last table is past the end of the database until it is
    /// claimed, which is how Rekordbox leaves pages it has reserved but not used yet.
    ///
    /// Note that this is only the database structure. Players expect some tables to be populated
    /// with default content, which this does not add.
    pub fn create_with_page_size(
        io: RW,
        db_type: DatabaseType,
        page_size: u32,
    ) -> RekordcrateResult<Self> {
        let page_types = db_type.table_page_types();
        let mut tables = Vec::with_capacity(page_types.len());
        let mut pages = Vec::with_capacity(page_types.len() * 2);

        for (table_index, &page_type) in page_types.iter().enumerate() {
            let index_page = PageIndex::try_from(
                u32::try_from(table_index)
                    .map_err(|_| RekordcrateError::IntegrityError("too many tables"))?
                    * 2
                    + 1,
            )?;
            let reserved_page = PageIndex::try_from(index_page.0 + 1)?;

            tables.push(Table {
                page_type,
                empty_candidate: reserved_page.0,
                first_page: index_page,
                last_page: index_page,
            });
            pages.push(LazyPage::Loaded(Page::new_empty_index(
                index_page,
                page_type,
                reserved_page,
            )));
            // The reserved page is left as zeros until a table claims it.
            pages.push(LazyPage::NotLoaded);
        }

        let num_tables = u32::try_from(tables.len())
            .map_err(|_| RekordcrateError::IntegrityError("too many tables"))?;
        let next_unused_page = PageIndex::try_from(
            u32::try_from(pages.len())
                .map_err(|_| RekordcrateError::IntegrityError("too many pages"))?
                + 1,
        )?;

        let mut db = Self {
            io,
            db_type,
            content: LazyDatabase {
                header: Header {
                    page_size,
                    num_tables,
                    next_unused_page,
                    // Matches the databases written by Rekordbox. Purpose unknown.
                    unknown: 5,
                    sequence: 1,
                    tables,
                },
                pages,
            },
        };

        db.flush()?;

        Ok(db)
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
        let endian = Endian::Little;
        self.io.seek(SeekFrom::Start(0))?;
        self.content.write_options(&mut self.io, endian, ())?;
        self.pad_to_page_boundary()?;
        Ok(())
    }

    /// Pads the database with zeros so that it contains a whole number of pages.
    ///
    /// The unused tail of a page heap is never written out, it is skipped over with a seek. That
    /// is fine for a page that is already backed by bytes in the database, but a page appended
    /// past the previous end of the database would be left truncated, because seeking past the
    /// end of a file does not extend it.
    fn pad_to_page_boundary(&mut self) -> RekordcrateResult<()> {
        let page_size = u64::from(self.content.header.page_size);
        let size = self.io.seek(SeekFrom::End(0))?;
        let padding = (page_size - size % page_size) % page_size;
        if padding > 0 {
            let padding = usize::try_from(padding).map_err(|_| {
                RekordcrateError::IntegrityError("a page of padding does not fit in memory")
            })?;
            self.io.write_all(&vec![0u8; padding])?;
        }
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
    use crate::util::MaybeCalculated;
    use std::fs::File;
    use std::io::Cursor;

    /// Appends a page to the keys table, fills it with rows, and checks that the rows survive a
    /// round-trip through serialization.
    #[test]
    fn test_append_page_with_rows() {
        let data = Vec::from(include_bytes!("../../data/pdb/num_rows/export.pdb"));
        let mut db = Database::open(Cursor::new(data), DatabaseType::Plain).unwrap();

        let page_type = PageType::Plain(PlainPageType::Keys);
        let page_index = db.append_page(page_type).unwrap();
        let page = db.load_page(page_index).unwrap();

        let new_keys = [(1000u32, "Amin"), (1001, "Bmin")];
        for (id, name) in new_keys {
            let row = Row::Plain(PlainRow::Key(Key {
                id: KeyId(id),
                id2: id,
                name: name.parse().unwrap(),
            }));
            let (_, insert) = page.allocate_row(row.heap_bytes_required(())).unwrap();
            insert(row);
        }
        assert_eq!(
            page.header.packed_row_counts.num_rows(),
            new_keys.len() as u16
        );

        db.flush().unwrap();
        let data = db.into_inner().into_inner();

        let mut db =
            Database::open_non_persistent(Cursor::new(data.as_slice()), DatabaseType::Plain)
                .unwrap();
        let names: Vec<String> = db
            .iter_rows::<Key>()
            .unwrap()
            .map(|key| Ok(key.name.clone().into_string().unwrap()))
            .collect()
            .unwrap();
        assert_eq!(names.len(), 67 + new_keys.len());
        for (_, name) in new_keys {
            assert!(names.contains(&name.to_string()), "{names:?}");
        }
    }

    fn artist_row(id: u32) -> Artist {
        Artist {
            subtype: Subtype(0x60),
            // Overwritten when the row is stored, from the place it takes in its page.
            index_shift: 0,
            id: ArtistId(id),
            offsets: OffsetArrayContainer {
                offsets: MaybeCalculated::Calculated,
                inner: TrailingName {
                    name: format!("Artist {id}").parse().unwrap(),
                },
            },
        }
    }

    /// The `index_shift` of a row refers to the place the row takes in its own page, not in its
    /// table: in the exports Rekordbox wrote it is `INDEX_SHIFT_STEP` times that place and it
    /// restarts from zero in every page of a table.
    #[test]
    fn test_index_shift_counts_within_the_page() {
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let mut pages = db
            .iter_pages(PageType::Plain(PlainPageType::Artists))
            .unwrap();

        let mut data_pages = 0;
        while let Some(page) = pages.next().unwrap() {
            let Some(content) = page.content.as_data() else {
                continue;
            };
            let shifts: Vec<u16> = content
                .rows
                .values()
                .filter_map(|row| match row {
                    Row::Plain(PlainRow::Artist(artist)) => Some(artist.index_shift),
                    _ => None,
                })
                .collect();
            if shifts.is_empty() {
                continue;
            }
            data_pages += 1;

            // Every page starts over at zero, which is the whole point.
            assert_eq!(
                Some(0),
                shifts.iter().copied().min(),
                "page {:?} does not start at zero: {shifts:?}",
                page.header.page_index
            );
            for shift in &shifts {
                assert_eq!(
                    0,
                    shift % INDEX_SHIFT_STEP,
                    "page {:?} holds {shift}, which is not a multiple of the step",
                    page.header.page_index
                );
            }
            // A page records a place for every row ever allocated in it, so the largest place a
            // row refers to is the last of those, even where a row has since been superseded.
            let allocated = page.header.packed_row_counts.num_rows();
            let largest = shifts.iter().copied().max().unwrap();
            assert!(
                largest <= (allocated - 1) * INDEX_SHIFT_STEP,
                "page {:?} holds {largest} for {allocated} allocated rows",
                page.header.page_index
            );
        }
        assert!(data_pages > 1, "fixture holds only {data_pages} data pages");
    }

    /// A table takes more rows than `INDEX_SHIFT_STEP` times a table-wide row index would fit in
    /// the `u16` the field is stored in, because the place it refers to is within the page.
    #[test]
    fn test_add_row_past_a_table_wide_index_shift() {
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();

        // One more than the number of rows that a table-wide `index_shift` could number.
        let rows = u32::from(u16::MAX / INDEX_SHIFT_STEP) + 2;
        for id in 0..rows {
            db.add_row(artist_row(10_000 + id))
                .unwrap_or_else(|error| panic!("row {id} of {rows} was refused: {error}"));
        }

        let mut pages = db
            .iter_pages(PageType::Plain(PlainPageType::Artists))
            .unwrap();
        while let Some(page) = pages.next().unwrap() {
            let Some(content) = page.content.as_data() else {
                continue;
            };
            // A page the fixture already held can carry a superseded row, whose place is still
            // taken, so the places are only known to grow by whole steps rather than by one.
            let mut previous = None;
            for row in content.rows.values() {
                let Row::Plain(PlainRow::Artist(artist)) = row else {
                    continue;
                };
                let shift = artist.index_shift;
                assert_eq!(
                    0,
                    shift % INDEX_SHIFT_STEP,
                    "page {:?} holds {shift}, which is not a multiple of the step",
                    page.header.page_index
                );
                if let Some(previous) = previous {
                    assert!(
                        shift > previous,
                        "page {:?} holds {shift} after {previous}",
                        page.header.page_index
                    );
                }
                previous = Some(shift);
            }
        }
    }

    fn key_row(id: u32, name: &str) -> Key {
        Key {
            id: KeyId(id),
            id2: id,
            name: name.parse().unwrap(),
        }
    }

    fn key_names(db: &mut Database<impl Read + Seek>) -> Vec<String> {
        db.iter_rows::<Key>()
            .unwrap()
            .map(|key| Ok(key.name.clone().into_string().unwrap()))
            .collect()
            .unwrap()
    }

    /// A row that fits in the last page of its table is stored there, without growing the table.
    #[test]
    fn test_add_row_reuses_last_page() {
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let next_unused_page = db.get_header().next_unused_page;
        let (_, table) = db
            .get_header()
            .find_table(PageType::Plain(PlainPageType::Keys))
            .unwrap();
        let last_page = table.last_page;

        let page_index = db.add_row(key_row(1000, "Amin")).unwrap();

        assert_eq!(page_index, last_page);
        assert_eq!(db.get_header().next_unused_page, next_unused_page);
        assert_eq!(key_names(&mut db).len(), 68);
    }

    /// Adding a row to an empty table has to allocate a data page, because the only page of an
    /// empty table is an index page that cannot hold rows.
    #[test]
    fn test_add_row_to_empty_table() {
        let file = File::open("data/complete_export/empty/PIONEER/rekordbox/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let (_, table) = db
            .get_header()
            .find_table(PageType::Plain(PlainPageType::Keys))
            .unwrap();
        let (first_page, last_page) = (table.first_page, table.last_page);
        assert_eq!(key_names(&mut db).len(), 0, "fixture should have no keys");

        let page_index = db.add_row(key_row(1, "Amin")).unwrap();

        assert_ne!(page_index, first_page);
        assert_ne!(page_index, last_page);
        assert_eq!(key_names(&mut db), vec!["Amin".to_string()]);
    }

    /// Adding more rows than fit in a single page grows the table across pages, and all of the
    /// rows survive a round-trip through serialization.
    #[test]
    fn test_add_row_spills_into_new_pages() {
        let data = Vec::from(include_bytes!(
            "../../data/complete_export/empty/PIONEER/rekordbox/export.pdb"
        ));
        let mut db = Database::open(Cursor::new(data), DatabaseType::Plain).unwrap();

        const NUM_KEYS: u32 = 500;
        let mut pages = Vec::new();
        for id in 0..NUM_KEYS {
            pages.push(db.add_row(key_row(id, &format!("key{id}"))).unwrap());
        }
        pages.dedup();
        assert!(
            pages.len() > 1,
            "{NUM_KEYS} keys should not fit in a single page, used pages: {pages:?}"
        );

        db.flush().unwrap();
        let data = db.into_inner().into_inner();

        // Set REKORDCRATE_TEST_DUMP_PATH to dump the grown database there for inspection.
        if let Ok(save_dir) = std::env::var("REKORDCRATE_TEST_DUMP_PATH") {
            let save_dir = std::path::Path::new(&save_dir).join("add_row_spills_into_new_pages");
            std::fs::create_dir_all(&save_dir).unwrap();
            std::fs::write(save_dir.join("export.pdb"), &data).unwrap();
        }

        let mut db =
            Database::open_non_persistent(Cursor::new(data.as_slice()), DatabaseType::Plain)
                .unwrap();

        let names = key_names(&mut db);
        assert_eq!(names.len(), NUM_KEYS as usize);
        for id in 0..NUM_KEYS {
            assert!(names.contains(&format!("key{id}")), "key{id} is missing");
        }
    }

    /// A row too large for an empty page cannot be stored anywhere.
    #[test]
    fn test_add_row_too_large() {
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let page_size = db.get_header().page_size;

        let header_before = db.get_header().clone();

        let name = "a".repeat(page_size as usize);
        let result = db.add_row(key_row(1000, &name));

        assert!(
            matches!(result, Err(RekordcrateError::RowTooLarge { .. })),
            "unexpected result: {result:?}"
        );
        // The failed insert must not have left an empty page behind in the table.
        assert_eq!(db.get_header(), &header_before);
    }

    const EMPTY_EXPORT: &[u8] =
        include_bytes!("../../data/complete_export/empty/PIONEER/rekordbox/export.pdb");

    fn create_in_memory(db_type: DatabaseType) -> Database<Cursor<Vec<u8>>> {
        Database::create(Cursor::new(Vec::new()), db_type).unwrap()
    }

    /// A created database has a table per page type, each of them empty.
    #[test]
    fn test_create_tables() {
        for (db_type, num_tables) in [(DatabaseType::Plain, 20), (DatabaseType::Ext, 9)] {
            let mut db = create_in_memory(db_type);
            db.flush().unwrap();
            let data = db.into_inner().into_inner();
            assert_eq!(
                data.len() % Database::<Cursor<Vec<u8>>>::DEFAULT_PAGE_SIZE as usize,
                0
            );

            // Reopen it to check that what we wrote can be parsed back.
            let mut db =
                Database::open_non_persistent(Cursor::new(data.as_slice()), db_type).unwrap();
            let header = db.get_header();
            assert_eq!(header.num_tables as usize, num_tables);
            assert_eq!(header.tables.len(), num_tables);
            assert_eq!(header.page_size, 4096);
            assert_eq!(
                header.next_unused_page,
                PageIndex::try_from(num_tables as u32 * 2 + 1).unwrap()
            );

            for (table_index, table) in header.tables.clone().iter().enumerate() {
                assert_eq!(
                    table.page_type,
                    db_type.table_page_types()[table_index],
                    "unexpected page type for table {table_index}"
                );
                // An empty table consists of its index page alone.
                assert_eq!(table.first_page, table.last_page);
                let page = db.load_page(table.first_page).unwrap();
                assert!(page.header.page_flags.is_index_page());
                assert_eq!(page.content.as_index().unwrap().entries, Vec::new());
            }
        }
    }

    /// A created database reserves a page for every table to grow into, the way Rekordbox does.
    ///
    /// The bytes of the pages themselves are compared against the databases that Rekordbox
    /// writes in `tests/test_pdb_create.rs`.
    #[test]
    fn test_create_reserves_pages_like_rekordbox() {
        let db = create_in_memory(DatabaseType::Plain);
        let reference =
            Database::open_non_persistent(Cursor::new(EMPTY_EXPORT), DatabaseType::Plain).unwrap();

        for (table, reference_table) in db
            .get_header()
            .tables
            .iter()
            .zip(&reference.get_header().tables)
        {
            assert_eq!(table.page_type, reference_table.page_type);
            // A table that has claimed its reserved page has a different one reserved in its
            // place, so only the tables Rekordbox left empty are comparable.
            if reference_table.first_page == reference_table.last_page {
                assert_eq!(
                    table.empty_candidate, reference_table.empty_candidate,
                    "unexpected reserved page for table {:?}",
                    table.page_type
                );
            }
        }
    }

    /// Rows can be added to a created database and survive a round-trip.
    #[test]
    fn test_create_then_add_rows() {
        let mut db = create_in_memory(DatabaseType::Plain);

        let page_index = db.add_row(key_row(1, "Amin")).unwrap();
        db.add_row(key_row(2, "Bmin")).unwrap();

        // The first data page of a table is the one reserved right after its index page.
        let (_, table) = db
            .get_header()
            .find_table(PageType::Plain(PlainPageType::Keys))
            .unwrap();
        assert_eq!(
            page_index,
            PageIndex::try_from(table.first_page.0 + 1).unwrap()
        );

        db.flush().unwrap();
        let data = db.into_inner().into_inner();

        // Set REKORDCRATE_TEST_DUMP_PATH to dump the created database there for inspection.
        if let Ok(save_dir) = std::env::var("REKORDCRATE_TEST_DUMP_PATH") {
            let save_dir = std::path::Path::new(&save_dir).join("create_then_add_rows");
            std::fs::create_dir_all(&save_dir).unwrap();
            std::fs::write(save_dir.join("export.pdb"), &data).unwrap();
        }

        let mut db =
            Database::open_non_persistent(Cursor::new(data.as_slice()), DatabaseType::Plain)
                .unwrap();
        assert_eq!(
            key_names(&mut db),
            vec!["Amin".to_string(), "Bmin".to_string()]
        );
    }

    /// A table claims the page Rekordbox reserved for it before growing the database, and the
    /// page it reserves in its place must not be handed out twice.
    #[test]
    fn test_append_page_claims_reserved_page() {
        let mut db = create_in_memory(DatabaseType::Plain);
        let page_type = PageType::Plain(PlainPageType::Keys);
        let (_, table) = db.get_header().find_table(page_type).unwrap();
        let (index_page, reserved_page) = (table.first_page, table.empty_candidate);
        let next_unused_page = db.get_header().next_unused_page;

        let first = db.append_page(page_type).unwrap();
        let second = db.append_page(page_type).unwrap();

        // The reserved page is claimed first, and is the page right after the index page.
        assert_eq!(first, PageIndex::try_from(reserved_page).unwrap());
        assert_eq!(first, PageIndex::try_from(index_page.0 + 1).unwrap());
        // Claiming it reserved a fresh page, which the second append then claims in turn.
        assert_eq!(second, next_unused_page);
        assert_ne!(first, second);

        let (_, table) = db.get_header().find_table(page_type).unwrap();
        assert_eq!(table.last_page, second);
        // Another page stands reserved for the next time the table grows.
        assert_ne!(table.empty_candidate, second.0);
        assert_eq!(db.iter_pages(page_type).unwrap().count().unwrap(), 3);
    }

    /// A page that is in use must never be claimed as a reserved page, even when a table points
    /// `empty_candidate` at it.
    #[test]
    fn test_append_page_does_not_claim_used_page() {
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let page_type = PageType::Plain(PlainPageType::Keys);
        let next_unused_page = db.get_header().next_unused_page;

        let pages_before: Vec<PageIndex> = db
            .iter_pages(page_type)
            .unwrap()
            .map(|page| Ok(page.header.page_index))
            .collect()
            .unwrap();
        // Point the table at a page of its own that holds rows. Databases written by Rekordbox
        // always reserve an unused page instead, but a page in use must not be handed out even if
        // one of them ever did this.
        let used_page = *pages_before.last().unwrap();
        let (_, table) = db.get_header_mut().find_table_mut(page_type).unwrap();
        table.empty_candidate = used_page.0;

        let page_index = db.append_page(page_type).unwrap();

        assert_eq!(page_index, next_unused_page);
        let (_, table) = db.get_header().find_table(page_type).unwrap();
        assert_eq!(
            table.empty_candidate, used_page.0,
            "empty_candidate of a table we did not claim from must not be moved"
        );
        // The rows of the page that was pointed at are still readable.
        assert_eq!(key_names(&mut db).len(), 67);
    }

    /// The whole page heap of a newly appended page must be accounted for as free space.
    #[test]
    fn test_appended_page_free_size() {
        let file = File::open("data/pdb/num_rows/export.pdb").unwrap();
        let mut db = Database::open_non_persistent(file, DatabaseType::Plain).unwrap();
        let page_size = db.get_header().page_size;

        let page_index = db
            .append_page(PageType::Plain(PlainPageType::Keys))
            .unwrap();
        let page = db.load_page(page_index).unwrap();

        assert_eq!(page.header.used_size, 0);
        assert_eq!(
            u32::from(page.header.free_size),
            page_size - PageHeader::BINARY_SIZE - DataPageHeader::BINARY_SIZE
        );
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
}
