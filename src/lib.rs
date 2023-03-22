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
    let mut bytes_read_from_le = proc_macro2::TokenStream::new();
    let mut bytes_read_from_be = proc_macro2::TokenStream::new();
    let mut le_iter_fields = proc_macro2::TokenStream::new();
    let mut be_iter_fields = proc_macro2::TokenStream::new();
    let mut le_iter_fields_into = proc_macro2::TokenStream::new();
    let mut be_iter_fields_into = proc_macro2::TokenStream::new();
    let mut next_return = proc_macro2::TokenStream::new();
    for field in bytemap.fields.clone() {
        let field_ident = field.ident;
        let field_pos = field.pos;
        let target_type = field.target_type;
        let field_read_from_le = quote::quote! {
            #field_ident: <#target_type>::try_from(::binary::endian::Le(value.0.get(#field_pos).ok_or(#field_pos)?)).map_err(|_|{#field_pos})?,
        };
        let field_read_from_be = quote::quote! {
            #field_ident: <#target_type>::try_from(::binary::endian::Be(value.0.get(#field_pos).ok_or(#field_pos)?)).map_err(|_|{#field_pos})?,
        };
        bytes_read_from_le.extend(field_read_from_le);
        bytes_read_from_be.extend(field_read_from_be);
        let iter_field_name = format_ident!("{}_iter", field_ident);
        let le_iter_field = quote::quote! {
            #iter_field_name: <#target_type as ::binary::endian::IntoLeIter>::IntoIter,
        };
        let be_iter_field = quote::quote! {
            #iter_field_name: <#target_type as ::binary::endian::IntoBeIter>::IntoIter,
        };
        let le_iter_field_into = quote::quote! {
            #iter_field_name: self.#field_ident.into_leiter(),
        };
        let be_iter_field_into = quote::quote! {
            #iter_field_name: self.#field_ident.into_beiter(),
        };
        let next_field_return = quote::quote! {
            if (#field_pos).contains(&self._current_idx) {
                self._current_idx += 1;
                return self.#iter_field_name.next();
            }
        };
        le_iter_fields.extend(le_iter_field);
        be_iter_fields.extend(be_iter_field);
        next_return.extend(next_field_return);
        le_iter_fields_into.extend(le_iter_field_into);
        be_iter_fields_into.extend(be_iter_field_into);
    }
    let limit = bytemap.fields.last().unwrap().pos_value.end();
    let le_iter_name = format_ident!("{}LeIter", ident);
    let be_iter_name = format_ident!("{}BeIter", ident);

    quote::quote! {
        #clean
        impl #impl_generics ::core::convert::TryFrom<::binary::endian::Le<&[u8]>> for #ident #ty_generics #where_clause {
            type Error = ::core::ops::RangeInclusive<usize>;
            fn try_from(value: ::binary::endian::Le<&[u8]>)->Result<Self, Self::Error> {
                Ok(Self {
                    #bytes_read_from_le
                })
            }
        }
        impl #impl_generics ::core::convert::TryFrom<::binary::endian::Be<&[u8]>> for #ident #ty_generics #where_clause {
            type Error = ::core::ops::RangeInclusive<usize>;
            fn try_from(value: ::binary::endian::Be<&[u8]>)->Result<Self, Self::Error> {
                Ok(Self {
                    #bytes_read_from_be
                })
            }
        }
        pub struct #le_iter_name #ty_generics {
            #le_iter_fields
            _current_idx:usize,
        }
        impl #impl_generics ::core::iter::Iterator for #le_iter_name #ty_generics #where_clause {
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
        impl #impl_generics ::binary::endian::IntoLeIter for #ident #ty_generics #where_clause {
            type Item = u8;
            type IntoIter = #le_iter_name;
            fn into_leiter(self) -> Self::IntoIter {
                #le_iter_name {
                    #le_iter_fields_into
                    _current_idx: 0usize,
                }
            }
        }
        pub struct #be_iter_name #ty_generics {
            #be_iter_fields
            _current_idx:usize,
        }
        impl #impl_generics ::core::iter::Iterator for #be_iter_name #ty_generics #where_clause {
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
        impl #impl_generics ::binary::endian::IntoBeIter for #ident #ty_generics #where_clause {
            type Item = u8;
            type IntoIter = #be_iter_name;
            fn into_beiter(self) -> Self::IntoIter {
                #be_iter_name {
                    #be_iter_fields_into
                    _current_idx: 0usize,
                }
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
