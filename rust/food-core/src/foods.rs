//! Embedded food composition data.
//!
//! `build.rs` embeds every cached row — one per food per source — from
//! `foods/{FOOD}_{Source}.json` plus the four inline recipes. Callers name a
//! food with [`FoodName`]; [`FoodName::source`] gives the source the solver
//! uses, and [`FOODS`] returns the matching row. Values are the solver's
//! base units: grams per gram of food, with `ENERGY` in joules per gram.
//!
//! [`Food`] and the nutrient vocabulary live in `food-base`, so `build.rs`
//! can name the fields directly.

use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

use serde::Serialize;

use food_base::Food;

pub use food_base::{nutrient_value, Nutrient};

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
    (FoodName::BANANA, FoodSource::SrLegacy),
    (FoodName::BARF, FoodSource::Inline),
    (FoodName::BASA_FISH, FoodSource::SrLegacy),
    (FoodName::BEEF, FoodSource::SrLegacy),
    (FoodName::BEEF_LIVER, FoodSource::SrLegacy),
    (FoodName::BEEN_SPROUT, FoodSource::Chinanutri),
    (FoodName::BELL_PEPER, FoodSource::SrLegacy),
    (FoodName::BOCAI, FoodSource::Chinanutri),
    (FoodName::BOKCHOY, FoodSource::SrLegacy),
    (FoodName::BONE_MEAL, FoodSource::Inline),
    (FoodName::BROCCOLI, FoodSource::SrLegacy),
    (FoodName::CABBAGE, FoodSource::SrLegacy),
    (FoodName::CANOLA_OIL, FoodSource::Chinanutri),
    (FoodName::CARROT, FoodSource::SrLegacy),
    (FoodName::CELERY, FoodSource::SrLegacy),
    (FoodName::CHICKEN_BREAST, FoodSource::SrLegacy),
    (FoodName::CHICKEN_GIZZARD, FoodSource::SrLegacy),
    (FoodName::CHICKEN_HEART, FoodSource::SrLegacy),
    (FoodName::CHICKEN_LIVER, FoodSource::SrLegacy),
    (FoodName::CHICKEN_THIGH, FoodSource::SrLegacy),
    (FoodName::CHINESE_LETTUS, FoodSource::Chinanutri),
    (FoodName::CUCUMBER, FoodSource::SrLegacy),
    (FoodName::DUCK_GIZZARD, FoodSource::Chinanutri),
    (FoodName::EGG, FoodSource::SrLegacy),
    (FoodName::EGGPLANT, FoodSource::SrLegacy),
    (FoodName::EGG_SHELL_POWDER, FoodSource::Inline),
    (FoodName::EGG_YOLK, FoodSource::SrLegacy),
    (FoodName::FUGUA, FoodSource::Chinanutri),
    (FoodName::JIANGDOU, FoodSource::Chinanutri),
    (FoodName::JIEGUA, FoodSource::Chinanutri),
    (FoodName::JUANXINCAI, FoodSource::Chinanutri),
    (FoodName::KONGXINCAI, FoodSource::Chinanutri),
    (FoodName::CHAYOTE, FoodSource::Chinanutri),
    (FoodName::LUOBO, FoodSource::Chinanutri),
    (FoodName::OYSTER, FoodSource::Chinanutri),
    (FoodName::PORK, FoodSource::Chinanutri),
    (FoodName::PORK_FAT, FoodSource::Chinanutri),
    (FoodName::PORK_HEART, FoodSource::SrLegacy),
    (FoodName::PORK_INTESTINE, FoodSource::SrLegacy),
    (FoodName::PORK_LIVER, FoodSource::SrLegacy),
    (FoodName::PORK_TONGUE, FoodSource::SrLegacy),
    (FoodName::POTATO, FoodSource::SrLegacy),
    (FoodName::PUMPKIN, FoodSource::Chinanutri),
    (FoodName::QINCAI, FoodSource::Chinanutri),
    (FoodName::RICE, FoodSource::SrLegacy),
    (FoodName::SALT, FoodSource::Inline),
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

/// The food table keyed by canonical name, holding each food's chosen row.
///
/// Built once from the generated `(food, source, row)` triples: every source is
/// embedded, and [`CHOOSE`] picks the single row the solver reads. Rows are
/// never merged, so a source with a fuller nutrient panel replaces the others
/// rather than adding to them.
pub static FOODS: LazyLock<HashMap<FoodName, Food>> = LazyLock::new(|| {
    ALL.iter()
        .map(|&food| {
            let source = food.source();
            let row = FOOD_ROWS
                .iter()
                .find(|(name, tag, _)| *name == food && *tag == source)
                .map(|(_, _, row)| *row)
                .expect("every food has a row for its chosen source");
            (food, row)
        })
        .collect()
});

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
