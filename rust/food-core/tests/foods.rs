//! The embedded food table must stay complete and self-consistent.

use food_core::foods;
use food_core::{chosen_source, food_rows, parse_food, Food, Nutrient, FOODS, FOOD_NAMES};

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
fn every_food_resolves_to_its_chosen_row() {
    // `FOODS` holds one row per food, taken from the source `CHOOSE` names (or
    // the primary source when there is no entry), and every name must resolve.
    for name in FOOD_NAMES {
        assert!(FOODS.contains_key(name), "{name} has no chosen row");
        let source = chosen_source(name).unwrap_or_else(|| panic!("{name} has no source"));
        assert!(
            food_rows(name).iter().any(|(tag, _)| *tag == source),
            "{name} chose {source} but has no such row"
        );
    }
}

#[test]
fn choose_only_names_known_foods() {
    for (name, source) in foods::CHOOSE {
        assert!(
            FOOD_NAMES.contains(name),
            "CHOOSE names unknown food {name}"
        );
        assert!(
            food_rows(name).iter().any(|(tag, _)| tag == source),
            "CHOOSE sends {name} to {source}, which has no cached row"
        );
    }
}
