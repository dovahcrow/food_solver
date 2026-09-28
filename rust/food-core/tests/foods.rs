//! The embedded food table must stay complete and self-consistent.

use food_core::foods;
use food_core::{chosen_source, parse_food, Food, Nutrient, FOODS, FOOD_NAMES};

#[test]
fn every_named_food_is_in_the_map() {
    assert_eq!(FOODS.len(), FOOD_NAMES.len());
    for name in FOOD_NAMES {
        assert!(FOODS.contains_key(name), "{name} missing from FOODS");
        assert_eq!(parse_food(name), Some(*name));
    }
}

#[test]
fn parse_food_normalises_names() {
    assert_eq!(parse_food("chicken breast"), Some("CHICKEN_BREAST"));
    assert_eq!(parse_food("Food.egg"), Some("EGG"));
    assert_eq!(parse_food("  pork  "), Some("PORK"));
    assert_eq!(parse_food("pork-heart"), Some("PORK_HEART"));
    assert_eq!(parse_food("nope"), None);
}

#[test]
fn every_food_carries_data() {
    // `build.rs` refuses to emit a food whose source has no nutrient values,
    // so an all-`None` row would mean a hand-edited table.
    for name in FOOD_NAMES {
        let food = FOODS.get(name).expect("embedded food");
        assert!(*food != Food::default(), "{name} carries no data");
    }
}

#[test]
fn accessor_reads_the_named_fields() {
    // Guards the generated `Nutrient` -> field mapping: PORK's B1 and B2
    // differ by design, so a shifted column would show up here.
    let pork = FOODS.get("PORK").expect("PORK");
    assert_eq!(pork.nutrient(Nutrient::Energy), Some(6000.0));
    assert_eq!(pork.nutrient(Nutrient::Protein), Some(0.203));
    assert_eq!(pork.nutrient(Nutrient::VitaminB1), Some(5.4e-06));
    assert_eq!(pork.nutrient(Nutrient::VitaminB2), Some(1e-06));
    assert_eq!(pork.nutrient(Nutrient::Fiber), None);
}

#[test]
fn every_food_has_a_row_and_a_source() {
    // `FOODS` holds one chosen row per food, and every food names a source.
    for name in FOOD_NAMES {
        assert!(FOODS.contains_key(name), "{name} has no row");
        assert!(chosen_source(name).is_some(), "{name} has no chosen source");
    }
}

#[test]
fn choose_has_one_entry_per_food() {
    // `CHOOSE` covers every food exactly once and names only known sources.
    assert_eq!(
        foods::CHOOSE.len(),
        FOOD_NAMES.len(),
        "CHOOSE must have one entry per food"
    );
    for (name, source) in foods::CHOOSE {
        assert!(
            FOOD_NAMES.contains(name),
            "CHOOSE names unknown food {name}"
        );
        assert!(
            ["Usda", "Chinanutri", "SrLegacy", "Mext", "Inline"].contains(source),
            "CHOOSE sends {name} to unknown source {source}"
        );
    }
}
