//! End-to-end solver checks on the reference batch.
//!
//! These numbers are the solver's current output for the reference batch,
//! recorded so a change in semantics or data shows up as a diff. They are not
//! an external ground truth:
//! the cache now picks one source per food (`choose` in the file metadata), and
//! `CELERY`, `RICE` and `EGG` were deliberately moved onto their richer SR
//! Legacy rows, so the optima shift. The expectations below are the solver's
//! current output for this batch; re-record them when a `choose` entry changes.

use food_core::{plan, FoodName, IngredientSpec, Nutrient, PlanRequest, Profile};

fn reference_request(days: u32) -> PlanRequest {
    PlanRequest {
        ingredients: vec![
            IngredientSpec::fixed(FoodName::CELERY, 245.0),
            IngredientSpec::fixed(FoodName::JIANGDOU, 411.0),
            IngredientSpec::fixed(FoodName::PORK, 500.0),
            IngredientSpec::minimize(FoodName::RICE, 1000.0 * f64::from(days)),
            IngredientSpec::minimize(FoodName::CANOLA_OIL, 5.0 * f64::from(days)),
            IngredientSpec::minimize(FoodName::SALT, 2.0 * f64::from(days)),
            IngredientSpec::minimize(FoodName::EGG_SHELL_POWDER, 5.0 * f64::from(days)),
            IngredientSpec::minimize(FoodName::EGG, 100.0 * f64::from(days)),
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
fn reference_batch_solves_to_a_known_optimum() {
    let result = plan(&reference_request(10)).expect("solve");
    assert!(result.optimal, "status: {}", result.status);

    // CELERY (fixed), JIANGDOU (fixed), PORK (fixed), then the optional
    // ingredients the solver picked: RICE, CANOLA_OIL, SALT,
    // EGG_SHELL_POWDER and EGG.
    let expected = [
        (FoodName::CELERY, 245.0),
        (FoodName::JIANGDOU, 411.0),
        (FoodName::PORK, 500.0),
        (FoodName::RICE, 583.0969403708767),
        (FoodName::CANOLA_OIL, 0.0),
        (FoodName::SALT, 0.0028061102421507926),
        (FoodName::EGG_SHELL_POWDER, 15.769841110678824),
        (FoodName::EGG, 894.7850273827171),
    ];
    for (food, grams) in expected {
        let actual = result
            .amount(food)
            .unwrap_or_else(|| panic!("{}", food.name()));
        assert_close(actual, grams, 0.5, food.name());
    }

    assert_close(
        result.objective.expect("objective"),
        2.0226827729798784,
        0.05,
        "objective",
    );

    let energy = result.nutrient(Nutrient::Energy).expect("energy");
    assert_eq!(energy.unit, "kJ");
    assert_close(energy.value, 1796.0900627751228, 0.05, "energy per day");
    assert_eq!(energy.status.as_str(), "within range");
}

#[test]
fn one_day_reference_batch_is_infeasible() {
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
        result.amount(FoodName::CELERY).expect("CELERY"),
        245.0,
        1e-6,
        "fixed weight",
    );
    let celery = result
        .recipe
        .iter()
        .find(|line| line.food == FoodName::CELERY)
        .unwrap();
    assert_close(celery.grams_per_day, 245.0 / 20.0, 1e-9, "per-day weight");
}

#[test]
fn nutrition_report_covers_every_requirement() {
    let result = plan(&reference_request(10)).expect("solve");
    let requirements = food_core::Profile::default().nutrient_needs().unwrap();
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

#[test]
fn report_scores_fixed_weights_without_solving() {
    // `report` takes every weight as exact and reports what that batch
    // supplies, so the recipe echoes the input rather than an optimum.
    let request = PlanRequest {
        ingredients: vec![
            IngredientSpec::fixed(FoodName::PORK, 500.0),
            IngredientSpec::fixed(FoodName::RICE, 700.0),
        ],
        days: 7,
        profile: Profile::default(),
        implicit_soft_upper_multiplier: Some(1.5),
        detail: false,
    };
    let result = food_core::report(&request).expect("report");

    assert_eq!(result.amount(FoodName::PORK), Some(500.0));
    assert_eq!(result.amount(FoodName::RICE), Some(700.0));
    assert_eq!(result.batch_grams, 1200.0);
    assert!(result.nutrition.len() > 1);
    // Per-day values divide the batch by the day count.
    let celery = result
        .recipe
        .iter()
        .find(|line| line.food == FoodName::RICE)
        .unwrap();
    assert!((celery.grams_per_day - 700.0 / 7.0).abs() < 1e-9);
}

#[test]
fn report_rejects_a_non_fixed_weight() {
    let request = PlanRequest {
        ingredients: vec![IngredientSpec::minimize(FoodName::RICE, 700.0)],
        days: 1,
        profile: Profile::default(),
        implicit_soft_upper_multiplier: Some(1.5),
        detail: false,
    };
    assert!(food_core::report(&request).is_err());
}
