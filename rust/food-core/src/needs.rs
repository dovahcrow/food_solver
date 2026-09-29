//! FEDIAF 2025 Table III-3b adult dog maintenance requirements.
//!
//! <https://europeanpetfood.org/wp-content/uploads/2025/09/FEDIAF-Nutritional-Guidelines_2025-ONLINE.pdf>
//!
//! Values are per 1000 kcal metabolisable energy. Masses are grams and energy
//! is joules. This is not a complete nutritional adequacy check: food data may
//! be incomplete. Maxima are the table's nutritional (N) values, not the EU
//! legal (L) dry-matter limits, which cannot be applied without dry matter.

use std::collections::BTreeMap;

use anyhow::{anyhow, Error};
use culpa::{throw, throws};

use crate::nutrient::Nutrient;
use crate::recipe::{NeedRequired, NeedSoftness, Requirement};
use crate::units::{G, KCAL, MCG, MG, VITAMIN_A_IU, VITAMIN_D_IU, VITAMIN_E_IU};

/// Dog profile selecting the requirement table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Profile {
    pub age: f64,
    pub weight: f64,
    pub active: bool,
    pub daily_kcal: Option<f64>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            age: 3.0,
            weight: 7.0,
            active: false,
            daily_kcal: None,
        }
    }
}

