// Copyright (C) 2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use quote::quote;

use crate::parse::{DerivedTrait, DerivedTraitWithDeps};

impl DerivedTrait {
    pub(crate) fn implement(self, name: &syn::Ident, length: usize) -> proc_macro2::TokenStream {
        let name_str = &name.to_string();
        let length_lit = syn::LitInt::new(&format!("{}", length), proc_macro2::Span::call_site());
        match self {
            Self::Default => quote!(
                impl ::core::default::Default for #name {
                    #[inline]
                    fn default() -> Self {
                        Self([0; #length_lit])
                    }
                }
            ),
            Self::PartialEq => quote!(
                impl ::core::cmp::PartialEq for #name {
                    #[inline]
                    fn eq(&self, other: &Self) -> bool {
                        (&self.0[..]).eq(&other.0[..])
                    }
                }
            ),
            Self::Eq => quote!(
                impl ::core::cmp::Eq for #name {}
            ),
            Self::PartialOrd => quote!(
                impl ::core::cmp::PartialOrd for #name {
                    #[inline]
                    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                        (&self.0[..]).partial_cmp(&other.0[..])
                    }
                }
            ),
            Self::Ord => quote!(
                impl ::core::cmp::Ord for #name {
                    #[inline]
                    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                        (&self.0[..]).cmp(&other.0[..])
                    }
                }
            ),
            Self::Hash => quote!(
                impl ::core::hash::Hash for #name {
                    #[inline]
                    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                        ::core::hash::Hash::hash(&self.0[..], state)
                    }
                }
            ),
            Self::AsRef => quote!(
                impl ::core::convert::AsRef<[u8]> for #name {
                    #[inline]
                    fn as_ref(&self) -> &[u8] {
                        &self.0[..]
                    }
                }
            ),
            Self::AsMut => quote!(
                impl ::core::convert::AsMut<[u8]> for #name {
                    #[inline]
                    fn as_mut(&mut self) -> &mut [u8] {
                        &mut self.0[..]
                    }
                }
            ),
            Self::From => quote!(
                impl ::core::convert::From<[u8; #length_lit]> for #name {
                    #[inline]
                    fn from(inner: [u8; #length_lit]) -> Self {
                        Self(inner)
                    }
                }
            ),
            Self::Into => quote!(
                impl ::core::convert::Into<[u8; #length_lit]> for #name {
                    #[inline]
                    fn into(self) -> [u8; #length_lit] {
                        self.0
                    }
                }
            ),
            Self::Borrow => quote!(
                impl ::core::borrow::Borrow<[u8]> for #name {
                    #[inline]
                    fn borrow(&self) -> &[u8] {
                        &self.0[..]
                    }
                }
            ),
            Self::BorrowMut => quote!(
                impl ::core::borrow::BorrowMut<[u8]> for #name {
                    #[inline]
                    fn borrow_mut(&mut self) -> &mut [u8] {
                        &mut self.0[..]
                    }
                }
            ),
            Self::Debug => quote!(
                impl ::core::fmt::Debug for #name {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        let s = &self.0[..];
                        if  f.alternate() {
                            write!(f, #name_str)?;
                            write!(f, "([{:#04x}", s[0])?;
                            for v in &s[1..] {
                                write!(f, ", {:#04x}", v)?;
                            }
                        } else {
                            write!(f, #name_str)?;
                            write!(f, "([{}", s[0])?;
                            for v in &s[1..] {
                                write!(f, ", {}", v)?;
                            }
                        }
                        write!(f, "])")
                    }
                }
            ),
            Self::Drop => quote!(
                impl ::core::ops::Drop for #name {
                    #[inline]
                    fn drop(&mut self) {
                        for elem in self.0.iter_mut() {
                            unsafe {
                                ::core::ptr::write_volatile(elem, 0);
                            }
                            ::core::sync::atomic::compiler_fence(
                                ::core::sync::atomic::Ordering::SeqCst);
                        }
                    }
                }
            ),
        }
    }
}

impl DerivedTraitWithDeps {
    pub(crate) fn implement(self, name: &syn::Ident, length: usize) -> proc_macro2::TokenStream {
        let name_str = &name.to_string();
        let length_lit = syn::LitInt::new(&format!("{}", length), proc_macro2::Span::call_site());
        match self {
            Self::Display => quote!(
                impl ::core::fmt::Display for #name {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        let s = &self.0[..];
                        if  f.alternate() {
                            write!(f, #name_str)?;
                            write!(f, "([{:#04x}", s[0])?;
                            for v in &s[1..] {
                                write!(f, ", {:#04x}", v)?;
                            }
                        } else {
                            write!(f, #name_str)?;
                            write!(f, "([{}", s[0])?;
                            for v in &s[1..] {
                                write!(f, ", {}", v)?;
                            }
                        }
                        write!(f, "])")
                    }
                }
            ),
            _ => quote!(),
        }
    }
}
