//! Which source each food comes from.
//!
//! The five inline foods (`BALANCEIT`, `BARF`, `BONE_MEAL`,
//! `EGG_SHELL_POWDER`, `SALT`) have no remote source; `build.rs` in `food-core`
//! hardcodes them, so this tool only records their existence and never fetches
//! them.

/// One place a food's nutrient row can be fetched from.
///
/// A source is a *sample* of the food, not a piece of one: sources are never
/// merged, and `CHOOSE` in `food-core` names the single source the solver
/// reads. A food may have several sources available (`BEEF` exists in both
/// USDA portal and SR Legacy), and any of them can be the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// USDA FoodData Central portal view, by its food id.
    Usda(u64),
    /// China Food Composition Tables, by its food id.
    Chinanutri(u64),
    /// USDA SR Legacy, by its fdcId.
    SrLegacy(u64),
    /// Defined inline in `food-core`'s `build.rs`; nothing to fetch.
    Inline,
}

/// One catalog row: the canonical food name and the source fetched by default.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub name: &'static str,
    /// The source `food fetch` writes without an explicit choice.
    pub default_source: Source,
    /// USDA only: divide the cooked-food values to approximate the raw food,
    /// matching `convert_cooked_chicken_breast_to_uncoocked`.
    pub cooked_to_raw: bool,
}

