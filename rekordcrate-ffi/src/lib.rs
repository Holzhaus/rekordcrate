// Copyright (c) 2022 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Bindings for using rekordcrate from C code.

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use rekordcrate::pdb::Track;
use rekordcrate::DeviceExport;
use std::ffi::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use std::path::Path;

#[cfg(target_family = "unix")]
fn cstr_to_path(cstr: &CStr) -> &Path {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    OsStr::from_bytes(cstr.to_bytes()).as_ref()
}

#[cfg(target_family = "windows")]
fn cstr_to_path(cstr: &CStr) -> &Path {
    ::std::str::from_utf8(slice.to_bytes())
        .expect("keep your surrogates paired")
        .as_ref()
}

/// Open a Rekordbox device export at the given path.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_new(path: *const c_char) -> *mut DeviceExport {
    let cstr = unsafe {
        assert!(!path.is_null());
        CStr::from_ptr(path)
    };

    let path = cstr_to_path(cstr).to_path_buf();
    Box::into_raw(Box::new(DeviceExport::new(path)))
}

/// Free the device export object.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_free(ptr: *mut DeviceExport) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(&mut Box::from_raw(ptr));
    }
}

/// Load the PDB file of the device export.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_load_pdb(ptr: *mut DeviceExport) {
    let device_export = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    device_export.load_pdb().expect("failed to load pdb");
}

/// Iterator over `Track` object.
#[allow(missing_debug_implementations)]
pub struct TrackIterator(Box<dyn Iterator<Item = rekordcrate::Result<Track>>>);

impl Iterator for TrackIterator {
    type Item = rekordcrate::Result<Track>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Get an iterator over all tracks in the collection.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_tracks(ptr: *const DeviceExport) -> *mut TrackIterator {
    let device_export = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };
    Box::into_raw(Box::new(TrackIterator(Box::new(
        device_export.get_tracks().unwrap(),
    ))))
}

/// Get the next track from a tracks iterator.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_tracks_next(ptr: *mut TrackIterator) -> *const Track {
    let track_iterator = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    match track_iterator.next() {
        Some(track) => Box::into_raw(Box::new(track.unwrap())),
        None => std::ptr::null_mut(),
    }
}

/// Free a track iterator.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_tracks_free(ptr: *mut TrackIterator) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(&mut Box::from_raw(ptr));
    }
}

/// Free a track.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_track_free(ptr: *mut Track) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(&mut Box::from_raw(ptr));
    }
}

/// Get the track file path.
#[no_mangle]
pub extern "C" fn rekordcrate_deviceexport_track_get_file_path(ptr: *mut Track) -> *const c_char {
    let track = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    match track
        .file_path
        .clone()
        .into_string()
        .ok()
        .map(CString::new)
        .and_then(|s| s.ok())
    {
        Some(str) => str.as_ptr(),
        None => std::ptr::null(),
    }
}
