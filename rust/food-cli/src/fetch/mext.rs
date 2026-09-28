//! Japan MEXT food composition getter.
//!
//! `日本食品標準成分表2020年版（八訂）` is published as Excel workbooks, three of
//! which matter here: the main table (`_012`, sheet `表全体`), the amino-acid
//! table (`_022`), and the fatty-acid table (`_032`). Each workbook carries a
//! `成分識別子` (component identifier) row naming every column and a `単位` row
//! giving its unit, so columns are located by tag rather than by position.
//! MEXT is the only one of the three sources that publishes iodine, biotin and
//! pantothenic acid for every food, which is why it earns its place.
//!
//! Cells use MEXT's notation: `-` means not measured, `Tr` means a trace too
//! small to quantify, and a parenthesised value is an estimate. `Tr` and `-`
//! both become "no data" here; an estimate keeps its value.

use std::io::Cursor;

use anyhow::{anyhow, Context, Error};
use calamine::{Data, Reader, Xlsx};
use culpa::throws;

use super::units::{G, KCAL, KJ, MCG, MG};

/// The three published workbooks that make up a food's row.
pub const MAIN_URL: &str = "https://www.mext.go.jp/content/20201225-mxt_kagsei-mext_01110_012.xlsx";
/// Amino-acid table (`第1表`).
pub const AMINO_URL: &str =
    "https://www.mext.go.jp/content/20201225-mxt_kagsei-mext_01110_022.xlsx";
/// Fatty-acid table (`第1表`).
pub const FATTY_URL: &str =
    "https://www.mext.go.jp/content/20201225-mxt_kagsei-mext_01110_032.xlsx";

/// Main-table tag -> solver key. `None` is a column the solver does not model,
/// kept explicit so an unmapped tag is visible rather than silently wrong.
const MAIN_TAGS: &[(&str, Option<&str>)] = &[
    ("ENERC", Some("ENERGY")),
    ("ENERC_KCAL", None),
    ("PROT-", Some("PROTEIN")),
    ("FAT-", Some("FAT")),
    ("CHOAVLDF-", Some("CARB")),
    ("CHOCDF-", None),
    ("FIB-", Some("FIBER")),
    ("ASH", Some("ASH")),
    ("CHOLE", Some("CHOLESTEROL")),
    ("NA", Some("SODIUM")),
    ("K", Some("POTASSIUM")),
    ("CA", Some("CALCIUM")),
    ("MG", Some("MAGNESIUM")),
    ("P", Some("PHOSPHORUS")),
    ("FE", Some("IRON")),
    ("ZN", Some("ZINC")),
    ("CU", Some("COPPER")),
    ("MN", Some("MANGANESE")),
    ("ID", Some("IODINE")),
    ("SE", Some("SELENIUM")),
    ("CR", None),
    ("MO", None),
    ("VITA_RAE", Some("VITAMIN_A")),
    ("RETOL", None),
    ("CARTA", None),
    ("CARTB", Some("CAROTENE")),
    ("CARTBEQ", None),
    ("CRYPXB", None),
    ("VITD", Some("VITAMIN_D")),
    ("TOCPHA", Some("VITAMIN_E")),
    ("TOCPHB", None),
    ("TOCPHG", None),
    ("TOCPHD", None),
    ("VITK", Some("VITAMIN_K")),
    ("THIA", Some("VITAMIN_B1")),
    ("RIBF", Some("VITAMIN_B2")),
    ("NIA", Some("NIACIN")),
    ("NE", None),
    ("VITB6A", Some("VITAMIN_B6")),
    ("VITB12", Some("VITAMIN_B12")),
    ("FOL", Some("FOLIC_ACID")),
    ("PANTAC", Some("VITAMIN_B5")),
    ("BIOT", Some("VITAMIN_B7")),
    ("VITC", Some("VITAMIN_C")),
    ("WATER", None),
    ("REFUSE", None),
];

/// Amino-acid-table tag -> solver key. The combined rows (`AAS`, `AAA`, `AAT`)
/// are derived by the solver's own `nutrient_value`, so they are not mapped.
const AMINO_TAGS: &[(&str, Option<&str>)] = &[
    ("ILE", Some("ISOLEUCINE")),
    ("LEU", Some("LEUCINE")),
    ("LYS", Some("LYSINE")),
    ("MET", Some("METHIONINE")),
    ("CYS", Some("CYSTINE")),
    ("PHE", Some("PHENYLALANINE")),
    ("TYR", Some("TYROSINE")),
    ("THR", Some("THREONINE")),
    ("TRP", Some("TRYPTOPHAN")),
    ("VAL", Some("VALINE")),
    ("HIS", Some("HISTIDINE")),
    ("ARG", Some("ARGININE")),
    ("PROTCAA", None),
    ("PROT-", None),
    ("AAS", None),
    ("AAA", None),
    ("AAT", None),
    ("ALA", None),
    ("ASP", None),
    ("GLU", None),
    ("GLY", None),
    ("PRO", None),
    ("SER", None),
    ("HYP", None),
    ("AMMON", None),
    ("WATER", None),
];