/// Every food the solver knows, in the canonical order.
pub const CATALOG: &[Entry] = &[
    Entry {
        name: "BALANCEIT",
        default_source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "BAICAI",
        default_source: Source::Chinanutri(450),
        cooked_to_raw: false,
    },
    Entry {
        name: "JUANXINCAI",
        default_source: Source::Chinanutri(463),
        cooked_to_raw: false,
    },
    Entry {
        name: "BANANA",
        default_source: Source::Usda(1105314),
        cooked_to_raw: false,
    },
    Entry {
        name: "BARF",
        default_source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "BROCCOLI",
        default_source: Source::Usda(747447),
        cooked_to_raw: false,
    },
    Entry {
        name: "BEEF",
        default_source: Source::Usda(2646173),
        cooked_to_raw: false,
    },
    Entry {
        name: "BEEF_LIVER",
        default_source: Source::Chinanutri(836),
        cooked_to_raw: false,
    },
    Entry {
        name: "BEEN_SPROUT",
        default_source: Source::Chinanutri(400),
        cooked_to_raw: false,
    },
    Entry {
        name: "BELL_PEPER",
        default_source: Source::Usda(2258590),
        cooked_to_raw: false,
    },
    Entry {
        name: "BOCAI",
        default_source: Source::Chinanutri(473),
        cooked_to_raw: false,
    },
    Entry {
        name: "BOKCHOY",
        default_source: Source::Usda(2685572),
        cooked_to_raw: false,
    },
    Entry {
        name: "BONE_MEAL",
        default_source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "CANOLA_OIL",
        default_source: Source::Chinanutri(1495),
        cooked_to_raw: false,
    },
    Entry {
        name: "CABBAGE",
        default_source: Source::Usda(2346407),
        cooked_to_raw: false,
    },
    Entry {
        name: "CARROT",
        default_source: Source::Usda(2258586),
        cooked_to_raw: false,
    },
    Entry {
        name: "CELERY",
        default_source: Source::Usda(2346405),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_BREAST",
        default_source: Source::Usda(331960),
        cooked_to_raw: true,
    },
    Entry {
        name: "CHICKEN_HEART",
        default_source: Source::Chinanutri(885),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_LIVER",
        default_source: Source::Chinanutri(884),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_GIZZARD",
        default_source: Source::Chinanutri(887),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_THIGH",
        default_source: Source::Usda(2646171),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHINESE_LETTUS",
        default_source: Source::Chinanutri(482),
        cooked_to_raw: false,
    },
    Entry {
        name: "CUCUMBER",
        default_source: Source::Usda(2346406),
        cooked_to_raw: false,
    },
    Entry {
        name: "DUCK_GIZZARD",
        default_source: Source::Chinanutri(900),
        cooked_to_raw: false,
    },
    Entry {
        name: "EGG_SHELL_POWDER",
        default_source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "EGG",
        default_source: Source::Usda(748967),
        cooked_to_raw: false,
    },
    Entry {
        name: "EGG_YOLK",
        default_source: Source::SrLegacy(172184),
        cooked_to_raw: false,
    },
    Entry {
        name: "EGGPLANT",
        default_source: Source::Usda(2685577),
        cooked_to_raw: false,
    },
    Entry {
        name: "FUGUA",
        default_source: Source::Chinanutri(421),
        cooked_to_raw: false,
    },
    Entry {
        name: "JIANGDOU",
        default_source: Source::Chinanutri(398),
        cooked_to_raw: false,
    },
    Entry {
        name: "JIEGUA",
        default_source: Source::Chinanutri(423),
        cooked_to_raw: false,
    },
    Entry {
        name: "KONGXINCAI",
        default_source: Source::Chinanutri(493),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHAYOTE",
        default_source: Source::Chinanutri(420),
        cooked_to_raw: false,
    },
    Entry {
        name: "LUOBO",
        default_source: Source::Chinanutri(371),
        cooked_to_raw: false,
    },
    Entry {
        name: "OYSTER",
        default_source: Source::Chinanutri(1112),
        cooked_to_raw: false,
    },
    Entry {
        name: "PUMPKIN",
        default_source: Source::Chinanutri(426),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK",
        default_source: Source::Chinanutri(788),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_FAT",
        default_source: Source::Chinanutri(780),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_INTESTINE",
        default_source: Source::Chinanutri(790),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_LIVER",
        default_source: Source::Chinanutri(797),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_HEART",
        default_source: Source::Chinanutri(802),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_TONGUE",
        default_source: Source::Chinanutri(799),
        cooked_to_raw: false,
    },
    Entry {
        name: "POTATO",
        default_source: Source::Usda(2346403),
        cooked_to_raw: false,
    },
    Entry {
        name: "RICE",
        default_source: Source::Usda(2512381),
        cooked_to_raw: false,
    },
    Entry {
        name: "QINCAI",
        default_source: Source::Chinanutri(479),
        cooked_to_raw: false,
    },
    Entry {
        name: "SHANYAO",
        default_source: Source::Chinanutri(525),
        cooked_to_raw: false,
    },
    Entry {
        name: "SALT",
        default_source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "SIJIDOU",
        default_source: Source::Chinanutri(392),
        cooked_to_raw: false,
    },
    Entry {
        name: "SIGUA",
        default_source: Source::Chinanutri(429),
        cooked_to_raw: false,
    },
    Entry {
        name: "SWEET_POTATO",
        default_source: Source::Usda(2346404),
        cooked_to_raw: false,
    },
    Entry {
        name: "SOYBEAN_GREEN",
        default_source: Source::Chinanutri(391),
        cooked_to_raw: false,
    },
    Entry {
        name: "SOY_MILK",
        default_source: Source::Usda(1999630),
        cooked_to_raw: false,
    },
    Entry {
        name: "SHITAKE",
        default_source: Source::Chinanutri(584),
        cooked_to_raw: false,
    },
    Entry {
        name: "TOMATO",
        default_source: Source::Chinanutri(405),
        cooked_to_raw: false,
    },
    Entry {
        name: "TOFU_FIRM",
        default_source: Source::Chinanutri(334),
        cooked_to_raw: false,
    },
    Entry {
        name: "TOFU_SOFT",
        default_source: Source::Chinanutri(335),
        cooked_to_raw: false,
    },
    Entry {
        name: "WHITE_MUSHROOM",
        default_source: Source::Chinanutri(577),
        cooked_to_raw: false,
    },
    Entry {
        name: "BASA_FISH",
        default_source: Source::Chinanutri(1020),
        cooked_to_raw: false,
    },
    Entry {
        name: "WINTER_MELON",
        default_source: Source::Chinanutri(419),
        cooked_to_raw: false,
    },
    Entry {
        name: "ZIGANLAN",
        default_source: Source::Chinanutri(463),
        cooked_to_raw: false,
    },
    Entry {
        name: "ZUCCHINI",
        default_source: Source::Usda(2685568),
        cooked_to_raw: false,
    },
];

