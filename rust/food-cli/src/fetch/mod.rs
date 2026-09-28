//! Download the food nutrient caches the solver embeds.
//!
//! `build.rs` in `food-core` turns `foods/*.json` into the embedded table, but
//! those files were only ever produced by the Python frontends. This module
//! ports all three getters — USDA FoodData Central, the China Food Composition
//! Tables, and USDA SR Legacy — so the `food fetch` command can refresh the
//! cache without Python.
//!
//! The files it writes are byte-for-byte compatible with the Python ones:
//! sorted keys, four-space indent, an `__name__` display name, and values in
//! the solver's base units (grams per gram of food, joules per gram).

pub mod catalog;
pub mod chinanutri;
pub mod mext;
pub mod nutrients;
pub mod sr_legacy;
pub mod units;
pub mod usda;

use std::path::Path;
use std::sync::LazyLock;

use anyhow::{anyhow, Context, Error};
use culpa::{throw, throws};
use food_core::FoodSource;
use regex::Regex;

pub use catalog::{Entry, Source, CATALOG};

/// Matches a China table cell holding exactly `<number><unit>`.
///
/// Mirrors the Python getter's `((\d*\.)?\d+)(g|mg|μg|kJ)$`, so a cell with a
/// footnote or a dash is skipped rather than misread.
pub static AMOUNT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^((\d*\.)?\d+)(g|mg|μg|kJ)$").expect("amount regex"));

