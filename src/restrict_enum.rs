use quote::ToTokens;
use syn::{parse::Parse, punctuated::Punctuated, Data, DeriveInput, Expr, Result, Token};

const TOP_LEVEL_PATH: &'static str = "restrict";
const SECOND_LEVEL_PATH: &'static str = "white_list";

/// (1,2,3,4..=5)
/// 以 , 分割的字面量数值表达式，且不重合

pub(crate) struct AllowedRange {
    pub(crate) white_list: Vec<Expr>,
}
impl Parse for AllowedRange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut white_list = Vec::new();
        Punctuated::<Expr, Token!(,)>::parse_separated_nonempty(input)?
            .into_iter()
            .for_each(|x| {
                white_list.push(x);
            });
        Ok(AllowedRange { white_list })
    }
}

pub(crate) struct RestrictVariant {
    pub(crate) ident: syn::Ident,
    pub(crate) restrict: AllowedRange,
}

impl Parse for RestrictVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Variant>()?;
        let mut white_list = Vec::new();
        for attr in ident.attrs {
            if attr.path.to_token_stream().to_string() == SECOND_LEVEL_PATH {
                let mut current_list = attr.parse_args::<AllowedRange>()?.white_list;
                white_list.append(&mut current_list);
            }
        }
        let restrict = AllowedRange { white_list };
        let ident = ident.ident;
        Ok(RestrictVariant { ident, restrict })
    }
}

pub(crate) struct RestrictEnum {
    pub(crate) pure_enum: DeriveInput,
    pub(crate) variant: Vec<RestrictVariant>,
}
impl RestrictEnum {
    pub(crate) fn clean(input: proc_macro::TokenStream) -> Result<DeriveInput> {
        let mut derive_input = syn::parse::<DeriveInput>(input)?;
        derive_input
            .attrs
            .retain(|attr| attr.path.to_token_stream().to_string() != TOP_LEVEL_PATH);
        if let Data::Enum(ref mut data_enum) = derive_input.data {
            data_enum.variants.iter_mut().for_each(|x| {
                x.attrs
                    .retain(|x| x.path.to_token_stream().to_string() != SECOND_LEVEL_PATH);
            });
        }
        Ok(derive_input)
    }
}
impl Parse for RestrictEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive_input = input.parse::<syn::DeriveInput>()?;
        let pure_enum = Self::clean(derive_input.to_owned().to_token_stream().into())?;
        let mut variant = Vec::new();
        if let Data::Enum(data_enum) = derive_input.data {
            data_enum.variants.into_iter().for_each(|raw_variant| {
                let token = raw_variant.to_token_stream();
                let restrict_variant = syn::parse2::<RestrictVariant>(token);
                if let Ok(x) = restrict_variant {
                    variant.push(x);
                }
            });
        }
        Ok(RestrictEnum { pure_enum, variant })
    }
}
