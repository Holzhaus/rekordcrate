// Copyright (c) 2026
// SPDX-License-Identifier: MPL-2.0

#![warn(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]

pub mod anlz;
pub mod pdb;
pub mod setting;
pub mod util;

#[cfg(feature = "xml")]
pub mod xml;

pub(crate) mod xor;

pub use crate::util::RekordcrateError as Error;
pub use crate::util::RekordcrateResult as Result;
