//! Compare the Rust solver against the Python reference.
//!
//! The expectations below were produced by running the equivalent solve with
//! the Python frontends on the same ingredient bounds and requirements.

use food_core::{plan, IngredientSpec, Nutrient, PlanRequest, Profile};

fn reference_request(days: u32) -> PlanRequest {
    PlanRequest {
        ingredients: vec![
            IngredientSpec::fixed("CELERY", 245.0),
            IngredientSpec::fixed("JIANGDOU", 411.0),
            IngredientSpec::fixed("PORK", 500.0),
            IngredientSpec::minimize("RICE", 1000.0 * f64::from(days)),
            IngredientSpec::minimize("CANOLA_OIL", 5.0 * f64::from(days)),
            IngredientSpec::minimize("SALT", 2.0 * f64::from(days)),
            IngredientSpec::minimize("EGG_SHELL_POWDER", 5.0 * f64::from(days)),
            IngredientSpec::minimize("EGG", 100.0 * f64::from(days)),
        ],
        days,
        profile: Profile {
            age: 3.0,
            weight: 7.0,
            active: false,
            daily_kcal: None,
        },
        implicit_soft_upper_multiplier: Some(1.5),
        detail: false,
    }
}

/// Compare two optima, allowing for interior-point tolerance differences.
fn assert_close(actual: f64, expected: f64, tolerance: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{label}: expected {expected}, got {actual}"
    );
}

#[test]
fn reference_batch_matches_the_python_solver() {
    let result = plan(&reference_request(10)).expect("solve");
    assert!(result.optimal, "status: {}", result.status);

    // Python: CELERY 245.0, JIANGDOU 411.0, PORK 500.00000000000006,
    // RICE 564.1651680898664, CANOLA_OIL 1.6767e-08, SALT 9.8374e-07,
    // EGG_SHELL_POWDER 15.386216432485288, EGG 961.6387897502321.
    let expected = [
        ("CELERY", 245.0),
        ("JIANGDOU", 411.0),
        ("PORK", 500.0),
        ("RICE", 564.1651680898664),
        ("CANOLA_OIL", 0.0),
        ("SALT", 0.0),
        ("EGG_SHELL_POWDER", 15.386216432485288),
        ("EGG", 961.6387897502321),
    ];
    for (food, grams) in expected {
        assert_close(result.amount(food).expect(food), grams, 0.5, food);
    }

    // Python's objective for this batch.
    assert_close(
        result.objective.expect("objective"),
        6.292734671354526,
        0.05,
        "objective",
    );

    let energy = result.nutrient(Nutrient::Energy).expect("energy");
    assert_eq!(energy.unit, "kJ");
    assert_close(energy.value, 1796.09, 0.05, "energy per day");
    assert_eq!(energy.status.as_str(), "within range");
}

#[test]
fn one_day_reference_batch_is_infeasible_like_python() {
    let result = plan(&reference_request(1)).expect("solve");
    assert!(!result.optimal);
    assert_eq!(result.status, "infeasible");
    assert!(result.recipe.is_empty());
    assert_eq!(result.attempted.len(), 8);
}

#[test]
fn days_scale_requirements_but_not_fixed_weights() {
    let result = plan(&reference_request(20)).expect("solve");
    assert!(result.optimal, "status: {}", result.status);
    assert_close(
        result.amount("CELERY").expect("CELERY"),
        245.0,
        1e-6,
        "fixed weight",
    );
    let celery = result
        .recipe
        .iter()
        .find(|line| line.food == "CELERY")
        .unwrap();
    assert_close(celery.grams_per_day, 245.0 / 20.0, 1e-9, "per-day weight");
}

#[test]
fn nutrition_report_covers_every_requirement() {
    let result = plan(&reference_request(10)).expect("solve");
    let requirements = food_core::needs::dog_needs(food_core::Profile::default()).unwrap();
    // The report has one line per requirement, in the nutrient enum's order.
    assert_eq!(result.nutrition.len(), requirements.len());
    let names: Vec<&str> = result.nutrition.iter().map(|r| r.nutrient.name()).collect();
    assert_eq!(names.first(), Some(&"ENERGY"));
    // Per-day values divide the batch total by the day count.
    let protein = result.nutrient(Nutrient::Protein).unwrap();
    assert!(protein.minimum > 0.0);
}

#[test]
fn combined_amino_targets_are_reported() {
    let result = plan(&reference_request(10)).expect("solve");
    for nutrient in [Nutrient::MethionineCystine, Nutrient::PhenylalanineTyrosine] {
        let report = result
            .nutrient(nutrient)
            .unwrap_or_else(|| panic!("{}", nutrient.name()));
        assert!(report.minimum > 0.0);
    }
}
