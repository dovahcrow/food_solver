//! Source-name -> solver-key tables, ported verbatim from the Python
//! getters in `src/food_getters/{usda,chinanutri}.py`.
//!
//! A `None` key marks a row the Python getter deliberately ignores (a
//! total, a duplicate vitamin form, or a nutrient the solver does not
//! model). Keeping those rows explicit preserves the port's parity.

/// USDA FoodData Central display name -> `Nutrient` key, or `None`.
pub const USDA_NAMES: &[(&str, Option<&str>)] = &[
    ("PUFA 18:2 n-6 c,c", Some("LINOLEIC_ACID")),
    ("PUFA 18:3 n-3 c,c,c (ALA)", Some("ALPHA_LINOLENIC_ACID")),
    ("PUFA 20:4 n-6", Some("ARACHIDONIC_ACID")),
    ("PUFA 20:5 n-3 (EPA)", Some("EPA")),
    ("PUFA 22:6 n-3 (DHA)", Some("DHA")),
    ("Water", None),
    ("Nitrogen", None),
    ("Protein", Some("PROTEIN")),
    ("Total lipid (fat)", Some("FAT")),
    ("Total fat (NLEA)", None),
    ("Ash", Some("ASH")),
    ("Carbohydrate, by difference", Some("CARB")),
    ("Carbohydrate, by summation", None),
    ("Fiber, total dietary", Some("FIBER")),
    ("Sugars, Total", None),
    ("Sucrose", None),
    ("Glucose", None),
    ("Fructose", None),
    ("Lactose", None),
    ("Maltose", None),
    ("Starch", None),
    ("Calcium, Ca", Some("CALCIUM")),
    ("Iron, Fe", Some("IRON")),
    ("Magnesium, Mg", Some("MAGNESIUM")),
    ("Phosphorus, P", Some("PHOSPHORUS")),
    ("Potassium, K", Some("POTASSIUM")),
    ("Sodium, Na", Some("SODIUM")),
    ("Zinc, Zn", Some("ZINC")),
    ("Copper, Cu", Some("COPPER")),
    ("Manganese, Mn", Some("MANGANESE")),
    ("Iodine, I", Some("IODINE")),
    ("Selenium, Se", Some("SELENIUM")),
    ("Thiamin", Some("VITAMIN_B1")),
    ("Riboflavin", Some("VITAMIN_B2")),
    ("Niacin", Some("NIACIN")),
    ("Vitamin B-6", Some("VITAMIN_B6")),
    ("Folate, total", Some("FOLIC_ACID")),
    ("Choline, total", Some("CHOLINE")),
    ("Choline, free", None),
    ("Choline, from phosphocholine", None),
    ("Choline, from phosphotidyl choline", None),
    ("Choline, from glycerophosphocholine", None),
    ("Choline, from sphingomyelin", None),
    ("Betaine", None),
    ("Vitamin B-12", Some("VITAMIN_B12")),
    ("Vitamin A, RAE", Some("VITAMIN_A")),
    ("Retinol", None),
    ("Carotene, beta", Some("CAROTENE")),
    ("cis-beta-Carotene", None),
    ("trans-beta-Carotene", None),
    ("Carotene, alpha", None),
    ("Cryptoxanthin, beta", None),
    ("Cryptoxanthin, alpha", None),
    ("Lycopene", None),
    ("cis-Lycopene", None),
    ("trans-Lycopene", None),
    ("cis-Lutein/Zeaxanthin", None),
    ("Lutein", None),
    ("Zeaxanthin", None),
    ("Vitamin D (D2 + D3), International Units", None),
    ("Vitamin D (D2 + D3)", Some("VITAMIN_D")),
    ("Vitamin D2 (ergocalciferol)", None),
    ("Vitamin D3 (cholecalciferol)", None),
    ("25-hydroxycholecalciferol", None),
    ("Fatty acids, total saturated", None),
    ("Fatty acids, total monounsaturated", None),
    ("Fatty acids, total polyunsaturated", None),
    ("Cholesterol", Some("CHOLESTEROL")),
    ("Tryptophan", Some("TRYPTOPHAN")),
    ("Threonine", Some("THREONINE")),
    ("Isoleucine", Some("ISOLEUCINE")),
    ("Leucine", Some("LEUCINE")),
    ("Lysine", Some("LYSINE")),
    ("Methionine", Some("METHIONINE")),
    ("Phenylalanine", Some("PHENYLALANINE")),
    ("Tyrosine", Some("TYROSINE")),
    ("Valine", Some("VALINE")),
    ("Arginine", Some("ARGININE")),
    ("Histidine", Some("HISTIDINE")),
    ("Alanine", None),
    ("Aspartic acid", None),
    ("Glutamic acid", None),
    ("Glycine", None),
    ("Proline", None),
    ("Serine", None),
    ("Hydroxyproline", None),
    ("Cysteine", None),
    ("Energy (Atwater Specific Factors)", None),
    ("Energy (Atwater General Factors)", Some("ENERGY")),
    ("Fatty acids, total trans", None),
    ("Fiber, soluble", None),
    ("Fiber, insoluble", None),
    ("Galactose", None),
    ("Vitamin C, total ascorbic acid", Some("VITAMIN_C")),
    ("Pantothenic acid", Some("VITAMIN_B5")),
    ("Lutein + zeaxanthin", None),
    ("Phytoene", None),
    ("Phytofluene", None),
    ("Vitamin E (alpha-tocopherol)", Some("VITAMIN_E")),
    ("Tocopherol, beta", None),
    ("Tocopherol, gamma", None),
    ("Tocopherol, delta", None),
    ("Tocotrienol, alpha", None),
    ("Tocotrienol, beta", None),
    ("Tocotrienol, gamma", None),
    ("Tocotrienol, delta", None),
    ("Cystine", Some("CYSTINE")),
    ("Biotin", None),
    ("Total dietary fiber (AOAC 2011.25)", None),
    ("High Molecular Weight Dietary Fiber (HMWDF)", None),
    ("Low Molecular Weight Dietary Fiber (LMWDF)", None),
    ("Molybdenum, Mo", None),
    ("Vitamin K (phylloquinone)", None),
    ("Vitamin K (Dihydrophylloquinone)", None),
    ("Vitamin K (Menaquinone-4)", None),
    ("Citric acid", None),
    ("Malic acid", None),
];

