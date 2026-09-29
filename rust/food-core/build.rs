//! Build script: turns the food database into Rust source.
//!
//! The cache holds one file per food per source, named
//! `{FOOD}_{Source}.json` (for example `BAICAI_Chinanutri.json`,
//! `BEEF_SrLegacy.json`). Every row is embedded, keyed by its food and source;
//! which one a run uses is decided at runtime by `CHOOSE` in `src/foods.rs`, so
//! this script never has to read that table.
//! The file name supplies the food and its source, so the JSON body is nothing
//! but nutrients. The four foods that exist only as inline recipes are still
//! hardcoded below. `src/foods.rs` includes the generated file.
//!
//! The nutrient field names are derived from `src/nutrient.rs`, so adding a
//! nutrient to the enum is enough to have it flow through to the table.

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use food_base::NUTRIENT_FIELDS;

/// Foods that have no JSON cache; they are defined inline here.
///
/// Only the overridden nutrients are listed; every other nutrient is a
/// measured zero rather than missing data.
const INLINE_FOODS: &[(&str, &[(&str, f64)])] = &[
    (
        "BALANCEIT",
        &[
            ("calcium", 0.1518),
            ("phosphorus", 0.07980000000000001),
            ("potassium", 0.10092000000000001),
            ("sodium", 0.0104),
            ("chloride", 0.01752),
            ("magnesium", 0.008),
            ("iron", 0.0016460000000000003),
            ("copper", 0.00015132),
            ("manganese", 0.0001132),
            ("zinc", 0.00265),
            ("iodine", 3.432e-05),
            ("selenium", 3.6000000000000003e-06),
            ("vitamin_a", 3.4019999999999996e-05),
            ("vitamin_d", 2.75e-07),
            ("vitamin_e", 0.0030015999999999997),
            ("vitamin_b1", 3.212e-05),
            ("vitamin_b2", 9.1e-05),
            ("vitamin_b5", 0.00017360000000000002),
            ("niacin", 0.00024031999999999998),
            ("folic_acid", 6e-06),
            ("vitamin_b12", 5.7e-07),
            ("choline", 0.024509),
        ],
    ),
    (
        "BARF",
        &[
            ("ash", 0.182),
            ("fiber", 0.07),
            ("protein", 0.0275),
            ("fat", 0.0047),
            ("calcium", 0.0456),
            ("phosphorus", 0.007600000000000001),
            ("potassium", 0.0165),
            ("sodium", 0.0018),
            ("chloride", 0.0),
            ("magnesium", 0.0048),
            ("iron", 0.000614),
            ("copper", 8.779999999999999e-06),
            ("manganese", 4.5600000000000004e-05),
            ("zinc", 3.75e-05),
            ("iodine", 2.84e-05),
            ("selenium", 3.6e-07),
            ("vitamin_a", 3e-07),
            ("vitamin_c", 2e-05),
            ("vitamin_d", 7.5e-09),
            ("vitamin_e", 2.3e-05),
            ("vitamin_b1", 2.2e-05),
            ("vitamin_b2", 1e-05),
            ("vitamin_b5", 1.3000000000000001e-05),
            ("vitamin_b6", 1.6e-05),
            ("vitamin_b7", 4.36e-07),
            ("vitamin_b12", 7e-09),
            ("niacin", 0.00011800000000000001),
            ("folic_acid", 2.05e-06),
        ],
    ),
    (
        "EGG_SHELL_POWDER",
        &[("calcium", 0.35), ("magnesium", 0.014)],
    ),
    // A rendered beef bone meal, taken from a product label's guaranteed
    // minimums: crude protein >= 26%, crude fat >= 6%, calcium >= 16%,
    // phosphorus >= 7%. The other nutrients stay a measured zero, so the label
    // simply does not claim them. Energy is Atwater from the two macros
    // (0.26*4 + 0.06*9 kcal/g), since a bone meal is not a free-energy food.
    (
        "BONE_MEAL",
        &[
            ("protein", 0.26),
            ("fat", 0.06),
            ("calcium", 0.16),
            ("phosphorus", 0.07),
            ("energy", 6610.72),
        ],
    ),
    // Salt is sodium chloride. Only the sodium is modelled: the trace minerals
    // the Chinese table also lists for it are not worth a solver variable, and
    // inlining keeps the rest of the row a measured zero instead of missing.
    ("SALT", &[("sodium", 0.39311)]),
];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let foods_dir = manifest.join("../../foods");
    let output = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("foods_data.rs");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", foods_dir.display());
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    // The field list comes from `food-base` as a Rust constant, so this
    // script never has to parse source text.
    let fields: Vec<String> = NUTRIENT_FIELDS.iter().map(|f| (*f).to_string()).collect();
    let rows = read_rows(&foods_dir, &fields);
    fs::write(&output, render(&fields, &rows)).expect("write generated food table");

    stamp_build_info(&manifest);
}

