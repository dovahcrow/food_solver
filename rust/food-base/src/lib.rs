//! Nutrient vocabulary and food composition, shared by the solver and its
//! build script.
//!
//! This crate exists so `food-core`'s `build.rs` can name the nutrient fields
//! as ordinary Rust values instead of parsing `src/nutrient.rs` as text. A
//! build script cannot depend on the crate it is building, so the shared
//! vocabulary has to live below both.

pub mod food;
pub mod nutrient;

pub use food::Food;
pub use nutrient::{nutrient_value, Nutrient, NUTRIENT_FIELDS};
