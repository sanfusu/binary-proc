//! ```
//! // bytemap(len)
//! #[bytemap(64)]
//! struct A {
//!     // 0..=1 为 pad
//!     #[pos(2..=3)]
//!     field1: u16,
//!     // 2..=63 为 pad
//!     field2: u8, // error pos must be specified
//! }
//! // bytemap(packed)
//! struct A {
//!     field1: u8, // 不支持 u8
//!     field2: u32,
//!     field3: u16
//! }
//! ```

use std::ops::RangeInclusive;

use quote::ToTokens;
use syn::{parse::Parse, parse2, Data, DeriveInput, Error, Result};

use crate::literal_pos::range_from_expr;
#[derive(Clone)]
pub(crate) struct ByteField {
    pub(crate) pos: syn::ExprRange,
    pub(crate) ident: syn::Ident,
    pub(crate) target_type: syn::Type,
    pub(crate) pos_value: RangeInclusive<usize>,
}

impl Parse for ByteField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field = syn::Field::parse_named(input)?;
        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path.to_token_stream().to_string() == "pos")
            .ok_or(syn::parse::Error::new_spanned(
                field.to_token_stream(),
                "pos attr must be used",
            ))?;
        let pos = attr.parse_args::<syn::Expr>()?;
        let ident = field
            .to_owned()
            .ident
            .ok_or(syn::parse::Error::new_spanned(
                field.to_token_stream(),
                "only named field is supported",
            ))?;
        let range = if let syn::Expr::Lit(ref lit) = pos {
            parse2(quote::quote!(#lit ..= #lit))?
        } else if let syn::Expr::Range(ref range) = pos {
            range.to_owned()
        } else {
            Err(syn::Error::new_spanned(
                pos.to_token_stream(),
                "Only ExprLit or ExprRange supported",
            ))?
        };
        let target_type = field.ty;
        return Ok(ByteField {
            pos: range,
            ident,
            target_type,
            pos_value: range_from_expr(&pos)?,
        });
    }
}

pub(crate) struct BytemapStruct {
    pub(crate) fields: Vec<ByteField>,
    pub(crate) clean_struct: DeriveInput,
}

impl BytemapStruct {
    pub(crate) fn clean(input: proc_macro::TokenStream) -> Result<DeriveInput> {
        let mut derive_input = syn::parse::<DeriveInput>(input)?;
        derive_input
            .attrs
            .retain(|attr| attr.path.to_token_stream().to_string() != "bytemap");
        if let Data::Struct(ref mut data_struct) = derive_input.data {
            data_struct.fields.iter_mut().for_each(|x| {
                x.attrs
                    .retain(|x| x.path.to_token_stream().to_string() != "pos");
            });
        }
        Ok(derive_input)
    }
}

impl Parse for BytemapStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive_input = DeriveInput::parse(input)?;
        let mut fields = Vec::<ByteField>::new();
        if let Data::Struct(data_struct) = derive_input.to_owned().data {
            for field in data_struct.fields {
                let token = field.to_token_stream();
                let byte_field = syn::parse2::<ByteField>(token)?;
                fields.push(byte_field);
            }
        }
        fields.sort_by(|x, y| x.pos_value.to_owned().cmp(y.pos_value.to_owned()));
        fields
            .clone()
            .into_iter()
            .try_fold(None, |prev: Option<ByteField>, curr| match prev {
                Some(prev_field) => {
                    if prev_field.pos_value.contains(curr.pos_value.start()) {
                        return Err(Error::new_spanned(curr.pos, "position overlapped"));
                    } else {
                        return Ok(Some(curr));
                    }
                }
                None => {
                    return Ok(Some(curr));
                }
            })?;
        Ok(BytemapStruct {
            fields,
            clean_struct: Self::clean(derive_input.to_token_stream().into())?,
        })
    }
}
