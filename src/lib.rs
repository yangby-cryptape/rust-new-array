// Copyright (C) 2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate proc_macro;

use quote::quote;

mod generate;
pub(crate) mod parse;

use crate::parse::NewArrayDef;

/// Implement the same traits as `[u8; N]` (`N<=32`) for a new type of a fixed-size array
/// automatically.
#[proc_macro_derive(NewArray, attributes(new_array))]
pub fn derive_new_array(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let new_array = syn::parse_macro_input!(input as NewArrayDef);
    let expanded = {
        let impl_traits = new_array
            .config
            .traits
            .iter()
            .map(|t| t.implement(&new_array.name, new_array.length));
        let impl_traits_with_deps = new_array
            .config
            .traits_with_deps
            .iter()
            .map(|t| t.implement(&new_array.name, new_array.length));
        quote!(
            #( #impl_traits )*
            #( #impl_traits_with_deps )*
        )
    };
    expanded.into()
}
