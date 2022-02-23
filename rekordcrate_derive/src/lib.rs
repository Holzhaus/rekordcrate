// Copyright (c) 2022 Jan Holthuis
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! This library provides access to device libraries exported from Pioneer's Rekordbox DJ software.

#![warn(unsafe_code)]
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

use proc_macro::TokenStream;
use quote::quote;

/// Add parse method to type.
#[proc_macro_derive(Parse)]
pub fn derive_parse_fn(tokens: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();
    let name = ast.ident;

    if let syn::Data::Enum(enum_data) = ast.data {
        let match_variants = enum_data
            .variants
            .into_iter()
            .enumerate()
            .filter(|(_, variant)| matches!(variant.fields, syn::Fields::Unit))
            .map(|(i, variant)| (u8::try_from(i + 0x80).unwrap(), variant.ident))
            .map(|(i, variant)| {
                quote! {
                        #i => Self::#variant,
                }
            });

        let result = quote! {
            impl #name {
                fn parse(input: &[u8]) -> IResult<&[u8], Self> {
                    let (input, value) = nom::number::complete::u8(input)?;
                    let value = match value {
                        #(#match_variants)*
                        _ => Self::Unknown(value),
                    };
                    Ok((input, value))
                }
            }
        };
        result.into()
    } else {
        panic!("Type `{}` is not an enum!", name);
    }
}
