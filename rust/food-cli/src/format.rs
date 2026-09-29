//! Number formatting for the CLI report.
//!
//! Food weights print with a resolution that follows their magnitude, so a
//! 500 g staple and a 1.2 g additive are both readable without a long tail of
//! noise digits. Report values still use C's `%g`.

/// C's `%g` with the default precision of 6.
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

/// Format a batch weight with a magnitude-dependent number of decimals.
///
/// | magnitude      | decimals |
/// |----------------|----------|
/// | `>= 100`       | 0        |
/// | `10 .. 100`    | 1        |
/// | `1 .. 10`      | 2        |
/// | `0.01 .. 1`    | 2        |
/// | `< 0.01`       | shown as `0` |
///
/// A negative value is impossible from the solver, but the magnitude rules
/// apply to it the same way (`abs` decides the decimals) so the helper stays
/// total.
pub fn weight(value: f64) -> String {
    if !value.is_finite() {
        return format!("{value}");
    }
    let magnitude = value.abs();
    if magnitude < 0.01 {
        return "0".to_string();
    }
    let decimals = if magnitude >= 100.0 {
        0
    } else if magnitude >= 10.0 {
        1
    } else {
        2
    };
    format!("{value:.decimals$}")
}

/// The `%.2f`-style fixed rendering used for bounds.
pub fn fixed2(value: f64) -> String {
    format!("{value:.2}")
}
