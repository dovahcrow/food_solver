//! `NUTRIENT_FIELDS` is the build script's source of truth for the generated
//! table, so it must stay aligned with the `Nutrient` enum.

use food_base::{Nutrient, NUTRIENT_FIELDS};

#[test]
fn field_names_match_the_enum_one_for_one_in_order() {
    assert_eq!(
        NUTRIENT_FIELDS.len(),
        Nutrient::all().len(),
        "NUTRIENT_FIELDS and Nutrient::all() must have the same length"
    );
    for (field, nutrient) in NUTRIENT_FIELDS.iter().zip(Nutrient::all()) {
        assert!(
            field.eq_ignore_ascii_case(nutrient.name()),
            "{field:?} does not name {}",
            nutrient.name()
        );
    }
}

#[test]
fn field_names_are_unique() {
    let mut seen = NUTRIENT_FIELDS.to_vec();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), NUTRIENT_FIELDS.len(), "duplicate field name");
}