impl Profile {
    /// Adult maintenance estimates; weight in kg and age in years.
    ///
    /// `active = false` selects the 95 kcal/kg^0.75 reference profile, `true`
    /// the 110 profile (ordinary activity, not working dogs). `daily_kcal`
    /// overrides the energy estimate while `active` still selects the nutrient
    /// profile. Age is a scope check, not an invented age correction: growing,
    /// gestating and lactating dogs need another profile.
    #[throws(Error)]
    pub fn nutrient_needs(&self) -> BTreeMap<Nutrient, Requirement> {
        // The body reads the profile by value; `Profile` is `Copy`.
        let profile = *self;
        if !profile.age.is_finite() || profile.age < 1.0 {
            throw!(anyhow!(
                "Only adult maintenance is supported (age >= 1 year)"
            ));
        }
        if !profile.weight.is_finite() || profile.weight <= 0.0 {
            throw!(anyhow!("weight must be a positive finite number in kg"));
        }
        let daily_kcal = match profile.daily_kcal {
            Some(kcal) => kcal,
            None => (if profile.active { 110.0 } else { 95.0 }) * profile.weight.powf(0.75),
        };
        if !daily_kcal.is_finite() || daily_kcal <= 0.0 {
            throw!(anyhow!("daily_kcal must be positive and finite"));
        }
        let energy = daily_kcal * KCAL;
        let modulation = daily_kcal / 1000.0;

        let minimum = |low_activity: f64, ordinary_activity: f64| {
            if profile.active {
                ordinary_activity
            } else {
                low_activity
            }
        };

        let soft = |minimum: f64, maximum: Option<f64>| Requirement {
            minimum,
            maximum,
            required: NeedRequired::Required,
            softness: NeedSoftness::Soft,
        };

        // A/D/E, choline and selenium keep their historical NOT_REQUIRED setting:
        // they are reported but not optimised.
        let soft_optional = |minimum: f64, maximum: Option<f64>| Requirement {
            minimum,
            maximum,
            required: NeedRequired::NotRequired,
            softness: NeedSoftness::Soft,
        };

        let hard = |minimum: f64, maximum: Option<f64>| Requirement {
            minimum,
            maximum,
            required: NeedRequired::Required,
            softness: NeedSoftness::Hard,
        };

        let mut needs = BTreeMap::new();

        // Additional adult minima, all SOFT because food composition data may be
        // incomplete. Individual methionine and phenylalanine minima apply
        // alongside their combined targets. Adult ALA, arachidonic acid, EPA+DHA,
        // biotin and vitamin K have no numeric minimum here, so no zero targets
        // are invented.
        let g = |low: f64, ordinary: f64| minimum(low, ordinary) * G * modulation;
        let mg = |low: f64, ordinary: f64| minimum(low, ordinary) * MG * modulation;
        let mcg = |low: f64, ordinary: f64| minimum(low, ordinary) * MCG * modulation;

        needs.insert(Nutrient::Fat, soft(g(13.75, 13.75), None));
        needs.insert(Nutrient::LinoleicAcid, soft_optional(g(3.82, 3.27), None));
        needs.insert(Nutrient::Arginine, soft_optional(g(1.51, 1.30), None));
        needs.insert(Nutrient::Histidine, soft_optional(g(0.67, 0.58), None));
        needs.insert(Nutrient::Isoleucine, soft_optional(g(1.33, 1.15), None));
        needs.insert(Nutrient::Leucine, soft_optional(g(2.37, 2.05), None));
        needs.insert(Nutrient::Lysine, soft_optional(g(1.22, 1.05), None));
        needs.insert(Nutrient::Methionine, soft_optional(g(1.16, 1.00), None));
        needs.insert(
            Nutrient::MethionineCystine,
            soft_optional(g(2.21, 1.91), None),
        );
        needs.insert(Nutrient::Phenylalanine, soft_optional(g(1.56, 1.35), None));
        needs.insert(
            Nutrient::PhenylalanineTyrosine,
            soft_optional(g(2.58, 2.23), None),
        );
        needs.insert(Nutrient::Threonine, soft_optional(g(1.51, 1.30), None));
        needs.insert(Nutrient::Tryptophan, soft_optional(g(0.49, 0.43), None));
        needs.insert(Nutrient::Valine, soft_optional(g(1.71, 1.48), None));
        needs.insert(Nutrient::VitaminB5, soft(mg(4.11, 3.55), None));
        needs.insert(Nutrient::FolicAcid, soft(mcg(74.70, 64.50), None));
        needs.insert(Nutrient::Chloride, soft_optional(g(0.43, 0.38), None));

        needs.insert(
            Nutrient::Energy,
            Requirement {
                minimum: energy * 0.9,
                maximum: Some(energy * 1.05),
                required: NeedRequired::Required,
                softness: NeedSoftness::Hard,
            },
        );
        needs.insert(
            Nutrient::Protein,
            Requirement {
                minimum: minimum(52.10, 45.00) * G * modulation,
                maximum: None,
                required: NeedRequired::Required,
                softness: NeedSoftness::Hard,
            },
        );

        let vitamin_a = |value: f64| value * VITAMIN_A_IU * modulation;
        needs.insert(
            Nutrient::VitaminA,
            soft(vitamin_a(1754.0), Some(vitamin_a(100_000.0))),
        );
        needs.insert(Nutrient::VitaminB1, soft(mg(0.62, 0.54), None));
        needs.insert(Nutrient::VitaminB2, soft(mg(1.74, 1.50), None));
        needs.insert(Nutrient::VitaminB6, soft(mg(0.42, 0.36), None));
        needs.insert(Nutrient::VitaminB12, soft(mcg(9.68, 8.36), None));
        needs.insert(
            Nutrient::VitaminE,
            soft_optional(10.40 * VITAMIN_E_IU * modulation, None),
        );
        let vitamin_d = |value: f64| value * VITAMIN_D_IU * modulation;
        needs.insert(
            Nutrient::VitaminD,
            soft(vitamin_d(159.0), Some(vitamin_d(800.0))),
        );

        needs.insert(
            Nutrient::Calcium,
            hard(
                minimum(1.45, 1.25) * G * modulation,
                Some(6.25 * G * modulation),
            ),
        );
        needs.insert(Nutrient::Copper, soft(mg(2.08, 1.80), None));
        needs.insert(Nutrient::Iodine, soft_optional(mg(0.30, 0.26), None));
        needs.insert(Nutrient::Zinc, soft(mg(20.80, 18.00), None));
        needs.insert(Nutrient::Choline, soft_optional(mg(474.0, 409.0), None));
        needs.insert(Nutrient::Iron, soft(mg(10.40, 9.00), None));
        needs.insert(Nutrient::Selenium, soft_optional(mcg(67.50, 57.50), None));
        needs.insert(
            Nutrient::Phosphorus,
            soft(g(1.16, 1.00), Some(4.0 * G * modulation)),
        );
        needs.insert(Nutrient::Sodium, hard(g(0.29, 0.25), None));
        needs.insert(Nutrient::Potassium, soft(g(1.45, 1.25), None));
        needs.insert(Nutrient::Manganese, soft(mg(1.67, 1.44), None));
        needs.insert(Nutrient::Magnesium, soft(g(0.20, 0.18), None));
        needs.insert(Nutrient::Niacin, soft(mg(4.74, 4.09), None));

        needs
    }
}

/// Scale a requirement table by a whole-day count.
#[throws(Error)]
pub fn scale(
    needs: &BTreeMap<Nutrient, Requirement>,
    days: u32,
) -> BTreeMap<Nutrient, Requirement> {
    if days == 0 {
        throw!(anyhow!("day must be a positive integer"));
    }
    let factor = days as f64;
    needs
        .iter()
        .map(|(nutrient, need)| {
            (
                *nutrient,
                Requirement {
                    minimum: need.minimum * factor,
                    maximum: need.maximum.map(|value| value * factor),
                    ..*need
                },
            )
        })
        .collect()
}
