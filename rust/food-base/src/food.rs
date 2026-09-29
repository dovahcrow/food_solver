//! The nutrient composition of one food.
//!
//! Amounts are grams per gram of food and energy is joules per gram, the
//! solver's base units. `None` means the source has no value for that
//! nutrient, which the report surfaces as incomplete data; it is never the
//! same as a measured zero.

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
