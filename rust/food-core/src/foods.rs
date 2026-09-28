//! Embedded food composition data.
//!
//! `build.rs` embeds every cached row — one per food per source — from
//! `foods/{FOOD}_{Source}.json` plus the three inline recipes. Callers name a
//! food with [`FoodName`]; [`chosen_source`] maps it to the source the solver
//! should use, and [`FOODS`] returns the matching row. Values are the solver's
//! base units: grams per gram of food, with `ENERGY` in joules per gram.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::nutrient::Nutrient;

/// What one food contributes, one field per nutrient.
///
/// Amounts are grams per gram of food and energy is joules per gram, the
/// solver's base units. `None` means the source has no value for that
/// nutrient, which the report surfaces as incomplete data; it is never the
/// same as a measured zero.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Food {
    pub energy: Option<f64>,
    pub protein: Option<f64>,
    pub fat: Option<f64>,
    pub cholesterol: Option<f64>,
    pub ash: Option<f64>,
    pub carb: Option<f64>,
    pub fiber: Option<f64>,
    pub calcium: Option<f64>,
    pub phosphorus: Option<f64>,
    pub potassium: Option<f64>,
    pub sodium: Option<f64>,
    pub magnesium: Option<f64>,
    pub iron: Option<f64>,
    pub zinc: Option<f64>,
    pub selenium: Option<f64>,
    pub copper: Option<f64>,
    pub manganese: Option<f64>,
    pub iodine: Option<f64>,
    pub chloride: Option<f64>,
    pub vitamin_a: Option<f64>,
    pub vitamin_c: Option<f64>,
    pub vitamin_d: Option<f64>,
    pub vitamin_e: Option<f64>,
    pub vitamin_k: Option<f64>,
    pub vitamin_b1: Option<f64>,
    pub vitamin_b2: Option<f64>,
    pub vitamin_b5: Option<f64>,
    pub vitamin_b6: Option<f64>,
    pub vitamin_b7: Option<f64>,
    pub vitamin_b12: Option<f64>,
    pub niacin: Option<f64>,
    pub pantothenic_acid: Option<f64>,
    pub folic_acid: Option<f64>,
    pub choline: Option<f64>,
    pub carotene: Option<f64>,
    pub arginine: Option<f64>,
    pub histidine: Option<f64>,
    pub isoleucine: Option<f64>,
    pub leucine: Option<f64>,
    pub lysine: Option<f64>,
    pub methionine: Option<f64>,
    pub cystine: Option<f64>,
    pub phenylalanine: Option<f64>,
    pub tyrosine: Option<f64>,
    pub threonine: Option<f64>,
    pub tryptophan: Option<f64>,
    pub valine: Option<f64>,
    pub methionine_cystine: Option<f64>,
    pub phenylalanine_tyrosine: Option<f64>,
    pub linoleic_acid: Option<f64>,
    pub alpha_linolenic_acid: Option<f64>,
    pub arachidonic_acid: Option<f64>,
    pub epa: Option<f64>,
    pub dha: Option<f64>,
    pub epa_dha: Option<f64>,
}

