//! Embedded food composition data.
//!
//! `build.rs` embeds every cached row — one per food per source — from
//! `foods/{FOOD}_{Source}.json` plus the three inline recipes. Callers name a
//! food with [`FoodName`]; [`FoodName::source`] gives the source the solver
//! uses, and [`FOODS`] returns the matching row. Values are the solver's
//! base units: grams per gram of food, with `ENERGY` in joules per gram.

use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

use serde::Serialize;

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
impl fmt::Display for FoodName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Serialise a `FoodName` as its canonical name string.
impl Serialize for FoodName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// `FoodSource` prints as its file-name tag.
impl fmt::Display for FoodSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Serialise a `FoodSource` as its tag string.
impl Serialize for FoodSource {
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
    (FoodName::BANANA, FoodSource::Usda),
    (FoodName::BARF, FoodSource::Inline),
    (FoodName::BASA_FISH, FoodSource::Chinanutri),
    (FoodName::BEEF, FoodSource::Usda),
    (FoodName::BEEN_SPROUT, FoodSource::Chinanutri),
    (FoodName::BELL_PEPER, FoodSource::Usda),
    (FoodName::BOCAI, FoodSource::Chinanutri),
    (FoodName::BOKCHOY, FoodSource::Usda),
    (FoodName::BROCCOLI, FoodSource::Usda),
    (FoodName::CABBAGE, FoodSource::Usda),
    (FoodName::CANOLA_OIL, FoodSource::Chinanutri),
    (FoodName::CARROT, FoodSource::Usda),
    (FoodName::CELERY, FoodSource::Usda),
    (FoodName::CHICKEN_BREAST, FoodSource::Usda),
    (FoodName::CHICKEN_GIZZARD, FoodSource::Chinanutri),
    (FoodName::CHICKEN_HEART, FoodSource::Chinanutri),
    (FoodName::CHICKEN_LIVER, FoodSource::Chinanutri),
    (FoodName::CHICKEN_THIGH, FoodSource::Usda),
    (FoodName::CHINESE_LETTUS, FoodSource::Chinanutri),
    (FoodName::CUCUMBER, FoodSource::Usda),
    (FoodName::DUCK_GIZZARD, FoodSource::Chinanutri),
    (FoodName::EGG, FoodSource::Usda),
    (FoodName::EGGPLANT, FoodSource::Usda),
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
    (FoodName::POTATO, FoodSource::Usda),
    (FoodName::PUMPKIN, FoodSource::Chinanutri),
    (FoodName::QINCAI, FoodSource::Chinanutri),
    (FoodName::RICE, FoodSource::Usda),
    (FoodName::SALT, FoodSource::Chinanutri),
    (FoodName::SHANYAO, FoodSource::Chinanutri),
    (FoodName::SHITAKE, FoodSource::Chinanutri),
    (FoodName::SIGUA, FoodSource::Chinanutri),
    (FoodName::SIJIDOU, FoodSource::Chinanutri),
    (FoodName::SOYBEAN_GREEN, FoodSource::Chinanutri),
    (FoodName::SOY_MILK, FoodSource::Usda),
    (FoodName::SWEET_POTATO, FoodSource::Usda),
    (FoodName::TOFU_FIRM, FoodSource::Chinanutri),
    (FoodName::TOFU_SOFT, FoodSource::Chinanutri),
    (FoodName::TOMATO, FoodSource::Chinanutri),
    (FoodName::WHITE_MUSHROOM, FoodSource::Chinanutri),
    (FoodName::WINTER_MELON, FoodSource::Chinanutri),
    (FoodName::ZIGANLAN, FoodSource::Chinanutri),
    (FoodName::ZUCCHINI, FoodSource::Usda),
];

/// The food table keyed by canonical name, holding each food's chosen row.
///
/// Built once from the generated `(food, source, row)` triples: every source is
/// embedded, and [`CHOOSE`] picks the one that fills the table.
pub static FOODS: LazyLock<HashMap<FoodName, Food>> = LazyLock::new(|| {
    ALL.iter()
        .map(|&food| {
            let source = food.source();
            let row = FOOD_ROWS
                .iter()
                .find(|(name, tag, _)| *name == food && *tag == source)
                .map(|(_, _, row)| *row)
                .expect("every food has a row for its chosen source");
            (food, patch(food, row))
        })
        .collect()
});

/// A row transform registered in [`PATCH`].
pub type FoodPatch = fn(Food) -> Food;

