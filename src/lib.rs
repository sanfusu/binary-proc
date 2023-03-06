#![allow(dead_code)]

use std::ops::RangeInclusive;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Field};

use crate::value_map::{ContainerType, RestrictEnum};
extern crate quote;
mod value_map;

struct _BitmapInfo {
    field_info: Vec<(Field, RangeInclusive<usize>)>,
    container_type: TokenStream,
}

#[proc_macro_attribute]
pub fn bitmap(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as DeriveInput);
    let mut output = proc_macro2::TokenStream::new();
    if let Data::Struct(mut x) = item.clone().data {
        x.fields.iter_mut().for_each(|field| {
            field.attrs.retain(|attr| {
                if let Some(ident) = attr.path.get_ident() {
                    ident.to_string() != "pos"
                } else {
                    true
                }
            })
        });
        item.data = Data::Struct(x);
        let tok = quote::quote! {
            #item
        };
        output.extend(tok.into_iter());
    }
    output.into()
}
/// ```
/// #[restrict(u8,u16)]
/// enum A {
/// #[white_list(1,2,3)]
///     D1,
/// #[white_list(4,5,6)]
///     D2,
/// #[white_list(8,10,11..=12,34..=54)]
///     D3,
/// #[white_list(100..=344)]
///     D4(U),
/// }
/// ```

#[proc_macro_attribute]
pub fn restrict(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    let container_type = parse_macro_input!(_attr as ContainerType);
    let restrict_enum = parse_macro_input!(_item as RestrictEnum);
    let clean_enum = restrict_enum.pure_enum;
    let enum_ident = clean_enum.ident.to_owned();
    let all_type = container_type.types;
    let mut match_expr = proc_macro2::TokenStream::new();
    restrict_enum.variant.into_iter().for_each(|x| {
        let ident = x.ident;
        let expr = x.restrict.white_list;
        let tmp = quote::quote! {
            #(#expr)|* => Ok(Self::#ident),
        };
        match_expr.extend(tmp);
    });
    quote::quote! {
        #clean_enum
        #(
            impl ::core::convert::TryFrom<#all_type> for #enum_ident {
                type Error = #all_type;
                fn try_from(value: #all_type) -> Result<Self, Self::Error> {
                    match value {
                        #match_expr
                        _ => Err(value),
                    }
                }
            }
        )*
    }
    .into()
}
