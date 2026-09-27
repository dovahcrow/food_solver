//! Base units used throughout the solver.
//!
//! Food mass and nutrient mass are in grams; energy is in joules.

/// Grams.
pub const G: f64 = 1.0;
/// Milligrams.
pub const MG: f64 = G / 1000.0;
/// Micrograms.
pub const MCG: f64 = MG / 1000.0;
/// Kilograms.
pub const KG: f64 = 1000.0 * G;
/// Kilojoules.
pub const KJ: f64 = 1000.0;
/// Kilocalories expressed in joules.
pub const KCAL: f64 = 4184.0;

/// Vitamin A activity equivalents.
pub const VITAMIN_A_IU: f64 = 0.3 * MCG;
/// Vitamin D activity equivalents.
pub const VITAMIN_D_IU: f64 = 0.025 * MCG;
/// Vitamin E as natural RRR-alpha-tocopherol; not for synthetic preparations.
pub const VITAMIN_E_IU: f64 = 0.67 * MG;