/// Fixes applied to a chosen row after it is loaded.
///
/// Some sources leave a nutrient out rather than recording a number, and the
/// report reads that as missing data. A patch replaces the ambiguity for the
/// fields it covers while keeping the values the source did record, so the
/// solver's ingredient is unaffected and only the report's completeness
/// changes.
pub const PATCH: &[(FoodName, FoodPatch)] = &[(FoodName::SALT, Food::read_missing_as_zero)];

/// Apply the patch registered for `food`, if any.
fn patch(food: FoodName, row: Food) -> Food {
    match PATCH.iter().find(|(name, _)| *name == food) {
        Some((_, apply)) => apply(row),
        None => row,
    }
}

impl Food {
    /// Read every field the source did not record as a measured `0`.
    ///
    /// The fields the source did record keep their values; the rest become
    /// `Some(0.0)`, so a later report does not flag the food as incomplete.
    pub fn read_missing_as_zero(self) -> Food {
        Food {
            energy: or_zero(self.energy),
            protein: or_zero(self.protein),
            fat: or_zero(self.fat),
            cholesterol: or_zero(self.cholesterol),
            ash: or_zero(self.ash),
            carb: or_zero(self.carb),
            fiber: or_zero(self.fiber),
            calcium: or_zero(self.calcium),
            phosphorus: or_zero(self.phosphorus),
            potassium: or_zero(self.potassium),
            sodium: or_zero(self.sodium),
            magnesium: or_zero(self.magnesium),
            iron: or_zero(self.iron),
            zinc: or_zero(self.zinc),
            selenium: or_zero(self.selenium),
            copper: or_zero(self.copper),
            manganese: or_zero(self.manganese),
            iodine: or_zero(self.iodine),
            chloride: or_zero(self.chloride),
            vitamin_a: or_zero(self.vitamin_a),
            vitamin_c: or_zero(self.vitamin_c),
            vitamin_d: or_zero(self.vitamin_d),
            vitamin_e: or_zero(self.vitamin_e),
            vitamin_k: or_zero(self.vitamin_k),
            vitamin_b1: or_zero(self.vitamin_b1),
            vitamin_b2: or_zero(self.vitamin_b2),
            vitamin_b5: or_zero(self.vitamin_b5),
            vitamin_b6: or_zero(self.vitamin_b6),
            vitamin_b7: or_zero(self.vitamin_b7),
            vitamin_b12: or_zero(self.vitamin_b12),
            niacin: or_zero(self.niacin),
            pantothenic_acid: or_zero(self.pantothenic_acid),
            folic_acid: or_zero(self.folic_acid),
            choline: or_zero(self.choline),
            carotene: or_zero(self.carotene),
            arginine: or_zero(self.arginine),
            histidine: or_zero(self.histidine),
            isoleucine: or_zero(self.isoleucine),
            leucine: or_zero(self.leucine),
            lysine: or_zero(self.lysine),
            methionine: or_zero(self.methionine),
            cystine: or_zero(self.cystine),
            phenylalanine: or_zero(self.phenylalanine),
            tyrosine: or_zero(self.tyrosine),
            threonine: or_zero(self.threonine),
            tryptophan: or_zero(self.tryptophan),
            valine: or_zero(self.valine),
            methionine_cystine: or_zero(self.methionine_cystine),
            phenylalanine_tyrosine: or_zero(self.phenylalanine_tyrosine),
            linoleic_acid: or_zero(self.linoleic_acid),
            alpha_linolenic_acid: or_zero(self.alpha_linolenic_acid),
            arachidonic_acid: or_zero(self.arachidonic_acid),
            epa: or_zero(self.epa),
            dha: or_zero(self.dha),
            epa_dha: or_zero(self.epa_dha),
        }
    }
}

/// `Some(value)` unchanged, `None` read as a measured zero.
fn or_zero(value: Option<f64>) -> Option<f64> {
    Some(value.unwrap_or(0.0))
}

impl FoodName {
    /// The source the solver uses for this food.
    ///
    /// Every food has exactly one entry in [`CHOOSE`].
    pub fn source(self) -> FoodSource {
        CHOOSE
            .iter()
            .find(|(name, _)| *name == self)
            .map(|(_, source)| *source)
            .expect("every food has a CHOOSE entry")
    }

    /// Parse a user-supplied food name into its canonical name.
    ///
    /// Accepts case-insensitive names, spaces or hyphens instead of
    /// underscores, and a leading `Food.` qualifier.
    pub fn parse(name: &str) -> Option<Self> {
        let trimmed = name.trim();
        let key = trimmed.rsplit('.').next().unwrap_or(trimmed);
        let normalized = key.trim().to_ascii_uppercase().replace([' ', '-'], "_");
        ALL.iter().copied().find(|food| food.name() == normalized)
    }
}