/// Fatty-acid-table tag -> solver key. Only the chemically identified acids are
/// mapped; the saturated/monounsaturated/total columns are not, for the same
/// reason the other sources leave them out.
const FATTY_TAGS: &[(&str, Option<&str>)] = &[
    ("F18D2N6", Some("LINOLEIC_ACID")),
    ("F18D3N3", Some("ALPHA_LINOLENIC_ACID")),
    ("F20D4N6", Some("ARACHIDONIC_ACID")),
    ("F20D5N3", Some("EPA")),
    ("F22D6N3", Some("DHA")),
    ("FACID", None),
    ("FASAT", None),
    ("FAMS", None),
    ("FAPU", None),
    ("FAPUN3", None),
    ("FAPUN6", None),
];

/// One food read from the main workbook, with its amino and fatty rows merged
/// in from the other two.
pub struct MextFood {
    /// MEXT 食品番号, e.g. `01001`.
    pub number: String,
    /// MEXT 食品名, e.g. `こむぎ　［小麦粉］　強力粉　1等`.
    pub name: String,
    pub nutrients: Vec<(&'static str, f64)>,
}

/// Parse a MEXT cell into grams (or joules) per gram of food.
///
/// Returns `None` for the not-measured (`-`) and trace (`Tr`) markers, and for
/// any text that is not a number. A parenthesised estimate keeps its value.
fn cell_amount(value: &Data, unit_factor: f64) -> Option<f64> {
    let raw = match value {
        Data::Float(number) => return Some(number * unit_factor),
        Data::Int(number) => return Some(*number as f64 * unit_factor),
        Data::String(text) => text.trim().to_string(),
        _ => return None,
    };
    let text = raw.trim();
    if text.is_empty() || text == "-" || text.eq_ignore_ascii_case("tr") {
        return None;
    }
    // An estimated value is wrapped in parentheses.
    let text = text
        .strip_prefix('(')
        .and_then(|t| t.strip_suffix(')'))
        .unwrap_or(text);
    text.trim()
        .parse::<f64>()
        .ok()
        .map(|value| value * unit_factor)
}

/// Convert a MEXT `x/100 g` unit into a per-gram factor in base units.
///
/// `kJ/100 g` and `kcal/100 g` are energy, so their factor is in joules.
fn unit_factor(unit: &str) -> Option<f64> {
    let unit = unit.replace(' ', "");
    let unit = unit.strip_suffix("/100g")?;
    let base = match unit {
        "g" => G,
        "mg" => MG,
        "µg" | "μg" => MCG,
        "kJ" => KJ,
        "kcal" => KCAL,
        _ => return None,
    };
    Some(base / 100.0)
}

/// The column index of each tag, taken from the `成分識別子` row.
fn tag_columns(range: &calamine::Range<Data>) -> Option<std::collections::HashMap<String, usize>> {
    let tag_row = range.rows().position(|row| {
        row.iter()
            .any(|cell| matches!(cell, Data::String(text) if text.trim() == "成分識別子"))
    })?;
    let mut columns = std::collections::HashMap::new();
    for (index, cell) in range.rows().nth(tag_row)?.iter().enumerate() {
        if let Data::String(text) = cell {
            let tag = text.trim().to_string();
            if !tag.is_empty() {
                columns.insert(tag, index);
            }
        }
    }
    Some(columns)
}

/// The unit of each column, taken from the `単位` row.
fn unit_columns(range: &calamine::Range<Data>) -> Vec<Option<f64>> {
    let Some(row) = range.rows().find(|row| {
        row.iter()
            .any(|cell| matches!(cell, Data::String(text) if text.trim() == "単位"))
    }) else {
        return Vec::new();
    };
    row.iter()
        .map(|cell| match cell {
            Data::String(text) => unit_factor(text),
            _ => None,
        })
        .collect()
}

/// The food number and name a MEXT data row starts with.
fn row_identity(row: &[Data]) -> Option<(String, String)> {
    let number = match row.get(1)? {
        Data::String(text) if text.trim().chars().all(|c| c.is_ascii_digit()) => text.trim(),
        Data::Float(_) | Data::Int(_) => return None,
        _ => return None,
    };
    if number.len() != 5 {
        return None;
    }
    let name = match row.get(3)? {
        Data::String(text) if !text.trim().is_empty() => text.trim().to_string(),
        _ => return None,
    };
    Some((number.to_string(), name))
}

/// Read one sheet into its nutrient rows.
///
/// Shared by all three workbooks: they differ only in their tag table.
fn parse_sheet(
    bytes: &[u8],
    tags: &[(&str, Option<&'static str>)],
) -> anyhow::Result<Vec<MextFood>> {
    let mut workbook = Xlsx::new(Cursor::new(bytes)).context("open MEXT workbook")?;
    let sheet = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("MEXT workbook has no sheets"))?;
    let range = workbook
        .worksheet_range(&sheet)
        .with_context(|| format!("read MEXT sheet {sheet:?}"))?;

    let columns = tag_columns(&range).ok_or_else(|| anyhow!("MEXT sheet has no tag row"))?;
    let units = unit_columns(&range);

    // Resolve each mapped tag to its column and per-gram factor once.
    let mut mapped: Vec<(&'static str, usize, f64)> = Vec::new();
    for (tag, key) in tags {
        let Some(key) = key else {
            continue;
        };
        let Some(&column) = columns.get(*tag) else {
            continue;
        };
        let Some(Some(factor)) = units.get(column) else {
            continue;
        };
        // A food row may carry the tag under a trailing-space variant.
        let column = *columns
            .iter()
            .find(|(name, index)| name.trim() == tag.trim() && *index == &column)
            .map(|(_, index)| index)
            .unwrap_or(&column);
        mapped.push((key, column, *factor));
    }

    let mut foods = Vec::new();
    for row in range.rows() {
        let Some((number, name)) = row_identity(row) else {
            continue;
        };
        let mut nutrients = Vec::new();
        for (key, column, factor) in &mapped {
            if let Some(value) = row.get(*column).and_then(|cell| cell_amount(cell, *factor)) {
                nutrients.push((*key, value));
            }
        }
        foods.push(MextFood {
            number,
            name,
            nutrients,
        });
    }
    Ok(foods)
}

/// Read the main workbook (`_012`).
#[throws(Error)]
pub fn parse_main(bytes: &[u8]) -> Vec<MextFood> {
    parse_sheet(bytes, MAIN_TAGS)?
}

/// Read the amino-acid workbook (`_022`).
#[throws(Error)]
pub fn parse_amino(bytes: &[u8]) -> Vec<MextFood> {
    parse_sheet(bytes, AMINO_TAGS)?
}

/// Read the fatty-acid workbook (`_032`).
#[throws(Error)]
pub fn parse_fatty(bytes: &[u8]) -> Vec<MextFood> {
    parse_sheet(bytes, FATTY_TAGS)?
}

/// Merge amino-and-fatty rows into a main row, keyed by MEXT food number.
///
/// Later sources only add nutrients the row is missing, never replace one, so
/// the main table stays authoritative for the columns it carries.
pub fn merge(main: &mut MextFood, extra: &MextFood) {
    for (key, value) in &extra.nutrients {
        if !main.nutrients.iter().any(|(existing, _)| existing == key) {
            main.nutrients.push((*key, *value));
        }
    }
}

/// Download the three workbooks and return every food with its rows merged.
///
/// The workbooks are a few megabytes each and cover thousands of foods, so a
/// caller fetches them once per run and indexes the result by 食品番号.
#[throws(Error)]
pub fn fetch_all() -> Vec<MextFood> {
    let main = get(MAIN_URL)?;
    let amino = get(AMINO_URL)?;
    let fatty = get(FATTY_URL)?;

    let mut foods = parse_main(&main)?;
    let amino = parse_amino(&amino)?;
    let fatty = parse_fatty(&fatty)?;
    for extra in amino.iter().chain(fatty.iter()) {
        if let Some(index) = foods.iter().position(|food| food.number == extra.number) {
            merge(&mut foods[index], extra);
        }
    }

    foods
}

/// Fetch one URL into memory.
#[throws(Error)]
fn get(url: &str) -> Vec<u8> {
    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("GET {url}"))?;
    let mut bytes = Vec::new();
    std::io::copy(&mut response.body_mut().as_reader(), &mut bytes)
        .with_context(|| format!("read {url}"))?;
    bytes
}
