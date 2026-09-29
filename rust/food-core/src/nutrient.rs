//! Nutrient identifiers and the combined-target lookup.

use serde::{Deserialize, Serialize};

use crate::foods::Food;

/// Every nutrient the solver understands, in canonical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Nutrient {
    Energy,
    Protein,
    Fat,
    Cholesterol,
    Ash,
    Carb,
    Fiber,

    Calcium,
    Phosphorus,
    Potassium,
    Sodium,
    Magnesium,
    Iron,
    Zinc,
    Selenium,
    Copper,
    Manganese,
    Iodine,
    Chloride,

    VitaminA,
    VitaminC,
    VitaminD,
    VitaminE,
    VitaminK,
    VitaminB1,
    VitaminB2,
    VitaminB5,
    VitaminB6,
    VitaminB7,
    VitaminB12,

    Niacin,
    PantothenicAcid,
    FolicAcid,
    Choline,
    Carotene,

    Arginine,
    Histidine,
    Isoleucine,
    Leucine,
    Lysine,
    Methionine,
    Cystine,
    Phenylalanine,
    Tyrosine,
    Threonine,
    Tryptophan,
    Valine,
    MethionineCystine,
    PhenylalanineTyrosine,

    LinoleicAcid,
    AlphaLinolenicAcid,
    ArachidonicAcid,
    Epa,
    Dha,
    EpaDha,
}

impl Nutrient {
    /// Uppercase `SCREAMING_SNAKE_CASE` name, matching the keys used in the
    /// food data and reports.
    pub fn name(self) -> &'static str {
        use Nutrient::*;
        match self {
            Energy => "ENERGY",
            Protein => "PROTEIN",
            Fat => "FAT",
            Cholesterol => "CHOLESTEROL",
            Ash => "ASH",
            Carb => "CARB",
            Fiber => "FIBER",
            Calcium => "CALCIUM",
            Phosphorus => "PHOSPHORUS",
            Potassium => "POTASSIUM",
            Sodium => "SODIUM",
            Magnesium => "MAGNESIUM",
            Iron => "IRON",
            Zinc => "ZINC",
            Selenium => "SELENIUM",
            Copper => "COPPER",
            Manganese => "MANGANESE",
            Iodine => "IODINE",
            Chloride => "CHLORIDE",
            VitaminA => "VITAMIN_A",
            VitaminC => "VITAMIN_C",
            VitaminD => "VITAMIN_D",
            VitaminE => "VITAMIN_E",
            VitaminK => "VITAMIN_K",
            VitaminB1 => "VITAMIN_B1",
            VitaminB2 => "VITAMIN_B2",
            VitaminB5 => "VITAMIN_B5",
            VitaminB6 => "VITAMIN_B6",
            VitaminB7 => "VITAMIN_B7",
            VitaminB12 => "VITAMIN_B12",
            Niacin => "NIACIN",
            PantothenicAcid => "PANTOTHENIC_ACID",
            FolicAcid => "FOLIC_ACID",
            Choline => "CHOLINE",
            Carotene => "CAROTENE",
            Arginine => "ARGININE",
            Histidine => "HISTIDINE",
            Isoleucine => "ISOLEUCINE",
            Leucine => "LEUCINE",
            Lysine => "LYSINE",
            Methionine => "METHIONINE",
            Cystine => "CYSTINE",
            Phenylalanine => "PHENYLALANINE",
            Tyrosine => "TYROSINE",
            Threonine => "THREONINE",
            Tryptophan => "TRYPTOPHAN",
            Valine => "VALINE",
            MethionineCystine => "METHIONINE_CYSTINE",
            PhenylalanineTyrosine => "PHENYLALANINE_TYROSINE",
            LinoleicAcid => "LINOLEIC_ACID",
            AlphaLinolenicAcid => "ALPHA_LINOLENIC_ACID",
            ArachidonicAcid => "ARACHIDONIC_ACID",
            Epa => "EPA",
            Dha => "DHA",
            EpaDha => "EPA_DHA",
        }
    }

    /// Every nutrient, in the order the reports use.
    pub fn all() -> &'static [Nutrient] {
        use Nutrient::*;
        &[
            Energy,
            Protein,
            Fat,
            Cholesterol,
            Ash,
            Carb,
            Fiber,
            Calcium,
            Phosphorus,
            Potassium,
            Sodium,
            Magnesium,
            Iron,
            Zinc,
            Selenium,
            Copper,
            Manganese,
            Iodine,
            Chloride,
            VitaminA,
            VitaminC,
            VitaminD,
            VitaminE,
            VitaminK,
            VitaminB1,
            VitaminB2,
            VitaminB5,
            VitaminB6,
            VitaminB7,
            VitaminB12,
            Niacin,
            PantothenicAcid,
            FolicAcid,
            Choline,
            Carotene,
            Arginine,
            Histidine,
            Isoleucine,
            Leucine,
            Lysine,
            Methionine,
            Cystine,
            Phenylalanine,
            Tyrosine,
            Threonine,
            Tryptophan,
            Valine,
            MethionineCystine,
            PhenylalanineTyrosine,
            LinoleicAcid,
            AlphaLinolenicAcid,
            ArachidonicAcid,
            Epa,
            Dha,
            EpaDha,
        ]
    }
}

/// Combined targets are sums of their components, not replacements for them.
fn combined_parts(nutrient: Nutrient) -> Option<&'static [Nutrient]> {
    use Nutrient::*;
    match nutrient {
        MethionineCystine => Some(&[Methionine, Cystine]),
        PhenylalanineTyrosine => Some(&[Phenylalanine, Tyrosine]),
        EpaDha => Some(&[Epa, Dha]),
        _ => None,
    }
}

/// Contribution of one food to one nutrient, plus whether the food supplies
/// complete data for it.
///
/// A combined target falls back to the known partial sum and stays incomplete.
/// The legacy `B5` name is a fallback for a missing `VITAMIN_B5`, never an
/// extra contributor. `None` means the food has no data for this nutrient.
pub fn nutrient_value(food: &Food, nutrient: Nutrient) -> (Option<f64>, bool) {
    if let Some(value) = food.nutrient(nutrient) {
        return (Some(value), true);
    }
    if nutrient == Nutrient::VitaminB5 {
        if let Some(value) = food.nutrient(Nutrient::PantothenicAcid) {
            return (Some(value), true);
        }
    }
    if let Some(parts) = combined_parts(nutrient) {
        // Mirror the Python behaviour: a combined target always contributes
        // its partial sum and is only "known" when every component is known.
        let mut total = 0.0;
        let mut complete = true;
        for part in parts {
            let (value, known) = nutrient_value(food, *part);
            total += value.unwrap_or(0.0);
            complete &= known;
        }
        return (Some(total), complete);
    }
    (None, false)
}
