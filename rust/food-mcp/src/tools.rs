//! The MCP tool surface, thin wrappers over `food-core`.

use anyhow::{anyhow, Error};
use culpa::{throw, throws};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use serde::{Deserialize, Serialize};

use food_core::{
    dog_needs, parse_food, plan, scale, IngredientSpec, NeedRequired, NeedSoftness, PlanRequest,
    Profile, FOOD_NAMES, MAX_DAYS,
};

/// One ingredient of the batch.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Ingredient {
    /// Food name such as RICE, EGG or CHICKEN_BREAST. Case-insensitive; a
    /// leading `Food.` is ignored.
    pub food: String,
    /// Batch grams, not per-day grams. Multiply a daily amount by the day
    /// count before passing it here.
    pub grams: f64,
    /// False pins the batch to exactly this weight. True lets the solver use
    /// any amount from 0 up to this weight.
    #[serde(default)]
    pub optional: bool,
    /// Among otherwise equivalent recipes, prefer less of this ingredient.
    /// Only meaningful with `optional = true`.
    #[serde(default)]
    pub minimize_usage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SolveRequest {
    /// Ingredients of the batch with their weights in grams.
    pub ingredients: Vec<Ingredient>,
    /// Days the batch covers; scales the nutrient requirements.
    #[serde(default = "one")]
    pub days: u32,
    /// Dog body weight in kg.
    #[serde(default = "seven")]
    pub weight: f64,
    /// Dog age in years; adults only (>= 1).
    #[serde(default = "three")]
    pub age: f64,
    /// False: 95 kcal/kg^0.75 profile. True: 110 profile for ordinary
    /// activity, not working dogs.
    #[serde(default)]
    pub active: bool,
    /// Override the estimated daily energy in kcal.
    #[serde(default)]
    pub daily_kcal: Option<f64>,
    /// Soft upper bound as a multiple of an open-ended minimum; must be
    /// greater than 1. Null disables it.
    #[serde(default = "default_multiplier")]
    pub implicit_soft_upper_multiplier: Option<f64>,
    /// Include each food's contribution to every nutrient.
    #[serde(default)]
    pub detail: bool,
}

fn one() -> u32 {
    1
}
fn three() -> f64 {
    3.0
}
fn seven() -> f64 {
    7.0
}
fn default_multiplier() -> Option<f64> {
    Some(1.5)
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct NeedsRequest {
    /// Body weight in kg.
    #[serde(default = "seven")]
    pub weight: f64,
    /// Days the batch covers.
    #[serde(default = "one")]
    pub days: u32,
    /// Age in years; adults only (>= 1).
    #[serde(default = "three")]
    pub age: f64,
    /// False: 95 kcal/kg^0.75 profile. True: 110 profile.
    #[serde(default)]
    pub active: bool,
    /// Override the estimated daily energy in kcal.
    #[serde(default)]
    pub daily_kcal: Option<f64>,
}

/// The MCP server. Stateless: every call rebuilds its problem from the inputs.
#[derive(Debug, Clone)]
pub struct FoodSolver {
    tool_router: ToolRouter<Self>,
}

impl Default for FoodSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl FoodSolver {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// List every food the solver knows.
    #[tool(name = "list_foods", description = "List every food the solver knows.")]
    pub fn list_foods(&self) -> Result<String, String> {
        render(list_foods_payload())
    }

    /// Adult dog maintenance requirements for a profile.
    #[tool(
        name = "get_needs",
        description = "Adult dog maintenance requirements from FEDIAF 2025 Table \
                       III-3b for this profile, per day and for the whole batch."
    )]
    pub fn get_needs(
        &self,
        Parameters(request): Parameters<NeedsRequest>,
    ) -> Result<String, String> {
        render(get_needs_payload(&request))
    }

    /// Solve ingredient weights and report the per-day nutrients.
    #[tool(
        name = "solve_recipe",
        description = "Solve ingredient weights and return the recipe plus the \
                       per-day nutrient report. Ingredient weights are batch \
                       totals; the day count scales the nutrient requirements only."
    )]
    pub fn solve_recipe(
        &self,
        Parameters(request): Parameters<SolveRequest>,
    ) -> Result<String, String> {
        render(solve_recipe_payload(&request))
    }
}

/// Bridge a fallible tool body onto the `Result<_, String>` the tool macro
/// requires: `rmcp` renders a tool error through `IntoCallToolResult`, which
/// only `String`, `ContentBlock`, `()` and its own error types implement.
fn render(result: Result<String, Error>) -> Result<String, String> {
    result.map_err(|error| error.to_string())
}

/// Every food the solver knows.
#[throws(Error)]
fn list_foods_payload() -> String {
    to_json(&serde_json::json!({
        "count": FOOD_NAMES.len(),
        "foods": FOOD_NAMES,
    }))?
}

/// Scaled requirement table for one profile.
#[throws(Error)]
fn get_needs_payload(request: &NeedsRequest) -> String {
    let profile = Profile {
        age: request.age,
        weight: request.weight,
        active: request.active,
        daily_kcal: request.daily_kcal,
    };
    let base = dog_needs(profile)?;
    let scaled = scale(&base, request.days)?;
    let days = f64::from(request.days);
    let rows: Vec<serde_json::Value> = scaled
        .iter()
        .map(|(nutrient, need)| {
            serde_json::json!({
                "nutrient": nutrient.name(),
                "minimum_per_day": need.minimum / days,
                "maximum_per_day": need.maximum.map(|value| value / days),
                "required": need.required == NeedRequired::Required,
                "soft": need.softness == NeedSoftness::Soft,
            })
        })
        .collect();
    to_json(&serde_json::json!({
        "days": request.days,
        "active": request.active,
        "daily_kcal": request.daily_kcal,
        "nutrients": rows,
    }))?
}

