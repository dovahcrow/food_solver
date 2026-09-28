//! Download the food nutrient caches the solver embeds.
//!
//! `build.rs` in `food-core` turns `foods/*.json` into the embedded table, but
//! those files were only ever produced by the Python frontends. This module
//! ports both getters — USDA FoodData Central and the China Food Composition
//! Tables — so the `food fetch` command can refresh the cache without Python.
//!
//! The files it writes are byte-for-byte compatible with the Python ones:
//! sorted keys, four-space indent, an `__name__` display name, and values in
//! the solver's base units (grams per gram of food, joules per gram).

pub mod catalog;
pub mod chinanutri;
pub mod nutrients;
pub mod units;
pub mod usda;

use std::path::Path;
use std::sync::LazyLock;

use anyhow::{anyhow, Context, Error};
use culpa::{throw, throws};
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

/// Fetch a single catalog entry.
///
/// Inline foods have no remote source and return an error; the caller filters
/// them out beforehand.
#[throws(Error)]
pub fn fetch(entry: &Entry) -> Row {
    let row = match entry.source {
        Source::Usda(id) => usda::get(id)?,
        Source::Chinanutri(id) => chinanutri::get(id)?,
        Source::Inline => throw!(anyhow!(
            "{} is defined inline in the Python source and has no remote data",
            entry.name
        )),
    };

    let nutrients = if entry.cooked_to_raw {
        // Same dirty fix as the Python frontend: some nutrients are only
        // published for the cooked food, so divide back towards the raw one.
        row.nutrients
            .into_iter()
            .map(|(key, value)| (key, value / 1.49))
            .collect()
    } else {
        row.nutrients
    };

    Row {
        name: row.name,
        nutrients,
        skipped: row.skipped,
    }
}

/// Render a row as the cache JSON the solver reads.
///
/// The bytes match Python's `json.dump(row, indent=4, sort_keys=True,
/// ensure_ascii=False)` for the same data: keys sorted, four-space indent,
/// `__name__` last (its underscore sorts after every uppercase key), raw UTF-8
/// for non-ASCII names, and the same shortest-round-trip float literals. That
/// keeps a Rust refresh diffing cleanly against the committed caches.
#[throws(Error)]
pub fn to_cache_json(row: &Row) -> String {
    let mut entries = row.nutrients.clone();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    let mut out = String::from("{\n");
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        out.push_str("    ");
        out.push_str(&json_string(key));
        out.push_str(": ");
        out.push_str(&python_float(*value));
    }
    if !row.nutrients.is_empty() {
        out.push_str(",\n");
    }
    out.push_str("    ");
    out.push_str(&json_string("__name__"));
    out.push_str(": ");
    out.push_str(&json_string(&row.name));
    out.push_str("\n}");
    out
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

/// Fetch one food and write its cache file under `directory`.
///
/// `directory` is created if needed, matching the Python `get_or_load`.
#[throws(Error)]
pub fn refresh(entry: &Entry, directory: &Path) -> Row {
    std::fs::create_dir_all(directory)
        .with_context(|| format!("create {}", directory.display()))?;
    let row = fetch(entry)?;
    let path = directory.join(format!("{}.json", entry.name));
    let text = to_cache_json(&row)?;
    std::fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    row
}

/// Where the cache lives: `FOODS_DIR` when set, else `foods` under the CWD.
pub fn foods_dir() -> std::path::PathBuf {
    std::env::var_os("FOODS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("foods"))
}