impl Food {
    /// The value recorded for one nutrient, or `None` when the source has
    /// no value for it.
    pub fn nutrient(&self, nutrient: Nutrient) -> Option<f64> {
        use Nutrient::*;
        match nutrient {
            Energy => self.energy,
            Protein => self.protein,
            Fat => self.fat,
            Cholesterol => self.cholesterol,
            Ash => self.ash,
            Carb => self.carb,
            Fiber => self.fiber,
            Calcium => self.calcium,
            Phosphorus => self.phosphorus,
            Potassium => self.potassium,
            Sodium => self.sodium,
            Magnesium => self.magnesium,
            Iron => self.iron,
            Zinc => self.zinc,
            Selenium => self.selenium,
            Copper => self.copper,
            Manganese => self.manganese,
            Iodine => self.iodine,
            Chloride => self.chloride,
            VitaminA => self.vitamin_a,
            VitaminC => self.vitamin_c,
            VitaminD => self.vitamin_d,
            VitaminE => self.vitamin_e,
            VitaminK => self.vitamin_k,
            VitaminB1 => self.vitamin_b1,
            VitaminB2 => self.vitamin_b2,
            VitaminB5 => self.vitamin_b5,
            VitaminB6 => self.vitamin_b6,
            VitaminB7 => self.vitamin_b7,
            VitaminB12 => self.vitamin_b12,
            Niacin => self.niacin,
            PantothenicAcid => self.pantothenic_acid,
            FolicAcid => self.folic_acid,
            Choline => self.choline,
            Carotene => self.carotene,
            Arginine => self.arginine,
            Histidine => self.histidine,
            Isoleucine => self.isoleucine,
            Leucine => self.leucine,
            Lysine => self.lysine,
            Methionine => self.methionine,
            Cystine => self.cystine,
            Phenylalanine => self.phenylalanine,
            Tyrosine => self.tyrosine,
            Threonine => self.threonine,
            Tryptophan => self.tryptophan,
            Valine => self.valine,
            MethionineCystine => self.methionine_cystine,
            PhenylalanineTyrosine => self.phenylalanine_tyrosine,
            LinoleicAcid => self.linoleic_acid,
            AlphaLinolenicAcid => self.alpha_linolenic_acid,
            ArachidonicAcid => self.arachidonic_acid,
            Epa => self.epa,
            Dha => self.dha,
            EpaDha => self.epa_dha,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/foods_data.rs"));

/// `FoodName` prints as its canonical name, so reports and JSON stay readable.
impl std::fmt::Display for FoodName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Serialise a `FoodName` as its canonical name string.
impl serde::Serialize for FoodName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// `FoodSource` prints as its file-name tag.
impl std::fmt::Display for FoodSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Serialise a `FoodSource` as its tag string.
impl serde::Serialize for FoodSource {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// Which source each food uses, hardcoded here.
///
/// One entry per food, so the choice lives in exactly one place. `build.rs`
/// embeds every source's row and this table decides which one a run reads.
pub const CHOOSE: &[(FoodName, FoodSource)] = &[
    (FoodName::BAICAI, FoodSource::Chinanutri),
    (FoodName::BALANCEIT, FoodSource::Inline),
    (FoodName::BANANA, FoodSource::SrLegacy),
    (FoodName::BARF, FoodSource::Inline),
    (FoodName::BASA_FISH, FoodSource::Chinanutri),
    (FoodName::BEEF, FoodSource::SrLegacy),
    (FoodName::BEEN_SPROUT, FoodSource::Chinanutri),
    (FoodName::BELL_PEPER, FoodSource::SrLegacy),
    (FoodName::BOCAI, FoodSource::Chinanutri),
    (FoodName::BOKCHOY, FoodSource::SrLegacy),
    (FoodName::BROCCOLI, FoodSource::SrLegacy),
    (FoodName::CABBAGE, FoodSource::SrLegacy),
    (FoodName::CANOLA_OIL, FoodSource::Chinanutri),
    (FoodName::CARROT, FoodSource::SrLegacy),
    (FoodName::CELERY, FoodSource::SrLegacy),
    (FoodName::CHICKEN_BREAST, FoodSource::SrLegacy),
    (FoodName::CHICKEN_GIZZARD, FoodSource::Chinanutri),
    (FoodName::CHICKEN_HEART, FoodSource::Chinanutri),
    (FoodName::CHICKEN_LIVER, FoodSource::Chinanutri),
    (FoodName::CHICKEN_THIGH, FoodSource::SrLegacy),
    (FoodName::CHINESE_LETTUS, FoodSource::Chinanutri),
    (FoodName::CUCUMBER, FoodSource::SrLegacy),
    (FoodName::DUCK_GIZZARD, FoodSource::Chinanutri),
    (FoodName::EGG, FoodSource::SrLegacy),
    (FoodName::EGGPLANT, FoodSource::SrLegacy),
    (FoodName::EGG_SHELL_POWDER, FoodSource::Inline),
    (FoodName::FUGUA, FoodSource::Chinanutri),
    (FoodName::JIANGDOU, FoodSource::Chinanutri),
    (FoodName::JIEGUA, FoodSource::Chinanutri),
    (FoodName::JUANXINCAI, FoodSource::Chinanutri),
    (FoodName::KONGXINCAI, FoodSource::Chinanutri),
    (FoodName::KUIGUA, FoodSource::Chinanutri),
    (FoodName::LUOBO, FoodSource::Chinanutri),
    (FoodName::OYSTER, FoodSource::Chinanutri),
    (FoodName::PORK, FoodSource::Chinanutri),
    (FoodName::PORK_FAT, FoodSource::Chinanutri),
    (FoodName::PORK_HEART, FoodSource::Chinanutri),
    (FoodName::PORK_INTESTINE, FoodSource::Chinanutri),
    (FoodName::PORK_LIVER, FoodSource::Chinanutri),
    (FoodName::PORK_TONGUE, FoodSource::Chinanutri),
    (FoodName::POTATO, FoodSource::SrLegacy),
    (FoodName::PUMPKIN, FoodSource::Chinanutri),
    (FoodName::QINCAI, FoodSource::Chinanutri),
    (FoodName::RICE, FoodSource::SrLegacy),
    (FoodName::SALT, FoodSource::Chinanutri),
    (FoodName::SHANYAO, FoodSource::Chinanutri),
    (FoodName::SHITAKE, FoodSource::Chinanutri),
    (FoodName::SIGUA, FoodSource::Chinanutri),
    (FoodName::SIJIDOU, FoodSource::Chinanutri),
    (FoodName::SOYBEAN_GREEN, FoodSource::Chinanutri),
    (FoodName::SOY_MILK, FoodSource::SrLegacy),
    (FoodName::SWEET_POTATO, FoodSource::SrLegacy),
    (FoodName::TOFU_FIRM, FoodSource::Chinanutri),
    (FoodName::TOFU_SOFT, FoodSource::Chinanutri),
    (FoodName::TOMATO, FoodSource::Chinanutri),
    (FoodName::WHITE_MUSHROOM, FoodSource::Chinanutri),
    (FoodName::WINTER_MELON, FoodSource::Chinanutri),
    (FoodName::ZIGANLAN, FoodSource::Chinanutri),
    (FoodName::ZUCCHINI, FoodSource::SrLegacy),
];

/// The source the solver uses for a food.
pub fn chosen_source(food: FoodName) -> FoodSource {
    CHOOSE
        .iter()
        .find(|(name, _)| *name == food)
        .map(|(_, source)| *source)
        .expect("every food has a CHOOSE entry")
}

/// The food table keyed by canonical name, holding each food's chosen row.
///
/// Built once from the generated `(food, source, row)` triples: every source is
/// embedded, and [`CHOOSE`] picks the one that fills the table.
pub static FOODS: LazyLock<HashMap<FoodName, Food>> = LazyLock::new(|| {
    ALL.iter()
        .map(|&food| {
            let source = chosen_source(food);
            let row = FOOD_ROWS
                .iter()
                .find(|(name, tag, _)| *name == food && *tag == source)
                .map(|(_, _, row)| *row)
                .expect("every food has a row for its chosen source");
            (food, row)
        })
        .collect()
});

/// Parse a user-supplied food name into its canonical name.
///
/// Accepts case-insensitive names, spaces or hyphens instead of
/// underscores, and a leading `Food.` qualifier.
pub fn parse_food(name: &str) -> Option<FoodName> {
    let trimmed = name.trim();
    let key = trimmed.rsplit('.').next().unwrap_or(trimmed);
    let normalized = key.trim().to_ascii_uppercase().replace([' ', '-'], "_");
    ALL.iter().copied().find(|food| food.name() == normalized)
}