/// One food's display name and its nutrient row.
///
/// The same shape both getters return, so one cache writer serves them all.
#[derive(Debug, Clone)]
pub struct Row {
    pub name: String,
    /// Keys are `Nutrient` names such as `PROTEIN`; values are grams per gram,
    /// except `ENERGY`, which is joules per gram.
    pub nutrients: Vec<(&'static str, f64)>,
    /// Display names the source returned that the table does not map. The
    /// Python getter prints these and carries on; keeping them makes the same
    /// drift visible without failing the fetch.
    pub skipped: Vec<String>,
}

/// One cache file: a row plus the source tag that names its file.
///
/// Which source the solver embeds is decided once, by the `CHOOSE` table in
/// `catalog.rs`; the JSON body carries no provenance of its own.
#[derive(Debug, Clone)]
pub struct CacheFile {
    pub row: Row,
    pub source: FoodSource,
}

/// Fetch one food from its primary source into a cache file.
///
/// The row is written as `{FOOD}_{Source}.json` with a `choose` property; this
/// is the only source a plain `food fetch` writes.
#[throws(Error)]
pub fn fetch_primary(entry: &Entry, directory: &Path) -> CacheFile {
    let mut row = match entry.source {
        Source::Usda(id) => usda::get(id)?,
        Source::Chinanutri(id) => chinanutri::get(id)?,
        Source::Inline => throw!(anyhow!(
            "{} is defined inline in the Python source and has no remote data",
            entry.name
        )),
    };

    if entry.cooked_to_raw {
        // Same dirty fix as the Python frontend: some nutrients are only
        // published for the cooked food, so divide back towards the raw one.
        row.nutrients = row
            .nutrients
            .into_iter()
            .map(|(key, value)| (key, value / 1.49))
            .collect();
    }

    let source = match entry.source {
        Source::Usda(_) => FoodSource::Usda,
        Source::Chinanutri(_) => FoodSource::Chinanutri,
        Source::Inline => FoodSource::Inline,
    };
    write_cache(directory, entry.name, row, source)?
}

/// Fetch this food's curated SR Legacy record into its own cache file.
///
/// Returns `None` when no SR Legacy record is curated for the food.
#[throws(Error)]
pub fn fetch_sr_legacy(entry: &Entry, directory: &Path) -> Option<CacheFile> {
    let Some(id) = catalog::sr_legacy_id(entry.name) else {
        return None;
    };
    let row = sr_legacy::get(id)?;
    let file = write_cache(directory, entry.name, row, FoodSource::SrLegacy)?;
    Some(file)
}

/// Fetch this food's MEXT record into its own cache file.
///
/// MEXT ships as three whole-country workbooks, so `foods` is the already
/// loaded index from [`load_mext`]; looking the food up there avoids
/// re-downloading the workbooks per food.
#[throws(Error)]
pub fn fetch_mext(
    entry: &Entry,
    foods: &std::collections::HashMap<String, mext::MextFood>,
    directory: &Path,
) -> Option<CacheFile> {
    let Some(number) = catalog::mext_number(entry.name) else {
        return None;
    };
    let Some(food) = foods.get(number) else {
        throw!(anyhow!(
            "MEXT food {number} for {} is not in the workbooks",
            entry.name
        ));
    };
    let row = Row {
        name: format!("{} {}", food.number, food.name),
        nutrients: food.nutrients.clone(),
        skipped: Vec::new(),
    };
    let file = write_cache(directory, entry.name, row, FoodSource::Mext)?;
    Some(file)
}

/// Write one source's row as `{FOOD}_{Source}.json`.
#[throws(Error)]
fn write_cache(directory: &Path, food: &str, row: Row, source: FoodSource) -> CacheFile {
    std::fs::create_dir_all(directory)
        .with_context(|| format!("create {}", directory.display()))?;
    let file = CacheFile { row, source };
    let path = directory.join(format!("{food}_{}.json", source.name()));
    let text = to_cache_json(&file)?;
    std::fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    file
}

/// Render a cache file, including its provenance and `choose` flag.
///
/// The bytes match Python's `json.dump(row, indent=4, sort_keys=True,
/// ensure_ascii=False)` for the same data: keys sorted, four-space indent,
/// `__name__` last (its underscore sorts after every uppercase key), raw UTF-8
/// for non-ASCII names, and the same shortest-round-trip float literals. The
/// source lives in the file name, not in the body.
#[throws(Error)]
pub fn to_cache_json(file: &CacheFile) -> String {
    let mut lines: Vec<String> = file
        .row
        .nutrients
        .iter()
        .map(|(key, value)| format!("    {}: {}", json_string(key), python_float(*value)))
        .collect();
    lines.sort();
    lines.push(format!(
        "    {}: {}",
        json_string("__name__"),
        json_string(&file.row.name)
    ));

    format!("{{\n{}\n}}", lines.join(",\n"))
}

/// Quote a string the way Python's `json.dumps` does with `ensure_ascii=False`.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Format a float the way Python's `repr` does.
///
/// Rust's `{:?}` already gives the shortest round-tripping digits with Python's
/// fixed/scientific switch points; only the exponent spelling differs, so the
/// sign and two-digit padding are added here (`4e-5` -> `4e-05`, `1e16` ->
/// `1e+16`).
fn python_float(value: f64) -> String {
    let text = format!("{value:?}");
    let Some((mantissa, exponent)) = text.split_once('e') else {
        return text;
    };
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{:02}", exponent.abs())
}

/// Download the three MEXT workbooks and index every food by its 食品番号.
#[throws(Error)]
pub fn load_mext() -> std::collections::HashMap<String, mext::MextFood> {
    mext::fetch_all()?
        .into_iter()
        .map(|food| (food.number.clone(), food))
        .collect()
}

/// Every curated SR Legacy food expanded straight from the bulk dataset.
///
/// `owned` pairs each catalog food with its curated SR Legacy fdcId; the
/// dataset holds thousands of foods but only the owned ids get a file, so the
/// result stays `{FOOD}_{Source}.json` rather than the USDA description.
#[throws(Error)]
pub fn expand_sr_legacy(payload: &str, owned: &[(String, u64)], directory: &Path) -> Vec<String> {
    let data: serde_json::Value =
        serde_json::from_str(payload).context("parse SR Legacy dataset")?;
    let foods = data
        .get("SRLegacyFoods")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("dataset has no SRLegacyFoods array"))?;

    std::fs::create_dir_all(directory)
        .with_context(|| format!("create {}", directory.display()))?;
    let mut written = Vec::new();
    for food in foods {
        let Some(id) = food.get("fdcId").and_then(serde_json::Value::as_u64) else {
            continue;
        };
        let Some((name, _)) = owned.iter().find(|(_, want)| *want == id) else {
            continue;
        };
        let row = sr_legacy::parse(&food.to_string())?;
        let file = write_cache(directory, name, row, FoodSource::SrLegacy)?;
        written.push(format!("{name}_{}.json", file.source.name()));
    }
    written
}

/// Where the cache lives: `FOODS_DIR` when set, else `foods` under the CWD.
pub fn foods_dir() -> std::path::PathBuf {
    std::env::var_os("FOODS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("foods"))
}