/// China Food Composition Tables display name -> `Nutrient` key.
pub const CHINA_NAMES: &[(&str, Option<&str>)] = &[
    ("能量(Energy)", Some("ENERGY")),
    ("蛋白质(Protein)", Some("PROTEIN")),
    ("脂肪(Fat)", Some("FAT")),
    ("胆固醇(Cholesterol)", Some("CHOLESTEROL")),
    ("灰分(Ash)", Some("ASH")),
    ("碳水化合物(CHO)", Some("CARB")),
    ("总膳食纤维(Dietary fiber)", Some("FIBER")),
    ("胡萝卜素(Carotene)", Some("CAROTENE")),
    ("维生素A(Vitamin)", Some("VITAMIN_A")),
    ("硫胺素(Thiamin)", Some("VITAMIN_B1")),
    ("核黄素(Riboflavin)", Some("VITAMIN_B2")),
    ("烟酸(Niacin)", Some("NIACIN")),
    ("维生素C(Vitamin C)", Some("VITAMIN_C")),
    ("钙(Ca)", Some("CALCIUM")),
    ("磷(P)", Some("PHOSPHORUS")),
    ("钾(K)", Some("POTASSIUM")),
    ("钠(Na)", Some("SODIUM")),
    ("镁(Mg)", Some("MAGNESIUM")),
    ("铁(Fe)", Some("IRON")),
    ("锌(Zn)", Some("ZINC")),
    ("硒(Se)", Some("SELENIUM")),
    ("铜(Cu)", Some("COPPER")),
    ("锰(Mn)", Some("MANGANESE")),
    ("碘(I)", Some("IODINE")),
];

/// China rows that are never nutrients (water, totals, fatty-acid sums).
pub const CHINA_IGNORED: &[&str] = &[
    "水分(Water)",
    "合计(Total)",
    "饱和脂肪酸(SFA)",
    "单不饱和脂肪酸(MUFA)",
    "多不饱和脂肪酸(PUFA)",
    "α-TE",
    "食部(Edible)",
];

/// Names that only appear in the SR Legacy dataset.
///
/// SR Legacy keeps the same vocabulary as the portal view for everything the
/// portal already carried, so `USDA_NAMES` still applies; these are the extra
/// rows it adds (the vitamin K forms and the alpha/beta carotene split). Rows
/// that the portal expressed under a different label but SR Legacy reports
/// individually — `Carotene, beta` covers `cis-`/`trans-beta-Carotene`,
/// `Vitamin D (D2 + D3)` covers the D2/D3 forms — are listed as `None` so the
/// ambiguity stays visible instead of being double counted.
pub const SR_LEGACY_EXTRA: &[(&str, Option<&str>)] = &[
    ("Vitamin K (phylloquinone)", Some("VITAMIN_K")),
    ("Vitamin K (Dihydrophylloquinone)", None),
    ("Vitamin K (Menaquinone-4)", None),
    ("Carotene, alpha", None),
    ("cis-beta-Carotene", None),
    ("trans-beta-Carotene", None),
    ("Retinol", None),
    ("Folic acid", None),
    ("Folate, DFE", None),
    ("Folate, food", None),
    ("Vitamin E, added", None),
    ("Vitamin B-12, added", None),
    ("Vitamin A, IU", None),
    ("Vitamin D2 (ergocalciferol)", None),
    ("Vitamin D3 (cholecalciferol)", None),
    ("PUFA 18:2 n-6 c,c", Some("LINOLEIC_ACID")),
    ("PUFA 18:3 n-3 c,c,c (ALA)", Some("ALPHA_LINOLENIC_ACID")),
    ("PUFA 20:4 n-6", Some("ARACHIDONIC_ACID")),
    ("PUFA 20:5 n-3 (EPA)", Some("EPA")),
    ("PUFA 22:6 n-3 (DHA)", Some("DHA")),
];

/// Map an SR Legacy nutrient name onto a solver key.
///
/// Returns `Some(None)` for a deliberately dropped row, `None` for a name the
/// tables do not list (the caller records and skips it, as the portal path
/// does). SR Legacy's fatty-acid total rows (`PUFA 18:2`, `SFA 16:0`, …) are
/// dropped for the same reason as the portal's: an unspecified chain is not
/// safely one named acid.
pub fn map_sr_legacy_name(name: &str) -> Option<Option<&'static str>> {
    if let Some((_, value)) = SR_LEGACY_EXTRA.iter().find(|(key, _)| *key == name) {
        return Some(*value);
    }
    if let Some((_, value)) = USDA_NAMES.iter().find(|(key, _)| *key == name) {
        return Some(*value);
    }
    Some(None)
}
