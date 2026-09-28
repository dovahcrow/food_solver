//! Batch recipe solver for adult dog food, shared by the CLI and MCP frontends.
//!
//! The flow is: ingredient batch weights plus a day count go in, a solved
//! recipe and a per-day nutrient report come out. Requirements come from
//! FEDIAF 2025 Table III-3b adult maintenance figures.
//!
//! Food amounts are batch totals in grams. The day count scales the nutrient
//! requirements and the report basis; it never scales fixed ingredient
//! weights, because a batch weight is a batch weight.

pub mod foods;
pub mod needs;
pub mod nutrient;
pub mod planner;
pub mod recipe;
pub mod units;

pub use foods::{chosen_source, parse_food, Food, FOODS, FOOD_NAMES};
pub use needs::{scale, Profile};
pub use nutrient::{nutrient_value, Nutrient};
pub use planner::{plan, IngredientSpec, PlanRequest, PlanResult, RecipeLine};
pub use recipe::{
    nutrition_report, solve, FoodRow, NeedRequired, NeedSoftness, NutrientReport, Problem,
    RatioConstraint, ReportStatus, Requirement, Solution, SolveStatus,
};

/// FEDIAF 2025 Table III-3b, adult maintenance: mass ratio Ca:P 1:1..2:1.
/// Dimensionless, so it is never scaled by the batch day count.
pub const CA_P_RATIO_MIN: f64 = 1.0;
pub const CA_P_RATIO_MAX: f64 = 2.0;

/// Largest batch the solver will plan, guarding against absurd inputs.
pub const MAX_DAYS: u32 = 3650;