/// Solve one batch and serialise its recipe and nutrient report.
#[throws(Error)]
fn solve_recipe_payload(request: &SolveRequest) -> String {
    let plan_request = build_request(request)?;
    let result = plan(&plan_request)?;
    to_json(&plan_payload(&result, &plan_request, request.detail))?
}

/// Validate and translate the tool request into a planning request.
#[throws(Error)]
fn build_request(request: &SolveRequest) -> PlanRequest {
    if request.days == 0 || request.days > MAX_DAYS {
        throw!(anyhow!("days must be between 1 and {MAX_DAYS}"));
    }
    let mut ingredients = Vec::new();
    for item in &request.ingredients {
        let food = parse_food(&item.food).ok_or_else(|| {
            anyhow!(
                "Unknown food {:?}; call list_foods for valid names",
                item.food
            )
        })?;
        if !item.grams.is_finite() || item.grams < 0.0 {
            throw!(anyhow!("grams for {food} must be a non-negative number"));
        }
        ingredients.push(if item.optional {
            IngredientSpec {
                food,
                lower: 0.0,
                upper: item.grams,
                minimize_usage: item.minimize_usage,
            }
        } else {
            IngredientSpec::fixed(food, item.grams)
        });
    }
    if ingredients.is_empty() {
        throw!(anyhow!("Add at least one ingredient"));
    }
    PlanRequest {
        ingredients,
        days: request.days,
        profile: Profile {
            age: request.age,
            weight: request.weight,
            active: request.active,
            daily_kcal: request.daily_kcal,
        },
        implicit_soft_upper_multiplier: request.implicit_soft_upper_multiplier,
        detail: request.detail,
    }
}

/// Serialise a solved plan into the tool's structured payload.
fn plan_payload(
    result: &food_core::PlanResult,
    request: &PlanRequest,
    detail: bool,
) -> serde_json::Value {
    let recipe: Vec<serde_json::Value> = result
        .recipe
        .iter()
        .map(|line| {
            serde_json::json!({
                "food": line.food,
                "grams": round6(line.grams),
                "grams_per_day": round6(line.grams_per_day),
                "optional": line.optional,
                "maximum_grams": round6(line.upper_bound),
            })
        })
        .collect();

    let mut payload = serde_json::json!({
        "optimal": result.optimal,
        "status": result.status,
        "days": result.days,
        "objective": result.objective.map(round9),
        "recipe": recipe,
        "batch_grams": round6(result.batch_grams),
        "nutrition": Vec::<serde_json::Value>::new(),
        "energy_per_day_kcal": serde_json::Value::Null,
        "per_day_basis": "nutrition amounts and bounds are per day",
    });

    if result.optimal {
        let nutrition: Vec<serde_json::Value> = result
            .nutrition
            .iter()
            .map(|report| {
                let components: Vec<serde_json::Value> = if detail {
                    report
                        .components
                        .iter()
                        .map(|component| {
                            serde_json::json!({
                                "food": component.food,
                                "amount": round6(component.amount),
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                serde_json::json!({
                    "nutrient": report.nutrient.name(),
                    "unit": report.unit,
                    "amount_per_day": round6(report.value),
                    "minimum_per_day": round6(report.minimum),
                    "maximum_per_day": report.maximum.map(round6),
                    "maximum_is_implicit": report.implicit_upper,
                    "status": report.status.as_str(),
                    "required": report.required == NeedRequired::Required,
                    "incomplete_data_from": report.missing,
                    "components_per_day": components,
                })
            })
            .collect();
        payload["nutrition"] = serde_json::Value::Array(nutrition);
        if let Some(kcal) = result.energy_kcal_per_day() {
            payload["energy_per_day_kcal"] = serde_json::json!(round3(kcal));
        }
        return payload;
    }

    let attempted: Vec<serde_json::Value> = result
        .attempted
        .iter()
        .map(|spec| {
            serde_json::json!({
                "food": spec.food,
                "optional": spec.is_optional(),
                "grams_lower_bound": round6(spec.lower),
                "grams_upper_bound": round6(spec.upper),
            })
        })
        .collect();
    payload["attempted_recipe"] = serde_json::Value::Array(attempted);
    payload["hint"] = serde_json::json!(
        "The constraints are infeasible: relax a pinned ingredient weight \
         (mark it optional), add an ingredient, or change the daily energy \
         estimate."
    );
    let _ = request;
    payload
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
fn round6(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}
fn round9(value: f64) -> f64 {
    (value * 1e9).round() / 1e9
}

#[throws(Error)]
fn to_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value)?
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FoodSolver {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Solve dog food batches. Give ingredients with their batch weights and \
             the day count to receive the solved recipe plus a per-day nutrient \
             report against FEDIAF 2025 adult maintenance requirements. Ingredient \
             amounts are batch totals; the day count scales the requirements, not \
             the fixed weights.",
        )
    }
}
