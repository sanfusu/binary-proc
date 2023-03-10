//! ```
//! #[bitmap(u8)]
//! struct A {
//!     #[pos(0..=1)]
//!     field1: bool,
//! }
//! ```

use quote::ToTokens;
use syn::{parse::Parse, Data, DeriveInput, Result};

pub(crate) struct BitField {
    pub(crate) pos: syn::Expr,
    pub(crate) ident: syn::Ident,
    pub(crate) target_type: syn::Type,
}

impl Parse for BitField {
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
        return Ok(BitField {
            pos: attr.parse_args::<syn::Expr>()?,
            ident: field
                .to_owned()
                .ident
                .ok_or(syn::parse::Error::new_spanned(
                    field.to_token_stream(),
                    "only named field is supported",
                ))?,
            target_type: field.ty,
        });
    }
}

pub(crate) struct BitmapStruct {
    pub(crate) fields: Vec<BitField>,
    pub(crate) clean_struct: DeriveInput,
}

impl BitmapStruct {
    pub(crate) fn clean(input: proc_macro::TokenStream) -> Result<DeriveInput> {
        let mut derive_input = syn::parse::<DeriveInput>(input)?;
        derive_input
            .attrs
            .retain(|attr| attr.path.to_token_stream().to_string() != "bitmap");
        if let Data::Struct(ref mut data_struct) = derive_input.data {
            data_struct.fields.iter_mut().for_each(|x| {
                x.attrs
                    .retain(|x| x.path.to_token_stream().to_string() != "pos");
            });
        }
        Ok(derive_input)
    }
}

impl Parse for BitmapStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive_input = DeriveInput::parse(input)?;
        let mut fields = Vec::<BitField>::new();
        if let Data::Struct(data_struct) = derive_input.to_owned().data {
            for field in data_struct.fields {
                let token = field.to_token_stream();
                let bit_field = syn::parse2::<BitField>(token)?;
                fields.push(bit_field);
            }
        }
        Ok(BitmapStruct {
            fields,
            clean_struct: Self::clean(derive_input.to_token_stream().into())?,
        })
    }
}