/// SR Legacy records for every food the dataset covers.
///
/// SR Legacy is the same USDA FoodData Central API under a different dataset.
/// Its rows carry the fuller amino-acid and fatty-acid panel, so `CHOOSE`
/// points most foods here at their SR Legacy row instead of the default one.
/// The rows are separate samples of the same food, never merged: the unselected
/// row stays on disk unused. Picked to match the other sources' food as closely
/// as the datasets allow.
pub const SR_LEGACY: &[(&str, u64)] = &[
    ("BAICAI", 169979),
    ("BANANA", 173944),
    ("BASA_FISH", 171952),
    ("BEEF", 171758),
    ("BEEF_LIVER", 169451),
    ("BEEN_SPROUT", 169957),
    ("BELL_PEPER", 170108),
    ("BOCAI", 168462),
    ("BOKCHOY", 170390),
    ("BROCCOLI", 170379),
    ("CABBAGE", 169975),
    ("CANOLA_OIL", 172336),
    ("CARROT", 170393),
    ("CELERY", 169988),
    ("CHAYOTE", 170402),
    ("CHICKEN_BREAST", 171140),
    ("CHICKEN_GIZZARD", 171456),
    ("CHICKEN_HEART", 171458),
    ("CHICKEN_LIVER", 171060),
    ("CHICKEN_THIGH", 172385),
    ("CHINESE_LETTUS", 169247),
    ("CUCUMBER", 168409),
    ("EGG", 171287),
    ("EGGPLANT", 169228),
    ("FUGUA", 169232),
    ("JIANGDOU", 169222),
    ("JUANXINCAI", 169975),
    ("KONGXINCAI", 169301),
    ("LUOBO", 168451),
    ("OYSTER", 174219),
    ("PORK", 168230),
    ("PORK_FAT", 167813),
    ("PORK_HEART", 168267),
    ("PORK_INTESTINE", 168266),
    ("PORK_LIVER", 167862),
    ("PORK_TONGUE", 168275),
    ("POTATO", 170026),
    ("PUMPKIN", 168448),
    ("QINCAI", 169988),
    ("RICE", 169756),
    ("SHANYAO", 170071),
    ("SHITAKE", 169242),
    ("SIGUA", 168414),
    ("SIJIDOU", 169961),
    ("SOYBEAN_GREEN", 169282),
    ("SOY_MILK", 172446),
    ("SWEET_POTATO", 168482),
    ("TOFU_FIRM", 172475),
    ("TOFU_SOFT", 172449),
    ("TOMATO", 170457),
    ("WHITE_MUSHROOM", 169251),
    ("WINTER_MELON", 170069),
    ("ZIGANLAN", 169975),
    ("ZUCCHINI", 169291),
];

/// The SR Legacy fdcId for a food, when one is curated.
pub fn sr_legacy_id(name: &str) -> Option<u64> {
    SR_LEGACY
        .iter()
        .find(|(food, _)| food.eq_ignore_ascii_case(name))
        .map(|(_, id)| *id)
}

/// Every curated SR Legacy record, whether it is the food's default source or
/// an alternative one that `CHOOSE` may point at.
///
/// A food that only SR Legacy carries names it in [`Source`], so the plain
/// [`sr_legacy_id`] table lookup would miss it.
pub fn sr_legacy_records() -> Vec<(&'static str, u64)> {
    CATALOG
        .iter()
        .filter_map(|entry| match entry.default_source {
            Source::SrLegacy(id) => Some((entry.name, id)),
            _ => sr_legacy_id(entry.name).map(|id| (entry.name, id)),
        })
        .collect()
}

/// MEXT (Japan) `食品番号` for the foods this table shares with the Japanese
/// food composition tables.
///
/// MEXT is the only source here that publishes iodine, biotin and pantothenic
/// acid, so it has the fullest panel for the foods it covers; no food points
/// `CHOOSE` at it yet. Numbers are `日本食品標準成分表2020年版（八訂）` 食品番号
/// values.
pub const MEXT: &[(&str, &str)] = &[
    ("PORK", "11115"),
    ("CHICKEN_BREAST", "11220"),
    ("CHICKEN_THIGH", "11224"),
    ("EGG", "12004"),
    ("RICE", "01083"),
    ("CARROT", "06214"),
    ("CABBAGE", "06061"),
    ("TOMATO", "06182"),
    ("POTATO", "02063"),
    ("SWEET_POTATO", "02006"),
    ("CUCUMBER", "06065"),
    ("EGGPLANT", "06191"),
    ("ZUCCHINI", "06116"),
    ("BANANA", "07107"),
    ("BROCCOLI", "06263"),
    ("CELERY", "06119"),
    ("BELL_PEPER", "06247"),
    ("SOY_MILK", "04052"),
    ("SHITAKE", "08039"),
    ("PUMPKIN", "06048"),
    ("LUOBO", "06134"),
    ("BOKCHOY", "06233"),
    ("SOYBEAN_GREEN", "06015"),
];

/// The MEXT food number for a food, when one is curated.
pub fn mext_number(name: &str) -> Option<&'static str> {
    MEXT.iter()
        .find(|(food, _)| food.eq_ignore_ascii_case(name))
        .map(|(_, number)| *number)
}

/// Filter the catalog by the food names a caller asked for.
///
/// Unknown names are returned separately so the caller can fail loudly rather
/// than silently fetching nothing.
pub fn select<'a>(wanted: impl IntoIterator<Item = &'a str>) -> (Vec<&'static Entry>, Vec<String>) {
    let mut chosen = Vec::new();
    let mut unknown = Vec::new();
    for raw in wanted {
        match CATALOG
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(raw))
        {
            Some(entry) => chosen.push(entry),
            None => unknown.push(raw.to_string()),
        }
    }
    (chosen, unknown)
}
