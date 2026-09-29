//! USDA SR Legacy getter.
//!
//! SR Legacy is the older, much richer USDA dataset: the same FoodData Central
//! API serves it, but its records carry the full amino-acid and fatty-acid
//! panels that the Foundation records behind `portal-data/external` omit. That
//! extra depth (amino acids, named fatty acids, pantothenic acid, choline,
//! vitamin K, B12) is why `CHOOSE` points many foods at their SR Legacy row.
//!
//! Rows are never merged: a food's row comes from exactly one source, and one
//! file is written per food per source.
//!
//! Response shape: `{"description": …, "foodNutrients": [{"nutrient": {"name",
//! "unitName"}, "amount": …}]}`. Amounts are per 100 g, so they are divided by
//! 100 to reach the solver's grams-per-gram.

use anyhow::{anyhow, Context, Error};
use culpa::{throw, throws};
use serde_json::Value;

use super::nutrients::map_sr_legacy_name;
use super::units::{normalize, KCAL, KJ, VITAMIN_D_IU};
use super::Row;

/// Parse an SR Legacy food object into a nutrient row.
///
/// Split from the network call so tests can exercise it on a fixture.
#[throws(Error)]
pub fn parse(body: &str) -> Row {
    let data: Value = serde_json::from_str(body).context("parse SR Legacy response")?;
    let name = data
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("SR Legacy response has no description"))?
        .to_string();

    let mut nutrients: Vec<(&'static str, f64)> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let rows = data
        .get("foodNutrients")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("SR Legacy response has no foodNutrients"))?;

    for row in rows {
        // SR Legacy stores the value under `amount`; a missing amount is a
        // row with no measurement, exactly like the portal's missing `value`.
        let Some(amount) = row.get("amount").and_then(Value::as_f64) else {
            continue;
        };
        let nutrient = row
            .get("nutrient")
            .ok_or_else(|| anyhow!("SR Legacy row has no nutrient object"))?;
        let display = nutrient
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("SR Legacy nutrient has no name"))?;
        let unit = nutrient
            .get("unitName")
            .and_then(Value::as_str)
            .map(str::trim)
            .ok_or_else(|| anyhow!("SR Legacy nutrient has no unit"))?;

        // "Energy" appears twice under the same name, once in kJ and once in
        // kcal, and the two published figures are rounded independently. Keep
        // the first and ignore the second so the row cannot carry ENERGY twice.
        if display == "Energy" {
            if nutrients.iter().any(|(key, _)| *key == "ENERGY") {
                continue;
            }
            let joules = match unit {
                "kJ" => amount * KJ,
                "kcal" => amount * KCAL,
                other => throw!(anyhow!("SR Legacy energy unit {other:?} is unknown")),
            };
            nutrients.push(("ENERGY", joules / 100.0));
            continue;
        }

        let Some(key) = map_sr_legacy_name(display) else {
            skipped.push(display.to_string());
            continue;
        };
        let Some(key) = key else {
            continue;
        };

        let amount = if display == "Vitamin D (D2 + D3), International Units" && unit == "IU" {
            amount * VITAMIN_D_IU
        } else {
            normalize(amount, unit)
                .ok_or_else(|| anyhow!("SR Legacy unit {unit:?} for {display:?} is unknown"))?
        };

        nutrients.push((key, amount / 100.0));
    }

    Row {
        name,
        nutrients,
        skipped,
    }
}

/// Fetch one food by its SR Legacy fdcId.
#[throws(Error)]
pub fn get(id: u64) -> Row {
    let url = format!(
        "https://api.nal.usda.gov/fdc/v1/food/{id}?api_key={}",
        super::usda::api_key()
    );
    let body = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?
        .body_mut()
        .read_to_string()
        .with_context(|| format!("read {url}"))?;
    parse(&body)?
}
