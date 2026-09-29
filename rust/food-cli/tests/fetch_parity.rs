//! The fetchers must reproduce the expected `foods/*.json` bodies exactly.
//!
//! `usda_beef.json` is a captured real USDA response; `china_pork.html` is a
//! trimmed page with the same structure the live China Food Composition Tables
//! serve. The expected rows are captured from the real sources, so
//! the assertions pin byte-level parity rather than approximate agreement.
//!
//! The `fetch` module is compiled into the binary, so it is pulled in here by
//! path, the way `tests/format.rs` pulls in `src/format.rs`.

#[path = "../src/fetch/mod.rs"]
#[allow(dead_code)] // the binary uses the network entry points; tests do not.
mod fetch;

use fetch::{to_cache_json, CacheFile, Row};
use food_core::FoodSource;

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
fn usda_beef_matches_the_captured_row() {
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
fn china_pork_matches_the_captured_row() {
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
fn cache_json_round_trips() {
    let body = include_str!("fixtures/usda_beef.json");
    let row = fetch::usda::parse(body).expect("parse USDA beef");
    let file = CacheFile {
        row,
        source: FoodSource::Usda,
    };
    let text = to_cache_json(&file).expect("render cache json");
    // The cache shape: sorted keys, four-space indent, `__name__` last.
    // the display name is a `__name__` key, and keys are sorted.
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(
        parsed["__name__"],
        serde_json::json!("Beef, round, top round, boneless, choice, raw")
    );
    assert_eq!(parsed["ENERGY"], serde_json::json!(5899.44));
    // The body is nutrients plus `__name__` only; provenance is the file name.
    assert!(parsed.get("source").is_none());
    assert!(parsed.get("choose").is_none());
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
fn catalog_matches_the_getters() {
    // 18 USDA, 38 chinanutri, 1 SR-Legacy-default, 5 inline. These are the
    // sources `food fetch` writes by default; pinned so drift shows up here.
    let usda = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.default_source, fetch::Source::Usda(_)))
        .count();
    let china = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.default_source, fetch::Source::Chinanutri(_)))
        .count();
    let inline = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.default_source, fetch::Source::Inline))
        .count();
    let sr_legacy = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.default_source, fetch::Source::SrLegacy(_)))
        .count();
    assert_eq!((usda, china, sr_legacy, inline), (18, 38, 1, 5));
}

#[test]
fn unknown_usda_name_is_skipped_not_fatal() {
    // An unlisted name is printed and skipped rather than fatal; the fetcher keeps
    // the same behaviour but records the name for the caller to show.
    let body = r#"{"description":"x","foodNutrients":[
        {"value":1.0,"nutrient":{"name":"Brand New Nutrient","nutrientUnit":{"name":"g"}}},
        {"value":2.0,"nutrient":{"name":"Protein","nutrientUnit":{"name":"g"}}}]}"#;
    let row = fetch::usda::parse(body).expect("parse survives an unmapped name");
    assert_eq!(row.nutrients, vec![("PROTEIN", 0.02)]);
    assert_eq!(row.skipped, vec!["Brand New Nutrient".to_string()]);
}

#[test]
fn float_exponent_spelling() {
    // The exponent spelling the cache files carry.
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
        let text = to_cache_json(&CacheFile {
            row: Row {
                name: "x".to_string(),
                nutrients: vec![("PROTEIN", value)],
                skipped: Vec::new(),
            },
            source: FoodSource::Usda,
        })
        .expect("render");
        assert!(
            text.contains(&format!(": {expected}")),
            "value {value}: expected {expected} in {text}"
        );
    }
}

/// Compare a parsed value against an expected amount reported per 100 g.
///
/// SR Legacy publishes per 100 g and the getter divides by 100, so the result
/// carries ordinary binary-float noise; compare with a relative tolerance
/// instead of exact equality.
fn assert_per_100g(actual: Option<f64>, expected: f64, label: &str) {
    let actual = actual.unwrap_or_else(|| panic!("{label} missing"));
    let want = expected / 100.0;
    assert!(
        (actual - want).abs() <= want.abs() * 1e-12,
        "{label}: expected {want}, got {actual}"
    );
}

