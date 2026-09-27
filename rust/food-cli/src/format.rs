//! Number formatting matching the Python frontend's `%g`-family output.
//!
//! Python printed food weights as `f"{amount:.1f}"` after rounding to two
//! significant digits, and report values with C's `%g`. Reproducing those
//! rules keeps the two frontends directly comparable.

/// Round to `digits` significant digits, like C's `%.{digits}g` precision.
pub fn round_significant(value: f64, digits: usize) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    let magnitude = value.abs().log10().floor() as i32;
    let factor = 10f64.powi(digits as i32 - 1 - magnitude);
    (value * factor).round() / factor
}

/// Python's `f"{value:g}"`, i.e. C's `%g` with the default precision of 6.
///
/// Six significant digits, trailing zeros removed, fixed notation while the
/// decimal exponent stays in `[-4, 6)`, and a two-digit exponent otherwise.
pub fn g(value: f64) -> String {
    const PRECISION: i32 = 6;
    if !value.is_finite() {
        return format!("{value}");
    }
    if value == 0.0 {
        return "0".to_string();
    }

    // Round to the working precision first, so the exponent below reflects the
    // rounded value (as C does) instead of the raw one.
    let scientific = format!("{value:.*e}", (PRECISION - 1) as usize);
    let (mantissa, exponent) = match scientific.split_once('e') {
        Some(parts) => parts,
        None => return scientific,
    };
    let exponent: i32 = exponent.parse().unwrap_or(0);

    if (-4..PRECISION).contains(&exponent) {
        let decimals = (PRECISION - 1 - exponent).max(0) as usize;
        trim_zeros(format!("{value:.decimals$}"))
    } else {
        let mantissa = trim_zeros(mantissa.to_string());
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exponent.abs())
    }
}

/// Drop trailing zeros (and a bare trailing point) from a decimal string.
fn trim_zeros(text: String) -> String {
    if !text.contains('.') {
        return text;
    }
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Two-significant-digit rounding, as the Python frontend displayed weights.
pub fn two_significant(value: f64) -> f64 {
    round_significant(value, 2)
}

/// The `%.2f`-style fixed rendering the Python frontend used for bounds.
pub fn fixed2(value: f64) -> String {
    format!("{value:.2}")
}