/// Expose the build moment and git revision to `src/build_info.rs`.
///
/// The values are baked in as env vars, so a binary can report which revision
/// it came from. `SOURCE_DATE_EPOCH` overrides the clock for reproducible
/// builds; without a checkout the revision is `unknown`.
fn stamp_build_info(manifest: &Path) {
    let seconds = match env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        Some(epoch) => epoch,
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or(0),
    };
    println!("cargo:rustc-env=FOOD_BUILD_DATE={}", iso8601_utc(seconds));
    println!("cargo:rustc-env=FOOD_GIT_SHA={}", git_sha(manifest));

    // A new commit should restamp even when no tracked source changed.
    let head = manifest.join("../../.git/HEAD");
    if head.exists() {
        println!("cargo:rerun-if-changed={}", head.display());
    }
}

/// Format UNIX seconds as `YYYY-MM-DDTHH:MM:SSZ`.
fn iso8601_utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let time = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60,
    )
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// The short git revision of the checkout containing this crate.
fn git_sha(manifest: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(manifest)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// The source suffixes a cache file may carry, longest first so the split on
/// the last `_` cannot mistake part of a source name for a food name.
const SOURCES: &[&str] = &["Chinanutri", "SrLegacy", "Inline", "Usda", "Mext"];

/// Split `BAICAI_Chinanutri` into its food name and source.
fn split_name(stem: &str) -> Option<(&str, &str)> {
    for source in SOURCES {
        if let Some(food) = stem.strip_suffix(&format!("_{source}")) {
            if !food.is_empty() {
                return Some((food, source));
            }
        }
    }
    None
}

/// Every cached row, keyed by its food and source.
///
/// Nothing is filtered here: the cache may hold several sources per food, and
/// which one a run uses is a runtime decision, not a build-time one.
fn read_rows(
    foods_dir: &Path,
    fields: &[String],
) -> BTreeMap<(String, String), BTreeMap<String, f64>> {
    let known: Vec<&str> = fields.iter().map(String::as_str).collect();
    let mut rows: BTreeMap<(String, String), BTreeMap<String, f64>> = BTreeMap::new();

    let mut entries: Vec<PathBuf> = fs::read_dir(foods_dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", foods_dir.display()))
        .map(|entry| entry.expect("food cache entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort();

    for path in entries {
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("food cache file name");
        let Some((name, source)) = split_name(stem) else {
            // A file that does not match the layout is ignored rather than
            // guessed at.
            continue;
        };
        // Watch each cache file: the directory entry only catches new files.
        println!("cargo:rerun-if-changed={}", path.display());

        let text = fs::read_to_string(&path).expect("read food cache");
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
        let object = parsed
            .as_object()
            .unwrap_or_else(|| panic!("{} is not a JSON object", path.display()));

        let mut row = BTreeMap::new();
        for (key, value) in object {
            if key == "__name__" {
                continue;
            }
            let field = key.to_ascii_lowercase();
            assert!(
                known.contains(&field.as_str()),
                "{}: unknown nutrient {key}",
                path.display()
            );
            let amount = value
                .as_f64()
                .unwrap_or_else(|| panic!("{}: {key} is not a number", path.display()));
            row.insert(field, amount);
        }
        assert!(!row.is_empty(), "{} has no nutrient data", path.display());
        assert!(
            rows.insert((name.to_owned(), source.to_owned()), row)
                .is_none(),
            "{} duplicates a food/source row",
            path.display()
        );
    }

    for (name, overrides) in INLINE_FOODS {
        let mut row: BTreeMap<String, f64> =
            fields.iter().map(|field| (field.clone(), 0.0)).collect();
        for (field, amount) in *overrides {
            assert!(known.contains(field), "inline food {name}: unknown {field}");
            row.insert((*field).to_owned(), *amount);
        }
        rows.insert(((*name).to_owned(), "Inline".to_owned()), row);
    }

    assert!(
        !rows.is_empty(),
        "no food rows found under {}",
        foods_dir.display()
    );
    rows
}

/// Render the Rust source that `src/foods.rs` includes.
///
/// Emits the `FoodName` enum (one variant per food, so callers are type safe
/// and the set is exhaustive), its `name()` mapping, the `ALL` list, and the
/// `(food, source, row)` table.
fn render(fields: &[String], rows: &BTreeMap<(String, String), BTreeMap<String, f64>>) -> String {
    let mut names: Vec<&str> = rows.keys().map(|(name, _)| name.as_str()).collect();
    names.dedup();

    // Every source the tool can write, not just the ones already cached, so a
    // variant like `Mext` exists before its first cache file does.
    let mut sources: Vec<&str> = SOURCES.to_vec();
    sources.sort_unstable();
    sources.dedup();

    let mut out = String::from(
        "// @generated by rust/food-core/build.rs -- do not edit.\n\
         //\n\
         // The `FoodName` and `FoodSource` enums, plus every cached row (one\n\
         // per food per source) from foods/*.json and the inline recipes. Field\n\
         // order follows the Nutrient enum. Which row a run uses is chosen at\n\
         // runtime by `CHOOSE` in foods.rs.\n\n\
         /// Every food the solver knows.\n\
         ///\n\
         /// Variants keep the canonical SCREAMING_SNAKE name so they read the\n\
         /// same as the cache files.\n\
         #[allow(non_camel_case_types)]\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n\
         pub enum FoodName {\n",
    );
    for name in &names {
        let _ = writeln!(out, "    {name},");
    }
    out.push_str(
        "}\n\nimpl FoodName {\n    /// The canonical SCREAMING_SNAKE name.\n    pub fn name(self) -> &'static str {\n        match self {\n",
    );
    for name in &names {
        let _ = writeln!(out, "            FoodName::{name} => \"{name}\",");
    }
    out.push_str(
        "        }\n    }\n}\n\n/// Every food, sorted by name.\npub const ALL: &[FoodName] = &[\n",
    );
    for name in &names {
        let _ = writeln!(out, "    FoodName::{name},");
    }

    out.push_str(
        "];\n\n/// Where a food's nutrient row comes from.\n\
         ///\n\
         /// The spelling in `name()` is the file-name tag\n\
         /// (`{FOOD}_{Source}.json`) and the spelling `parse` accepts.\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n\
         pub enum FoodSource {\n",
    );
    for source in &sources {
        let _ = writeln!(out, "    {source},");
    }
    out.push_str(
        "}\n\nimpl FoodSource {\n    /// Every source, sorted by tag.\n    pub const ALL: &'static [FoodSource] = &[\n",
    );
    for source in &sources {
        let _ = writeln!(out, "        FoodSource::{source},");
    }
    out.push_str(
        "    ];\n\n    /// The file-name tag, e.g. `SrLegacy`.\n    pub fn name(self) -> &'static str {\n        match self {\n",
    );
    for source in &sources {
        let _ = writeln!(out, "            FoodSource::{source} => \"{source}\",");
    }
    out.push_str(
        "        }\n    }\n\n    /// Parse a file-name tag, case-insensitively.\n    pub fn parse(tag: &str) -> Option<Self> {\n        Self::ALL\n            .iter()\n            .copied()\n            .find(|source| source.name().eq_ignore_ascii_case(tag))\n    }\n}\n\n",
    );

    out.push_str(
        "/// One row per (food, source). `Food` field order follows the enum.\n\
         pub const FOOD_ROWS: &[(FoodName, FoodSource, Food)] = &[\n",
    );
    for ((name, source), row) in rows {
        out.push_str("    (\n");
        let _ = writeln!(out, "        FoodName::{name},");
        let _ = writeln!(out, "        FoodSource::{source},");
        out.push_str("        Food {\n");
        // `Food::default()` is not const-callable, so every field is spelled
        // out; a missing nutrient is `None`, which stays distinct from a
        // measured zero.
        for field in fields {
            match row.get(field) {
                Some(amount) => {
                    let _ = writeln!(out, "            {field}: Some({}),", number(*amount));
                }
                None => {
                    let _ = writeln!(out, "            {field}: None,");
                }
            }
        }
        out.push_str("        },\n    ),\n");
    }
    out.push_str("];\n");
    out
}

/// A literal that round-trips exactly, without the exponent leading zeros.
fn number(value: f64) -> String {
    format!("{value:?}")
}
