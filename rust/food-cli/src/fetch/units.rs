//! Unit conversion for the fetchers.
//!
//! Food mass and nutrient mass are grams; energy is joules. `normalize` maps a
//! source-reported unit onto those units.

/// Grams.
pub const G: f64 = 1.0;
/// Milligrams.
pub const MG: f64 = G / 1000.0;
/// Micrograms.
pub const MCG: f64 = MG / 1000.0;
/// Kilojoules.
pub const KJ: f64 = 1000.0;
/// Kilocalories expressed in joules.
pub const KCAL: f64 = 4184.0;

/// Vitamin D activity equivalents; USDA reports vitamin D in IU.
pub const VITAMIN_D_IU: f64 = 0.025 * MCG;

/// Convert an amount in `unit` into the solver's base units.
///
/// Accepts the spellings the two data sources emit, including the two
/// micro-sign characters (`U+03BC` and `U+00B5`). An unknown unit is an error
/// rather than a silent pass-through, so a source change cannot quietly skew
/// the numbers.
pub fn normalize(amount: f64, unit: &str) -> Option<f64> {
    let factor = match unit {
        "g" => G,
        "mg" => MG,
        "μg" | "µg" | "ug" => MCG,
        "kJ" => KJ,
        "kcal" => KCAL,
        _ => return None,
    };
    Some(amount * factor)
}
