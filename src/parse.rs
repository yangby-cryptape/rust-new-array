// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syn::{parse::Result as ParseResult, spanned::Spanned as _, Error as SynError};

const ATTR_NAME: &str = "new_array";
const ATTR_DERIVE: &str = "derive";
const ATTR_DERIVE_WITH_DEPS: &str = "derive_with_deps";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DerivedTrait {
    // ::core::default
    Default,
    // ::core::cmp
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    // ::core::hash
    Hash,
    // ::core::convert
    AsRef,
    AsMut,
    From,
    Into,
    // ::core::borrow
    Borrow,
    BorrowMut,
    // ::core::fmt
    Debug,
    // ::core::ops
    Drop,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DerivedTraitWithDeps {
    Display,
    FromStr,
}

//    IntoIterator,
//    FixedSizeArray,
// "IntoIterator" => Ok(Self::IntoIterator),
// "FixedSizeArray" => Ok(Self::FixedSizeArray),

pub(crate) struct NewArrayDef {
    pub(crate) name: syn::Ident,
    pub(crate) length: usize,
    pub(crate) config: NewArrayConf,
}

pub(crate) struct NewArrayConf {
    pub(crate) traits: Vec<DerivedTrait>,
    pub(crate) traits_with_deps: Vec<DerivedTraitWithDeps>,
}

impl DerivedTrait {
    pub(crate) fn parse_from_input(input: &str, span: proc_macro2::Span) -> ParseResult<Self> {
        match input {
            "Default" => Ok(Self::Default),
            "PartialEq" => Ok(Self::PartialEq),
            "PartialOrd" => Ok(Self::PartialOrd),
            "Eq" => Ok(Self::Eq),
            "Ord" => Ok(Self::Ord),
            "Hash" => Ok(Self::Hash),
            "AsRef" => Ok(Self::AsRef),
            "AsMut" => Ok(Self::AsMut),
            "From" => Ok(Self::From),
            "Into" => Ok(Self::Into),
            "Borrow" => Ok(Self::Borrow),
            "BorrowMut" => Ok(Self::BorrowMut),
            "Debug" => Ok(Self::Debug),
            "Drop" => Ok(Self::Drop),
            _ => Err(SynError::new(span, "this attribute was unknown")),
        }
    }
}

impl DerivedTraitWithDeps {
    pub(crate) fn parse_from_input(input: &str, span: proc_macro2::Span) -> ParseResult<Self> {
        match input {
            "Display" => Ok(Self::Display),
            "FromStr" => Ok(Self::FromStr),
            _ => Err(SynError::new(span, "this attribute was unknown")),
        }
    }
}

impl syn::parse::Parse for NewArrayDef {
    fn parse(input: syn::parse::ParseStream) -> ParseResult<Self> {
        let derive_input: syn::DeriveInput = input.parse()?;
        let attrs_span = derive_input.span();
        let syn::DeriveInput {
            attrs, ident, data, ..
        } = derive_input;
        let ident_span = ident.span();
        match data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Unnamed(unnamed_fields) => {
                    let fields = unnamed_fields.unnamed.into_iter().collect::<Vec<_>>();
                    if fields.len() != 1 {
                        Err(SynError::new(ident_span, "should have only one field"))
                    } else {
                        let field = &fields[0];
                        for attr in &field.attrs[..] {
                            if let syn::AttrStyle::Outer = attr.style {
                                let meta = attr.parse_meta().map_err(|_| {
                                    SynError::new(attr.span(), "failed to parse the attributes")
                                })?;
                                if meta.path().is_ident(ATTR_NAME) {
                                    return Err(SynError::new(
                                        meta.span(),
                                        "should not be a field attribute",
                                    ));
                                }
                            }
                        }
                        match field.ty {
                            syn::Type::Array(ref ta) => {
                                match ta.elem.as_ref() {
                                    syn::Type::Path(ref tp) => {
                                        if tp.qself.is_none() && tp.path.is_ident("u8") {
                                            Ok(())
                                        } else {
                                            Err(SynError::new(
                                                tp.path.span(),
                                                "the type should be `u8`",
                                            ))
                                        }
                                    }
                                    _ => Err(SynError::new(
                                        ta.elem.span(),
                                        "the type should be `u8`",
                                    )),
                                }?;
                                match ta.len {
                                    syn::Expr::Lit(ref el) => match el.lit {
                                        syn::Lit::Int(ref li) => {
                                            let length = li.base10_parse::<usize>()?;
                                            if length <= 32 {
                                                Err(SynError::new(
                                                    li.span(),
                                                    "the length is smaller than or equal to 32, \
                                                    don't have to derive `NewArray`",
                                                ))
                                            } else {
                                                let config = parse_attrs(attrs_span, &attrs[..])?;
                                                Ok(Self {
                                                    name: ident,
                                                    length,
                                                    config,
                                                })
                                            }
                                        }
                                        _ => Err(SynError::new(
                                            el.lit.span(),
                                            "should be an integer literal",
                                        )),
                                    },
                                    _ => Err(SynError::new(
                                        ta.len.span(),
                                        "should be an integer literal",
                                    )),
                                }
                            }
                            _ => Err(SynError::new(
                                field.span(),
                                "the field should be a fixed size array type",
                            )),
                        }
                    }
                }
                _ => Err(SynError::new(ident_span, "only support unnamed fields")),
            },
            _ => Err(SynError::new(ident_span, "only support structs")),
        }
    }
}

