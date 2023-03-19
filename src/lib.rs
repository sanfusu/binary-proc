#![allow(dead_code)]

use crate::restrict_enum::RestrictEnum;
use bitmap_struct::BitmapStruct;
use bytemap_struct::BytemapStruct;
use container_type::ContainerType;
use proc_macro::TokenStream;
use quote::format_ident;
use syn::parse_macro_input;

extern crate quote;

mod bitmap_struct;
mod bytemap_struct;
mod container_type;
mod literal_pos;
mod restrict_enum;

#[proc_macro_attribute]
pub fn bytemap(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // let types = parse_macro_input!(_attr as ContainerType).types;
    let bytemap = parse_macro_input!(item as BytemapStruct);
    let ident = bytemap.clean_struct.to_owned().ident;
    let clean = bytemap.clean_struct.to_owned();
    let (impl_generics, ty_generics, where_clause) = clean.generics.split_for_impl();
    let mut bits_read = proc_macro2::TokenStream::new();
    let mut iter_fields = proc_macro2::TokenStream::new();
    let mut iter_fields_into = proc_macro2::TokenStream::new();
    let mut next_return = proc_macro2::TokenStream::new();
    for field in bytemap.fields.clone() {
        let field_ident = field.ident;
        let field_pos = field.pos;
        let target_type = field.target_type;
        let field_read = quote::quote! {
            #field_ident: <#target_type>::try_from(value.get(#field_pos).ok_or(#field_pos)?).map_err(|_|{#field_pos})?,
        };
        bits_read.extend(field_read);
        let iter_field_name = format_ident!("{}_iter", field_ident);
        let iter_field = quote::quote! {
            #iter_field_name: <#target_type as ::core::iter::IntoIterator>::IntoIter,
        };
        let iter_field_into = quote::quote! {
            #iter_field_name: self.#field_ident.into_iter(),
        };
        let next_field_return = quote::quote! {
            if (#field_pos).contains(&self._current_idx) {
                self._current_idx += 1;
                return self.#iter_field_name.next();
            }
        };
        iter_fields.extend(iter_field);
        next_return.extend(next_field_return);
        iter_fields_into.extend(iter_field_into);
    }
    let limit = bytemap.fields.last().unwrap().pos_value.end();
    let iter_name = format_ident!("{}Iter", ident);

    quote::quote! {
        #clean
        impl #impl_generics ::core::convert::TryFrom<&[u8]> for #ident #ty_generics #where_clause {
            type Error = ::core::ops::RangeInclusive<usize>;
            fn try_from(value:&[u8])->Result<Self, Self::Error> {
                Ok(Self {
                    #bits_read
                })
            }
        }
        impl #impl_generics ::core::iter::IntoIterator for #ident #ty_generics #where_clause {
            type Item = u8;
            type IntoIter = #iter_name;
            fn into_iter(self) -> Self::IntoIter {
                #iter_name {
                    #iter_fields_into
                    _current_idx: 0usize,
                }
            }
        }
        pub struct #iter_name #ty_generics {
            #iter_fields
            _current_idx:usize,
        }
        impl #impl_generics ::core::iter::Iterator for #iter_name #ty_generics #where_clause {
            type Item = u8;
            fn next(&mut self) -> Option<Self::Item> {
                if self._current_idx > #limit {
                    return None;
                }
                #next_return
                self._current_idx += 1;
                return Some(0);
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn bitmap(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let types = parse_macro_input!(_attr as ContainerType).types;
    let bitmap = parse_macro_input!(item as BitmapStruct);
    let ident = bitmap.clean_struct.to_owned().ident;
    let clean = bitmap.clean_struct.to_owned();
    let (impl_generics, ty_generics, where_clause) = clean.generics.split_for_impl();
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
            impl #impl_generics ::core::convert::TryFrom<#types> for #ident #ty_generics #where_clause {
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
    let (impl_generics, ty_generics, where_clause) = clean_enum.generics.split_for_impl();
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
            impl #impl_generics ::core::convert::TryFrom<#all_type> for #enum_ident  #ty_generics #where_clause {
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
