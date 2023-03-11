#![allow(dead_code)]

use std::ops::RangeInclusive;

use crate::restrict_enum::RestrictEnum;
use bitmap_struct::BitmapStruct;
use container_type::ContainerType;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Field};

extern crate quote;

mod bitmap_struct;
mod container_type;
mod restrict_enum;

struct _BitmapInfo {
    field_info: Vec<(Field, RangeInclusive<usize>)>,
    container_type: TokenStream,
}

#[proc_macro_attribute]
pub fn bitmap(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let types = parse_macro_input!(_attr as ContainerType).types;
    let bitmap = parse_macro_input!(item as BitmapStruct);
    let ident = bitmap.clean_struct.to_owned().ident;
    let clean = bitmap.clean_struct.to_owned();
    let mut bits_read = proc_macro2::TokenStream::new();
    for field in bitmap.fields {
        let field_ident = field.ident;
        let field_pos = field.pos;
        let field_read = quote::quote! {
            #field_ident: {
                let bits = value.bits(#field_pos);
                bits.read().try_into().map_err(|_| bits.range)
            }?,
        };
        bits_read.extend(field_read);
    }
    quote::quote! {
        #clean
        #(
            impl ::core::convert::TryFrom<#types> for #ident {
                type Error = ::core::ops::RangeInclusive<u32>;
                fn try_from(value:#types)->Result<Self, Self::Error> {
                    Ok(Self {
                        #bits_read
                    })
                }
            }
        )*
    }
    .into()
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
    let restrict_enum = parse_macro_input!(_item as RestrictEnum);
    let clean_enum = restrict_enum.pure_enum;
    let enum_ident = clean_enum.ident.to_owned();
    let all_type = (parse_macro_input!(_attr as ContainerType)).types;
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