impl ::std::default::Default for NewArrayConf {
    fn default() -> Self {
        Self {
            traits: Vec::new(),
            traits_with_deps: Vec::new(),
        }
    }
}

impl NewArrayConf {
    fn apply_attrs(&mut self, meta: &syn::Meta) -> ParseResult<()> {
        if !(meta.path().is_ident(ATTR_DERIVE) || meta.path().is_ident(ATTR_DERIVE_WITH_DEPS)) {
            return Err(SynError::new(
                meta.path().span(),
                "this attribute was unknown",
            ));
        }
        match meta {
            syn::Meta::Path(path) => Err(SynError::new(
                path.span(),
                "this attribute should not be a path",
            )),
            syn::Meta::List(list) => {
                let mut path_params = Vec::new();
                for nested_meta in list.nested.iter() {
                    match nested_meta {
                        syn::NestedMeta::Meta(meta) => match meta {
                            syn::Meta::Path(path) => {
                                if path_params.iter().any(|tmp| tmp == &path) {
                                    return Err(SynError::new(
                                        path.span(),
                                        "this attribute has been set twice",
                                    ));
                                } else {
                                    path_params.push(path);
                                }
                            }
                            _ => {
                                return Err(SynError::new(
                                    meta.span(),
                                    "this attribute should be a path",
                                ));
                            }
                        },
                        syn::NestedMeta::Lit(lit) => {
                            return Err(SynError::new(
                                lit.span(),
                                "this attribute should not be a literal",
                            ));
                        }
                    }
                }
                if path_params.is_empty() {
                    return Err(SynError::new(
                        list.span(),
                        "this attribute should not be empty",
                    ));
                }
                match list
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        SynError::new(list.path.span(), "this attribute should be a single ident")
                    })?
                    .to_string()
                    .as_ref()
                {
                    ATTR_DERIVE => {
                        self.update_derived_traits(&path_params)?;
                    }
                    ATTR_DERIVE_WITH_DEPS => {
                        self.update_derived_traits_with_deps(&path_params)?;
                    }
                    attr => {
                        return Err(SynError::new(
                            list.path.span(),
                            format!("unsupport attribute `{}`", attr),
                        ));
                    }
                }
                Ok(())
            }
            syn::Meta::NameValue(name_value) => Err(SynError::new(
                name_value.span(),
                "this attribute should not be a name-value pair",
            )),
        }
    }

    fn update_derived_traits<'a>(&mut self, path_params: &[&syn::Path]) -> ParseResult<()> {
        for p in path_params.iter() {
            let s = p
                .get_ident()
                .ok_or_else(|| SynError::new(p.span(), "this attribute should be a single ident"))?
                .to_string();
            let dt = DerivedTrait::parse_from_input(&s, p.span())?;
            if self.traits.iter().any(|tmp| tmp == &dt) {
                return Err(SynError::new(
                    p.span(),
                    "this attribute has already been set",
                ));
            }
            self.traits.push(dt);
        }
        Ok(())
    }

    fn update_derived_traits_with_deps<'a>(
        &mut self,
        path_params: &[&syn::Path],
    ) -> ParseResult<()> {
        for p in path_params.iter() {
            let s = p
                .get_ident()
                .ok_or_else(|| SynError::new(p.span(), "this attribute should be a single ident"))?
                .to_string();
            let dt = DerivedTraitWithDeps::parse_from_input(&s, p.span())?;
            if self.traits_with_deps.iter().any(|tmp| tmp == &dt) {
                return Err(SynError::new(
                    p.span(),
                    "this attribute has already been set",
                ));
            }
            self.traits_with_deps.push(dt);
        }
        Ok(())
    }
}

fn parse_attrs(span: proc_macro2::Span, attrs: &[syn::Attribute]) -> ParseResult<NewArrayConf> {
    let mut conf = NewArrayConf::default();
    for attr in attrs.iter() {
        if let syn::AttrStyle::Outer = attr.style {
            let meta = attr
                .parse_meta()
                .map_err(|_| SynError::new(span, "failed to parse the attributes"))?;
            match meta {
                syn::Meta::Path(path) => {
                    if path.is_ident(ATTR_NAME) {
                        return Err(SynError::new(
                            path.span(),
                            "the attribute should not be a path",
                        ));
                    }
                }
                syn::Meta::List(list) => {
                    if list.path.is_ident(ATTR_NAME) {
                        if list.nested.is_empty() {
                            return Err(SynError::new(
                                list.span(),
                                "this attribute should not be empty",
                            ));
                        }
                        for nested_meta in list.nested.iter() {
                            match nested_meta {
                                syn::NestedMeta::Meta(meta) => conf.apply_attrs(meta)?,
                                syn::NestedMeta::Lit(lit) => {
                                    return Err(SynError::new(
                                        lit.span(),
                                        "the attribute in nested meta should not be a literal",
                                    ));
                                }
                            }
                        }
                    }
                }
                syn::Meta::NameValue(name_value) => {
                    if name_value.path.is_ident(ATTR_NAME) {
                        return Err(SynError::new(
                            name_value.span(),
                            "the attribute should not be a name-value pair",
                        ));
                    }
                }
            }
        }
    }
    Ok(conf)
}
