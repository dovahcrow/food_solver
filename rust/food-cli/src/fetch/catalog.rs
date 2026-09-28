//! Which source each food comes from, ported from `GETTERS` in `src/food.py`.
//!
//! The three inline foods (`BALANCEIT`, `BARF`, `EGG_SHELL_POWDER`) have no
//! remote source; `build.rs` in `food-core` hardcodes them, so this tool only
//! records their existence and never fetches them.

/// Where a food's nutrient row comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// USDA FoodData Central, by its portal food id.
    Usda(u64),
    /// China Food Composition Tables, by its food id.
    Chinanutri(u64),
    /// Defined inline in the Python source; nothing to fetch.
    Inline,
}

/// One catalog row: the canonical food name and where it is fetched from.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub name: &'static str,
    pub source: Source,
    /// USDA only: divide the cooked-food values to approximate the raw food,
    /// matching `convert_cooked_chicken_breast_to_uncoocked`.
    pub cooked_to_raw: bool,
}

/// Every food the solver knows, in the Python enum's order.
pub const CATALOG: &[Entry] = &[
    Entry {
        name: "BALANCEIT",
        source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "BAICAI",
        source: Source::Chinanutri(450),
        cooked_to_raw: false,
    },
    Entry {
        name: "JUANXINCAI",
        source: Source::Chinanutri(463),
        cooked_to_raw: false,
    },
    Entry {
        name: "BANANA",
        source: Source::Usda(1105314),
        cooked_to_raw: false,
    },
    Entry {
        name: "BARF",
        source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "BROCCOLI",
        source: Source::Usda(747447),
        cooked_to_raw: false,
    },
    Entry {
        name: "BEEF",
        source: Source::Usda(2646173),
        cooked_to_raw: false,
    },
    Entry {
        name: "BEEN_SPROUT",
        source: Source::Chinanutri(400),
        cooked_to_raw: false,
    },
    Entry {
        name: "BELL_PEPER",
        source: Source::Usda(2258590),
        cooked_to_raw: false,
    },
    Entry {
        name: "BOCAI",
        source: Source::Chinanutri(473),
        cooked_to_raw: false,
    },
    Entry {
        name: "BOKCHOY",
        source: Source::Usda(2685572),
        cooked_to_raw: false,
    },
    Entry {
        name: "CANOLA_OIL",
        source: Source::Chinanutri(1495),
        cooked_to_raw: false,
    },
    Entry {
        name: "CABBAGE",
        source: Source::Usda(2346407),
        cooked_to_raw: false,
    },
    Entry {
        name: "CARROT",
        source: Source::Usda(2258586),
        cooked_to_raw: false,
    },
    Entry {
        name: "CELERY",
        source: Source::Usda(2346405),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_BREAST",
        source: Source::Usda(331960),
        cooked_to_raw: true,
    },
    Entry {
        name: "CHICKEN_HEART",
        source: Source::Chinanutri(885),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_LIVER",
        source: Source::Chinanutri(884),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_GIZZARD",
        source: Source::Chinanutri(887),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHICKEN_THIGH",
        source: Source::Usda(2646171),
        cooked_to_raw: false,
    },
    Entry {
        name: "CHINESE_LETTUS",
        source: Source::Chinanutri(482),
        cooked_to_raw: false,
    },
    Entry {
        name: "CUCUMBER",
        source: Source::Usda(2346406),
        cooked_to_raw: false,
    },
    Entry {
        name: "DUCK_GIZZARD",
        source: Source::Chinanutri(900),
        cooked_to_raw: false,
    },
    Entry {
        name: "EGG_SHELL_POWDER",
        source: Source::Inline,
        cooked_to_raw: false,
    },
    Entry {
        name: "EGG",
        source: Source::Usda(748967),
        cooked_to_raw: false,
    },
    Entry {
        name: "EGGPLANT",
        source: Source::Usda(2685577),
        cooked_to_raw: false,
    },
    Entry {
        name: "FUGUA",
        source: Source::Chinanutri(421),
        cooked_to_raw: false,
    },
    Entry {
        name: "JIANGDOU",
        source: Source::Chinanutri(398),
        cooked_to_raw: false,
    },
    Entry {
        name: "JIEGUA",
        source: Source::Chinanutri(423),
        cooked_to_raw: false,
    },
    Entry {
        name: "KONGXINCAI",
        source: Source::Chinanutri(493),
        cooked_to_raw: false,
    },
    Entry {
        name: "KUIGUA",
        source: Source::Chinanutri(420),
        cooked_to_raw: false,
    },
    Entry {
        name: "LUOBO",
        source: Source::Chinanutri(371),
        cooked_to_raw: false,
    },
    Entry {
        name: "OYSTER",
        source: Source::Chinanutri(1112),
        cooked_to_raw: false,
    },
    Entry {
        name: "PUMPKIN",
        source: Source::Chinanutri(426),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK",
        source: Source::Chinanutri(788),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_FAT",
        source: Source::Chinanutri(780),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_INTESTINE",
        source: Source::Chinanutri(790),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_LIVER",
        source: Source::Chinanutri(797),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_HEART",
        source: Source::Chinanutri(802),
        cooked_to_raw: false,
    },
    Entry {
        name: "PORK_TONGUE",
        source: Source::Chinanutri(799),
        cooked_to_raw: false,
    },
    Entry {
        name: "POTATO",
        source: Source::Usda(2346403),
        cooked_to_raw: false,
    },
    Entry {
        name: "RICE",
        source: Source::Usda(2512381),
        cooked_to_raw: false,
    },
    Entry {
        name: "QINCAI",
        source: Source::Chinanutri(479),
        cooked_to_raw: false,
    },
    Entry {
        name: "SHANYAO",
        source: Source::Chinanutri(525),
        cooked_to_raw: false,
    },
    Entry {
        name: "SALT",
        source: Source::Chinanutri(1565),
        cooked_to_raw: false,
    },
    Entry {
        name: "SIJIDOU",
        source: Source::Chinanutri(392),
        cooked_to_raw: false,
    },
    Entry {
        name: "SIGUA",
        source: Source::Chinanutri(429),
        cooked_to_raw: false,
    },
    Entry {
        name: "SWEET_POTATO",
        source: Source::Usda(2346404),
        cooked_to_raw: false,
    },
    Entry {
        name: "SOYBEAN_GREEN",
        source: Source::Chinanutri(391),
        cooked_to_raw: false,
    },
    Entry {
        name: "SOY_MILK",
        source: Source::Usda(1999630),
        cooked_to_raw: false,
    },
    Entry {
        name: "SHITAKE",
        source: Source::Chinanutri(584),
        cooked_to_raw: false,
    },
    Entry {
        name: "TOMATO",
        source: Source::Chinanutri(405),
        cooked_to_raw: false,
    },
    Entry {
        name: "TOFU_FIRM",
        source: Source::Chinanutri(334),
        cooked_to_raw: false,
    },
    Entry {
        name: "TOFU_SOFT",
        source: Source::Chinanutri(335),
        cooked_to_raw: false,
    },
    Entry {
        name: "WHITE_MUSHROOM",
        source: Source::Chinanutri(577),
        cooked_to_raw: false,
    },
    Entry {
        name: "BASA_FISH",
        source: Source::Chinanutri(1020),
        cooked_to_raw: false,
    },
    Entry {
        name: "WINTER_MELON",
        source: Source::Chinanutri(419),
        cooked_to_raw: false,
    },
    Entry {
        name: "ZIGANLAN",
        source: Source::Chinanutri(463),
        cooked_to_raw: false,
    },
    Entry {
        name: "ZUCCHINI",
        source: Source::Usda(2685568),
        cooked_to_raw: false,
    },
];

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
