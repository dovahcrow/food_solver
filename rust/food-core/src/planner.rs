//! The batch-planning flow both frontends call: request in, result out.

use anyhow::{anyhow, Error};
use culpa::{throw, throws};

use crate::foods::{FoodName, FOODS};
use crate::needs::{scale, Profile};
use crate::nutrient::Nutrient;
use crate::recipe::{
    nutrition_report, solve, FoodRow, NutrientReport, Problem, RatioConstraint, Requirement,
};
use crate::{CA_P_RATIO_MAX, CA_P_RATIO_MIN, MAX_DAYS};

/// One ingredient of the batch. Bounds are batch grams: equal bounds pin a
/// mandatory weight, `[0, upper]` lets the solver choose freely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IngredientSpec {
    pub food: FoodName,
    pub lower: f64,
    pub upper: f64,
    pub minimize_usage: bool,
}

impl IngredientSpec {
    /// A free ingredient the solver may use up to `grams`.
    pub fn optional_upto(food: FoodName, grams: f64) -> Self {
        Self {
            food,
            lower: 0.0,
            upper: grams,
            minimize_usage: false,
        }
    }

    /// A pinned ingredient that must be used in exactly `grams`.
    pub fn fixed(food: FoodName, grams: f64) -> Self {
        Self {
            food,
            lower: grams,
            upper: grams,
            minimize_usage: false,
        }
    }

    /// A free ingredient that should be used minimally when possible.
    pub fn minimize(food: FoodName, grams: f64) -> Self {
        Self {
            food,
            lower: 0.0,
            upper: grams,
            minimize_usage: true,
        }
    }

    pub fn is_optional(&self) -> bool {
        self.lower == 0.0
    }
}

/// A full batch-planning request.
#[derive(Debug, Clone)]
pub struct PlanRequest {
    pub ingredients: Vec<IngredientSpec>,
    pub days: u32,
    pub profile: Profile,
    pub implicit_soft_upper_multiplier: Option<f64>,
    pub detail: bool,
}

impl PlanRequest {
    /// Scaled FEDIAF requirements for this request's profile and day count.
    #[throws(Error)]
    pub fn requirements(&self) -> Vec<(Nutrient, Requirement)> {
        let base = self.profile.nutrient_needs()?;
        scale(&base, self.days)?.into_iter().collect()
    }
}

impl Default for PlanRequest {
    fn default() -> Self {
        Self {
            ingredients: Vec::new(),
            days: 1,
            profile: Profile::default(),
            implicit_soft_upper_multiplier: Some(1.5),
            detail: false,
        }
    }
}

/// Solved grams of one ingredient.
#[derive(Debug, Clone)]
pub struct RecipeLine {
    pub food: FoodName,
    pub grams: f64,
    pub grams_per_day: f64,
    pub optional: bool,
    pub upper_bound: f64,
}

/// Outcome of a planning request, shared by both frontends.
#[derive(Debug, Clone)]
pub struct PlanResult {
    pub optimal: bool,
    pub status: String,
    pub objective: Option<f64>,
    pub days: u32,
    pub recipe: Vec<RecipeLine>,
    pub batch_grams: f64,
    pub nutrition: Vec<NutrientReport>,
    /// The declared ingredient bounds, reported when the batch is infeasible.
    pub attempted: Vec<IngredientSpec>,
}

impl PlanResult {
    /// Batch grams of one food; only valid for an optimal solve.
    pub fn amount(&self, food: FoodName) -> Option<f64> {
        self.recipe
            .iter()
            .find(|line| line.food == food)
            .map(|line| line.grams)
    }

    pub fn nutrient(&self, nutrient: Nutrient) -> Option<&NutrientReport> {
        self.nutrition
            .iter()
            .find(|report| report.nutrient == nutrient)
    }

    /// Daily energy in kcal, derived from the report's display unit.
    pub fn energy_kcal_per_day(&self) -> Option<f64> {
        let report = self.nutrient(Nutrient::Energy)?;
        if !report.value.is_finite() {
            return None;
        }
        Some(report.value * report.scale / crate::units::KCAL)
    }
}

/// Build the solver's ingredient rows from the request.
#[throws(Error)]
fn food_rows(ingredients: &[IngredientSpec]) -> Vec<FoodRow> {
    let mut rows = Vec::new();
    let mut seen: Vec<FoodName> = Vec::new();
    for spec in ingredients {
        if seen.contains(&spec.food) {
            throw!(anyhow!(
                "{} was listed more than once; merge its weights",
                spec.food.name()
            ));
        }
        seen.push(spec.food);
        if !spec.lower.is_finite() || !spec.upper.is_finite() {
            throw!(anyhow!("Bounds for {} must be finite", spec.food.name()));
        }
        if spec.lower < 0.0 || spec.upper < spec.lower {
            throw!(anyhow!(
                "Invalid bounds for {}: {} to {}",
                spec.food.name(),
                spec.lower,
                spec.upper
            ));
        }
        let values = *FOODS
            .get(&spec.food)
            .ok_or_else(|| anyhow!("Unknown food {}", spec.food.name()))?;
        rows.push(FoodRow {
            name: spec.food,
            values,
            lower: spec.lower,
            upper: spec.upper,
            minimize_usage: spec.minimize_usage,
        });
    }
    if rows.is_empty() {
        throw!(anyhow!("Add at least one ingredient"));
    }
    rows
}

