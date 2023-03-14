use quote::ToTokens;
use std::ops::RangeInclusive;
use syn::{parse2, Error, RangeLimits, Result};

pub(crate) fn range_from_expr(range: syn::PatRange) -> Result<RangeInclusive<usize>> {
    let lo_expr_token = range.lo.to_token_stream();
    let lo_lit = parse2::<syn::LitInt>(lo_expr_token)?;
    let lo_value = lo_lit.base10_parse::<usize>()?;

    match range.limits {
        RangeLimits::Closed(_) => {}
        _ => {
            return Err(Error::new_spanned(
                range.limits.to_token_stream(),
                "Only RangeInclusive supported",
            ));
        }
    };

    let hi_expr_token = range.hi.to_token_stream();
    let hi_lit = parse2::<syn::LitInt>(hi_expr_token)?;
    let hi_value = hi_lit.base10_parse::<usize>()?;

    Ok(lo_value..=hi_value)
}
