use quote::ToTokens;
use std::ops::RangeInclusive;
use syn::{
    Error,
    Expr::{self, Lit},
    ExprLit,
    Lit::Int,
    RangeLimits, Result,
};

pub(crate) fn range_from_expr(expr: &Expr) -> Result<RangeInclusive<usize>> {
    if let Expr::Range(range) = expr {
        let mut lo_value = 0;
        if let Some(Lit(ExprLit {
            lit: Int(int_lit), ..
        })) = range.from.as_deref()
        {
            lo_value = int_lit.base10_parse::<usize>()?;
        }

        let hi = range.to.as_deref().ok_or(Error::new_spanned(
            expr.to_token_stream(),
            "The range should be bounded",
        ))?;
        let mut hi_value = if let Lit(ExprLit {
            lit: Int(int_lit), ..
        }) = hi
        {
            int_lit.base10_parse::<usize>()?
        } else {
            Err(Error::new_spanned(expr, ""))?
        };
        if let RangeLimits::HalfOpen(_) = range.limits {
            hi_value -= 1
        }
        return Ok(lo_value..=hi_value);
    } else if let Expr::Lit(ExprLit {
        lit: Int(int_lit), ..
    }) = expr
    {
        let lit_value = int_lit.base10_parse::<usize>()?;
        return Ok(lit_value..=lit_value);
    } else {
        return Err(Error::new_spanned(
            expr.to_token_stream(),
            "Only literal range or literal int is suppored",
        ));
    }
}
