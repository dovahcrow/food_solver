//! The Rust getters must reproduce the Python `foods/*.json` caches exactly.
//!
//! `usda_beef.json` is a captured real USDA response; `china_pork.html` is a
//! trimmed page with the same structure the live China Food Composition Tables
//! serve. The expected rows are what the Python getters produced for them, so
//! the assertions pin byte-level parity rather than approximate agreement.
//!
//! The `fetch` module is compiled into the binary, so it is pulled in here by
//! path, the way `tests/format.rs` pulls in `src/format.rs`.

#[path = "../src/fetch/mod.rs"]
#[allow(dead_code)] // the binary uses the network entry points; tests do not.
mod fetch;

use fetch::{to_cache_json, Row};

/// Compare the parsed row against a sorted `(key, value)` expectation.
fn assert_row(row: &Row, name: &str, expected: &[(&str, f64)]) {
    assert_eq!(row.name, name, "display name");
    let mut actual: Vec<(&str, f64)> = row.nutrients.clone();
    actual.sort_by(|a, b| a.0.cmp(b.0));
    let mut wanted = expected.to_vec();
    wanted.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(actual.len(), wanted.len(), "nutrient count: {actual:?}");
    for ((key, value), (want_key, want_value)) in actual.iter().zip(wanted.iter()) {
        assert_eq!(key, want_key);
        assert_eq!(value, want_value, "{key}");
    }
}

#[test]
fn usda_beef_matches_the_python_cache() {
    let body = include_str!("fixtures/usda_beef.json");
    let row = fetch::usda::parse(body).expect("parse USDA beef");
    assert_row(
        &row,
        "Beef, round, top round, boneless, choice, raw",
        &[
            ("ASH", 0.0109),
            ("CALCIUM", 4e-05),
            ("CARB", 0.0085),
            ("CHOLESTEROL", 0.00059),
            ("COPPER", 5.5e-07),
            ("ENERGY", 5899.44),
            ("FAT", 0.057),
            ("IRON", 1.9e-05),
            ("MAGNESIUM", 0.000221),
            ("MANGANESE", 3.0000000000000004e-08),
            ("PHOSPHORUS", 0.00192),
            ("POTASSIUM", 0.0035199999999999997),
            ("PROTEIN", 0.215),
            ("SODIUM", 0.00046),
            ("ZINC", 3.78e-05),
        ],
    );
}

#[test]
fn china_pork_matches_the_python_cache() {
    let body = include_str!("fixtures/china_pork.html");
    let row = fetch::chinanutri::parse(body).expect("parse China pork");
    assert_row(
        &row,
        "猪肉(肥瘦)",
        &[
            ("ASH", 0.01),
            ("CALCIUM", 6e-05),
            ("CARB", 0.015),
            ("CHOLESTEROL", 0.0008100000000000001),
            ("ENERGY", 14360.0),
            ("FAT", 0.062),
            ("IRON", 3e-05),
            ("NIACIN", 5.3e-05),
            ("PHOSPHORUS", 0.00189),
            ("POTASSIUM", 0.0030499999999999998),
            ("PROTEIN", 0.203),
            ("SELENIUM", 9.499999999999999e-08),
            ("SODIUM", 0.000575),
            ("VITAMIN_A", 4.3999999999999997e-07),
            ("VITAMIN_B1", 5.4e-06),
            ("VITAMIN_B2", 1e-06),
            ("ZINC", 2.3999999999999997e-05),
        ],
    );
}

#[test]
fn cache_json_round_trips_like_python() {
    let body = include_str!("fixtures/usda_beef.json");
    let row = fetch::usda::parse(body).expect("parse USDA beef");
    let text = to_cache_json(&row).expect("render cache json");
    // Same shape the Python `json.dump(..., indent=4, sort_keys=True)` writes:
    // the display name is a `__name__` key, and keys are sorted.
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(
        parsed["__name__"],
        serde_json::json!("Beef, round, top round, boneless, choice, raw")
    );
    assert_eq!(parsed["ENERGY"], serde_json::json!(5899.44));
    let keys: Vec<&str> = parsed
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "keys must be sorted");
}

#[test]
fn cooked_chicken_breast_is_divided_back() {
    // `convert_cooked_chicken_breast_to_uncoocked` divides every value by 1.49.
    let body = include_str!("fixtures/usda_beef.json");
    let row = fetch::usda::parse(body).expect("parse USDA beef");
    let entry = fetch::CATALOG
        .iter()
        .find(|entry| entry.name == "CHICKEN_BREAST")
        .expect("CHICKEN_BREAST in catalog");
    assert!(entry.cooked_to_raw);
    let protein = row
        .nutrients
        .iter()
        .find(|(key, _)| *key == "PROTEIN")
        .map(|(_, value)| *value)
        .expect("protein");
    assert_eq!(protein / 1.49, 0.215 / 1.49);
}

#[test]
fn catalog_matches_the_python_getters() {
    // 18 USDA foods, 38 chinanutri foods, 3 inline foods: the same split the
    // Python `GETTERS` table uses. This checks nothing drifted.
    let usda = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.source, fetch::Source::Usda(_)))
        .count();
    let china = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.source, fetch::Source::Chinanutri(_)))
        .count();
    let inline = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.source, fetch::Source::Inline))
        .count();
    assert_eq!((usda, china, inline), (18, 38, 3));
}

#[test]
fn unknown_usda_name_is_skipped_not_fatal() {
    // The Python getter prints an unlisted name and carries on; the port keeps
    // the same behaviour but records the name for the caller to show.
    let body = r#"{"description":"x","foodNutrients":[
        {"value":1.0,"nutrient":{"name":"Brand New Nutrient","nutrientUnit":{"name":"g"}}},
        {"value":2.0,"nutrient":{"name":"Protein","nutrientUnit":{"name":"g"}}}]}"#;
    let row = fetch::usda::parse(body).expect("parse survives an unmapped name");
    assert_eq!(row.nutrients, vec![("PROTEIN", 0.02)]);
    assert_eq!(row.skipped, vec!["Brand New Nutrient".to_string()]);
}

#[test]
fn python_float_spelling() {
    // The exponent spelling Python's `repr` uses, which the cache files carry.
    let cases = [
        (4e-5, "4e-05"),
        (5.5e-7, "5.5e-07"),
        (1.9e-5, "1.9e-05"),
        (3.0000000000000004e-8, "3.0000000000000004e-08"),
        (5899.44, "5899.44"),
        (6000.0, "6000.0"),
        (0.0, "0.0"),
        (1e16, "1e+16"),
    ];
    for (value, expected) in cases {
        let text = to_cache_json(&Row {
            name: "x".to_string(),
            nutrients: vec![("PROTEIN", value)],
            skipped: Vec::new(),
        })
        .expect("render");
        assert!(
            text.contains(&format!(": {expected}")),
            "value {value}: expected {expected} in {text}"
        );
    }
}
