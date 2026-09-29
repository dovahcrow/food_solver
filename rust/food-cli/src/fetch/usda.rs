//! USDA FoodData Central portal getter.
//!
//! `https://fdc.nal.usda.gov/portal-data/external/{id}` returns the food with
//! its nutrients, each already carrying a value and unit. Values are per 100 g
//! of food, so the result is divided by 100 to get the solver's grams-per-gram.
//!
//! Fatty-acid rows without a chemically identified chain (`PUFA 18:2`, totals)
//! are skipped rather than guessed at: an unspecified `18:2` is not safely
//! linoleic acid.

use anyhow::{anyhow, Context, Error};
use culpa::throws;
use serde_json::Value;

use super::nutrients::USDA_NAMES;
use super::units::{normalize, VITAMIN_D_IU};
use super::Row;

/// The FoodData Central API key, read from `USDA_API_KEY`.
///
/// One key serves every FoodData Central endpoint, so both this module and the
/// SR Legacy getter use this helper. A real key is effectively required in
/// practice; `DEMO_KEY` is the documented public fallback and is rate limited.
pub fn api_key() -> String {
    std::env::var("USDA_API_KEY").unwrap_or_else(|_| "DEMO_KEY".to_string())
}

/// Map a USDA display name onto a solver key.
///
/// `Some(None)` is a name the table deliberately drops; `None` is a name the
/// table does not list at all, which the caller records and skips.
fn map_name(name: &str) -> Option<Option<&'static str>> {
    if let Some((_, value)) = USDA_NAMES.iter().find(|(key, _)| *key == name) {
        return Some(*value);
    }
    // Unnamed MUFA/SFA/PUFA rows are not in the table on purpose.
    if name.starts_with("MUFA") || name.starts_with("SFA") || name.starts_with("PUFA") {
        return Some(None);
    }
    None
}

/// Parse the portal JSON body into a nutrient row.
///
/// Split out from the network call so tests can exercise it on a fixture.
#[throws(Error)]
pub fn parse(body: &str) -> Row {
    let data: Value = serde_json::from_str(body).context("parse USDA response")?;
    let name = data
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("USDA response has no description"))?
        .to_string();

    let mut nutrients: Vec<(&'static str, f64)> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let rows = data
        .get("foodNutrients")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("USDA response has no foodNutrients"))?;

    for row in rows {
        let Some(value) = row.get("value").and_then(Value::as_f64) else {
            continue;
        };
        let nutrient = row
            .get("nutrient")
            .ok_or_else(|| anyhow!("USDA row has no nutrient object"))?;
        let display = nutrient
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("USDA nutrient has no name"))?;
        let unit = nutrient
            .get("nutrientUnit")
            .and_then(|unit| unit.get("name"))
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| anyhow!("USDA nutrient has no unit"))?;

        let Some(key) = map_name(display) else {
            skipped.push(display.to_string());
            continue;
        };
        let Some(key) = key else {
            continue;
        };

        // Vitamin D is reported in IU, which the solver stores as mass.
        let amount = if display == "Vitamin D (D2 + D3), International Units" && unit == "IU" {
            value * VITAMIN_D_IU
        } else {
            normalize(value, unit)
                .ok_or_else(|| anyhow!("USDA unit {unit:?} for {display:?} is unknown"))?
        };

        // Portal values are per 100 g of food.
        nutrients.push((key, amount / 100.0));
    }

    Row {
        name,
        nutrients,
        skipped,
    }
}

/// Fetch one food by its USDA portal id.
#[throws(Error)]
pub fn get(id: u64) -> Row {
    let url = format!("https://fdc.nal.usda.gov/portal-data/external/{id}");
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .body_mut()
        .read_to_string()
        .with_context(|| format!("read {url}"))?;
    parse(&body)?
}