/// Solve a batch and collect its recipe and per-day nutrient report.
#[throws(Error)]
pub fn plan(request: &PlanRequest) -> PlanResult {
    if request.days == 0 || request.days > MAX_DAYS {
        throw!(anyhow!("days must be between 1 and {MAX_DAYS}"));
    }

    let foods = food_rows(&request.ingredients)?;
    let needs = request.requirements()?;
    let problem = Problem {
        foods,
        needs: needs.clone(),
        ratios: vec![RatioConstraint {
            numerator: Nutrient::Calcium,
            denominator: Nutrient::Phosphorus,
            minimum: CA_P_RATIO_MIN,
            maximum: CA_P_RATIO_MAX,
        }],
        implicit_soft_upper_multiplier: request.implicit_soft_upper_multiplier,
    };
    let solution = solve(&problem)?;

    if !solution.is_optimal() {
        return PlanResult {
            optimal: false,
            status: solution.status.as_str().to_string(),
            objective: solution.objective,
            days: request.days,
            recipe: Vec::new(),
            batch_grams: 0.0,
            nutrition: Vec::new(),
            attempted: request.ingredients.clone(),
        };
    }

    let grams = solution.grams.unwrap_or_default();
    let recipe: Vec<RecipeLine> = problem
        .foods
        .iter()
        .zip(&grams)
        .map(|(food, grams)| RecipeLine {
            food: food.name,
            grams: *grams,
            grams_per_day: *grams / f64::from(request.days),
            optional: food.lower == 0.0,
            upper_bound: food.upper,
        })
        .collect();
    let nutrition = nutrition_report(&problem, &needs, &grams, request.days, request.detail);
    let batch_grams = recipe.iter().map(|line| line.grams).sum();

    PlanResult {
        optimal: true,
        status: solution.status.as_str().to_string(),
        objective: solution.objective,
        days: request.days,
        recipe,
        batch_grams,
        nutrition,
        attempted: Vec::new(),
    }
}

/// Build the nutrient report for a batch of fixed weights, without solving.
///
/// Every ingredient is taken at its declared weight (`lower`, which the fixed
/// spec sets equal to `upper`), so this answers "what does this exact batch
/// supply?" rather than "what should the batch be?". The profile and day count
/// still drive the requirement bounds the report compares against.
#[throws(Error)]
pub fn report(request: &PlanRequest) -> PlanResult {
    if request.days == 0 || request.days > MAX_DAYS {
        throw!(anyhow!("days must be between 1 and {MAX_DAYS}"));
    }

    let foods = food_rows(&request.ingredients)?;
    for food in &foods {
        if food.lower != food.upper {
            throw!(anyhow!(
                "{} must have an exact weight for a report",
                food.name.name()
            ));
        }
    }
    let needs = request.requirements()?;
    let problem = Problem {
        foods,
        needs: needs.clone(),
        ratios: vec![RatioConstraint {
            numerator: Nutrient::Calcium,
            denominator: Nutrient::Phosphorus,
            minimum: CA_P_RATIO_MIN,
            maximum: CA_P_RATIO_MAX,
        }],
        implicit_soft_upper_multiplier: request.implicit_soft_upper_multiplier,
    };

    let grams: Vec<f64> = problem.foods.iter().map(|food| food.lower).collect();
    let recipe: Vec<RecipeLine> = problem
        .foods
        .iter()
        .zip(&grams)
        .map(|(food, grams)| RecipeLine {
            food: food.name,
            grams: *grams,
            grams_per_day: *grams / f64::from(request.days),
            optional: false,
            upper_bound: food.upper,
        })
        .collect();
    let nutrition = nutrition_report(&problem, &needs, &grams, request.days, request.detail);
    let batch_grams = recipe.iter().map(|line| line.grams).sum();

    PlanResult {
        optimal: true,
        status: "reported".to_string(),
        objective: None,
        days: request.days,
        recipe,
        batch_grams,
        nutrition,
        attempted: Vec::new(),
    }
}

/// Resolve a user-supplied food name to its canonical name.
#[throws(Error)]
pub fn resolve_food(name: &str) -> FoodName {
    FoodName::parse(name)
        .ok_or_else(|| anyhow!("Unknown food {name:?}; call list_foods for valid names"))?
}