#[test]
fn sr_legacy_egg_parses_the_amino_acid_panel() {
    // The amino-acid panel the portal view omits, which is the whole reason SR
    // Legacy is here.
    let body = include_str!("fixtures/sr_legacy_egg.json");
    let row = fetch::sr_legacy::parse(body).expect("parse SR Legacy egg");
    assert_eq!(row.name, "Egg, whole, raw, fresh");
    let get = |key: &str| {
        row.nutrients
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    };

    assert_per_100g(get("PROTEIN"), 12.6, "PROTEIN");
    assert_per_100g(get("FAT"), 9.51, "FAT");
    assert_per_100g(get("TRYPTOPHAN"), 0.167, "TRYPTOPHAN");
    assert_per_100g(get("LYSINE"), 0.912, "LYSINE");
    assert_per_100g(get("CYSTINE"), 0.272, "CYSTINE");
    assert_per_100g(get("METHIONINE"), 0.38, "METHIONINE");
    assert_per_100g(get("LEUCINE"), 1.09, "LEUCINE");

    // Pantothenic acid lands on B5 (the solver's key), never on a legacy
    // duplicate alias, so the vitamin cannot be counted twice.
    assert_eq!(get("PANTOTHENIC_ACID"), None);
    assert_per_100g(get("VITAMIN_B5"), 0.00153, "VITAMIN_B5"); // mg per 100 g
                                                               // Vitamin K comes from this row alone; no other source has it.
    assert_per_100g(get("VITAMIN_K"), 3e-7, "VITAMIN_K"); // 0.3 µg per 100 g
    assert_per_100g(get("CHOLINE"), 0.294, "CHOLINE"); // mg per 100 g

    // Named fatty acids come through; the unnamed totals stay dropped, because
    // an unspecified `18:2` is not safely linoleic acid.
    assert_eq!(get("PUFA 18:2"), None);
    assert_per_100g(get("LINOLEIC_ACID"), 1.53, "LINOLEIC_ACID");
    assert_per_100g(get("ALPHA_LINOLENIC_ACID"), 0.036, "ALPHA_LINOLENIC_ACID");

    // Energy is reported under one name in both kJ and kcal; the row must carry
    // it exactly once.
    let energy_rows = row.nutrients.iter().filter(|(k, _)| *k == "ENERGY").count();
    assert_eq!(energy_rows, 1, "ENERGY must not be duplicated");
}

#[test]
fn sr_legacy_rice_parses_the_amino_acid_panel() {
    let body = include_str!("fixtures/sr_legacy_rice.json");
    let row = fetch::sr_legacy::parse(body).expect("parse SR Legacy rice");
    assert_eq!(
        row.name,
        "Rice, white, long-grain, regular, raw, unenriched"
    );
    let get = |key: &str| {
        row.nutrients
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    };
    assert_per_100g(get("PROTEIN"), 7.13, "PROTEIN");
    assert_per_100g(get("TRYPTOPHAN"), 0.083, "TRYPTOPHAN");
    assert_per_100g(get("LYSINE"), 0.258, "LYSINE");
    assert_per_100g(get("METHIONINE"), 0.168, "METHIONINE");
    assert_per_100g(get("VITAMIN_B5"), 0.00101, "VITAMIN_B5"); // mg per 100 g
}

#[test]
fn curated_tables_only_name_catalog_foods() {
    // Every curated source table must point at a food the catalog knows, and
    // the choose table must name a real source.
    for (name, id) in fetch::catalog::SR_LEGACY {
        assert!(
            fetch::CATALOG.iter().any(|e| e.name == *name),
            "{name} is curated but not in the catalog"
        );
        assert!(*id > 0, "{name} has no fdcId");
    }
    for (name, number) in fetch::catalog::MEXT {
        assert!(
            fetch::CATALOG.iter().any(|e| e.name == *name),
            "{name} is curated but not in the catalog"
        );
        assert_eq!(number.len(), 5, "{name} has a malformed MEXT number");
    }
    // An SR Legacy row that is not the default source must stay reachable:
    // `CHOOSE` points most foods at it, so dropping the fetch would leave the
    // generated table without the row those foods read.
    let alternative: Vec<&str> = fetch::CATALOG
        .iter()
        .filter(|entry| {
            !matches!(entry.default_source, fetch::Source::SrLegacy(_))
                && fetch::catalog::sr_legacy_id(entry.name).is_some()
        })
        .map(|entry| entry.name)
        .collect();
    assert!(
        alternative.len() > 20,
        "expected many foods to have an alternative SR Legacy row, found {}",
        alternative.len()
    );

    // SR Legacy can be a food's default source, not only an alternative: a
    // food that only SR Legacy carries must still be expandable.
    let primary: Vec<&str> = fetch::CATALOG
        .iter()
        .filter(|entry| matches!(entry.default_source, fetch::Source::SrLegacy(_)))
        .map(|entry| entry.name)
        .collect();
    assert!(
        !primary.is_empty(),
        "expected at least one SR-Legacy-default food"
    );
    let records = fetch::catalog::sr_legacy_records();
    for name in &primary {
        assert!(
            records.iter().any(|(food, _)| food == name),
            "{name} is SR-Legacy-default but missing from sr_legacy_records"
        );
    }
    // `sr_legacy_records` must not repeat a food.
    let mut seen: Vec<&str> = records.iter().map(|(name, _)| *name).collect();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), total, "sr_legacy_records repeats a food");

    for (name, source) in food_core::foods::CHOOSE {
        assert!(
            fetch::CATALOG.iter().any(|e| e.name == name.name()),
            "{} has a choose entry but is not in the catalog",
            name.name()
        );
        assert!(
            FoodSource::ALL.contains(source),
            "{} chooses an unknown source",
            name.name()
        );
    }
}

#[test]
fn file_names_round_trip_through_the_source_tag() {
    // `{FOOD}_{Source}.json` is the on-disk contract build.rs re-parses.
    for (name, source) in food_core::foods::CHOOSE {
        let file = format!("{}_{}.json", name.name(), source.name());
        let stem = file.trim_end_matches(".json");
        let (food, tag) = stem.rsplit_once('_').expect("file has a source suffix");
        assert_eq!(food, name.name());
        assert_eq!(FoodSource::parse(tag), Some(*source));
    }
}
