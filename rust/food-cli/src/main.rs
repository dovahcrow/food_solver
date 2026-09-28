//! Command-line frontend for the food solver.
//!
//! |Flow| Batch ingredient weights plus a day count go in; a solved recipe and
//! its per-day nutrient report come out.
//!
//! `food solve -d 10 -i RICE:500` solves a batch; `food fetch` refreshes the
//! `foods/*.json` caches the table is built from, ported from the Python
//! getters.

mod fetch;
mod format;

use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use anyhow::{anyhow, Context, Error};
use clap::{ArgAction, Parser, Subcommand};
use culpa::{throw, throws};
use food_core::{
    plan, FoodName, IngredientSpec, NutrientReport, PlanRequest, Profile, ReportStatus,
};

/// ANSI colours, matching the Python frontend so output stays comparable.
mod color {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const GRAY: &str = "\x1b[37m";
    pub const LIGHT_GREEN: &str = "\x1b[92m";
    pub const LIGHT_YELLOW: &str = "\x1b[93m";
}

#[derive(Parser)]
#[command(
    name = "food",
    about = "Solve dog food batches and refresh the food database.",
    long_about = "Solve a batch with `food solve`, or refresh the food data the \
                  solver embeds with `food fetch`."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Solve a batch from ingredient amounts and a day count.
    Solve(CmdSolve),
    /// Download food nutrient caches from USDA and the China Food tables.
    Fetch(CmdFetch),
    /// Expand the bulk SR Legacy JSON into per-food cache files.
    ExpandSrLegacy(CmdExpandSrLegacy),
}

/// Solve a batch from ingredient amounts and a day count.
#[derive(Parser, Debug)]
struct CmdSolve {
    /// Number of days the batch covers; scales the requirements.
    #[arg(short = 'd', long = "day", default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    day: u32,

    /// Include each food's contribution to every nutrient.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    detail: bool,

    /// Override the adult dog daily energy estimate (kcal, before mixing).
    #[arg(long, value_parser = clap::value_parser!(f64))]
    daily_kcal: Option<f64>,

    /// Dog body weight in kg.
    #[arg(long, default_value_t = 7.0)]
    weight: f64,

    /// Dog age in years; only adult maintenance is supported.
    #[arg(long, default_value_t = 3.0)]
    age: f64,

    /// Select the 110 kcal/kg^0.75 profile instead of 95.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    active: bool,

    /// Add an ingredient as FOOD:GRAMS[:optional]. A fixed weight must be used
    /// exactly; an optional one may be any amount up to GRAMS. Repeat per
    /// ingredient; `minimize` marks an optional ingredient to use minimally.
    #[arg(
        short = 'i',
        long = "ingredient",
        value_name = "FOOD:GRAMS[:optional]",
        required = true
    )]
    ingredient: Vec<IngredientInput>,
}

/// Fetch food nutrient caches.
#[derive(Parser, Debug)]
struct CmdFetch {
    /// Foods to refresh, e.g. PORK BEEF. Case-insensitive; default is all.
    #[arg(value_name = "FOOD")]
    foods: Vec<String>,

    /// Cache directory; overrides FOODS_DIR.
    #[arg(long, value_name = "DIR")]
    dir: Option<PathBuf>,

    /// Print what would be fetched without writing anything.
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// List the fetchable foods and exit.
    #[arg(long, default_value_t = false)]
    list: bool,
}

#[throws(Error)]
fn main_body(cli: Cli) {
    match cli.cmd {
        Command::Solve(solve) => solve.run()?,
        Command::Fetch(fetch) => fetch.run()?,
        Command::ExpandSrLegacy(expand) => expand.run()?,
    }
}

impl CmdFetch {
    #[throws(Error)]
    fn run(&self) {
        if self.list {
            for entry in fetch::CATALOG {
                let source = match entry.source {
                    fetch::Source::Usda(id) => format!("usda {id}"),
                    fetch::Source::Chinanutri(id) => format!("chinanutri {id}"),
                    fetch::Source::Inline => "inline".to_string(),
                };
                // Secondary sources are reported next to the primary one.
                let mut extras = Vec::new();
                if let Some(id) = fetch::catalog::sr_legacy_id(entry.name) {
                    extras.push(format!("sr-legacy {id}"));
                }
                if let Some(number) = fetch::catalog::mext_number(entry.name) {
                    extras.push(format!("mext {number}"));
                }
                let suffix = if extras.is_empty() {
                    String::new()
                } else {
                    format!(" + {}", extras.join(" + "))
                };
                let chosen = FoodName::parse(entry.name)
                    .map(|food| food.source().name())
                    .unwrap_or("?");
                println!("{} {source}{suffix} -> choose {chosen}", entry.name);
            }
            return;
        }

        let wanted: Vec<&str> = self.foods.iter().map(String::as_str).collect();
        let (mut chosen, unknown) = fetch::catalog::select(wanted);
        if !unknown.is_empty() {
            throw!(anyhow!(
                "unknown food(s): {}; run `food fetch --list`",
                unknown.join(", ")
            ));
        }
        // Without explicit names, everything except the inline-only foods.
        if chosen.is_empty() {
            chosen = fetch::CATALOG
                .iter()
                .filter(|entry| entry.source != fetch::Source::Inline)
                .collect();
        }

        let directory = self.dir.clone().unwrap_or_else(fetch::foods_dir);
        let total = chosen.len();
        for (index, entry) in chosen.iter().enumerate() {
            if self.dry_run {
                println!("[{}/{}] {} (dry run)", index + 1, total, entry.name);
                continue;
            }
            eprintln!("[{}/{}] fetching {}", index + 1, total, entry.name);
            // Every source the food has: the primary row, then any curated
            // SR Legacy and MEXT records. Each lands in its own file.
            // Each source lands in its own file; a failure on one is
            // reported and the rest still run.
            match fetch::fetch_primary(entry, &directory) {
                Ok(file) => report_file(&file),
                Err(error) => eprintln!("    error: {error:#}"),
            }
            match fetch::fetch_sr_legacy(entry, &directory) {
                Ok(Some(file)) => report_file(&file),
                Ok(None) => {}
                Err(error) => eprintln!("    error: {error:#}"),
            }
            if fetch::catalog::mext_number(entry.name).is_some() {
                // MEXT is one bulk download, so a failure there is reported
                // without stopping the per-food sources.
                match fetch::load_mext() {
                    Ok(foods) => match fetch::fetch_mext(entry, &foods, &directory) {
                        Ok(Some(file)) => report_file(&file),
                        Ok(None) => {}
                        Err(error) => eprintln!("    error: {error:#}"),
                    },
                    Err(error) => eprintln!("    error: {error:#}"),
                }
            }
        }
    }
}

/// Report one written cache file on stderr.
fn report_file(file: &fetch::CacheFile) {
    eprintln!(
        "    {}: {} ({} nutrients)",
        file.source.name(),
        file.row.name,
        file.row.nutrients.len()
    );
    if !file.row.skipped.is_empty() {
        eprintln!(
            "    {}: skipped unmapped rows: {}",
            file.source.name(),
            file.row.skipped.join(", ")
        );
    }
}

/// Expand the bulk SR Legacy dataset into per-food cache files.
#[derive(Parser, Debug)]
struct CmdExpandSrLegacy {
    /// The `FoodData_Central_sr_legacy_food_json_*.json` file to expand.
    #[arg(value_name = "JSON")]
    json: PathBuf,

    /// Cache directory; overrides FOODS_DIR.
    #[arg(long, value_name = "DIR")]
    dir: Option<PathBuf>,

    /// Print what would be written without writing anything.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

impl CmdExpandSrLegacy {
    #[throws(Error)]
    fn run(&self) {
        let payload = std::fs::read_to_string(&self.json)
            .with_context(|| format!("read {}", self.json.display()))?;
        let directory = self.dir.clone().unwrap_or_else(fetch::foods_dir);

        // The dataset names foods as USDA text, so match each catalog entry to
        // its curated SR Legacy record by description to recover the `{FOOD}`
        // part of the file name.
        let owned: Vec<(String, u64)> = fetch::CATALOG
            .iter()
            .filter_map(|entry| {
                fetch::catalog::sr_legacy_id(entry.name).map(|id| (entry.name.to_string(), id))
            })
            .collect();

        if self.dry_run {
            println!(
                "would expand {} into {}/",
                self.json.display(),
                directory.display()
            );
            for (name, _) in &owned {
                println!("  {name}_SrLegacy.json");
            }
            return;
        }
        let written = fetch::expand_sr_legacy(&payload, &owned, &directory)?;
        for file in &written {
            println!("wrote {}", directory.join(file).display());
        }
        println!("{} file(s) written", written.len());
    }
}

impl CmdSolve {
    #[throws(Error)]
    fn run(&self) {
        let ingredients: Vec<IngredientSpec> =
            self.ingredient.iter().map(|item| item.spec()).collect();

        let request = PlanRequest {
            ingredients,
            days: self.day,
            profile: Profile {
                age: self.age,
                weight: self.weight,
                active: self.active,
                daily_kcal: self.daily_kcal,
            },
            implicit_soft_upper_multiplier: Some(1.5),
            detail: self.detail,
        };
        let result = plan(&request)?;

        if !result.optimal {
            println!("Solution not found");
            return;
        }

        println!("Solution:");
        for line in &result.recipe {
            // Python shows two significant digits of the batch grams.
            println!(
                "  {} = {:.1}",
                line.food,
                format::two_significant(line.grams)
            );
        }
        println!("Nutrition (per day):");
        for report in &result.nutrition {
            println!("{}", format_nutrient(report, self.detail));
        }
    }
}

/// One `-i` argument: `FOOD:GRAMS[:optional|minimize|fixed]`.
///
/// Parsed by clap through [`FromStr`], so a `CmdSolve` holds ingredients
/// already understood rather than raw strings.
#[derive(Debug, Clone, Copy)]
struct IngredientInput {
    food: FoodName,
    grams: f64,
    kind: IngredientKind,
}

/// How an ingredient's weight is constrained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngredientKind {
    /// The weight must be used exactly.
    Fixed,
    /// Any amount from 0 up to the weight.
    Optional,
    /// Optional, and prefer less of it among otherwise equivalent recipes.
    Minimize,
}

impl IngredientInput {
    /// The solver spec this argument describes.
    fn spec(self) -> IngredientSpec {
        match self.kind {
            IngredientKind::Fixed => IngredientSpec::fixed(self.food, self.grams),
            IngredientKind::Optional => IngredientSpec::optional_upto(self.food, self.grams),
            IngredientKind::Minimize => IngredientSpec::minimize(self.food, self.grams),
        }
    }
}

impl FromStr for IngredientInput {
    type Err = String;

    /// Parse `FOOD:GRAMS[:optional|minimize|fixed]`.
    fn from_str(spec: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(format!(
                "{spec:?} must be FOOD:GRAMS or FOOD:GRAMS:optional"
            ));
        }
        let food = FoodName::parse(parts[0])
            .ok_or_else(|| format!("Unknown food {:?}; run `food fetch --list`", parts[0]))?;
        let grams: f64 = parts[1]
            .parse()
            .map_err(|_| format!("{:?} is not a number of grams", parts[1]))?;
        if !grams.is_finite() || grams < 0.0 {
            return Err("grams must be a non-negative number".to_string());
        }
        let kind = if parts.len() == 2 {
            IngredientKind::Fixed
        } else {
            match parts[2].to_ascii_lowercase().as_str() {
                "optional" | "opt" | "true" | "1" | "yes" => IngredientKind::Optional,
                "minimize" | "min" => IngredientKind::Minimize,
                "fixed" | "false" | "0" | "no" => IngredientKind::Fixed,
                other => return Err(format!("{other:?} is not optional, minimize or fixed")),
            }
        };
        Ok(Self { food, grams, kind })
    }
}

fn color_for(report: &NutrientReport) -> &'static str {
    let required = report.required == food_core::NeedRequired::Required;
    match (required, report.status) {
        (false, ReportStatus::BelowMinimum) => color::LIGHT_YELLOW,
        (false, ReportStatus::AboveMaximum) => color::YELLOW,
        (false, ReportStatus::WithinRange) => color::GRAY,
        (true, ReportStatus::BelowMinimum) => color::RED,
        (true, ReportStatus::AboveMaximum) => color::YELLOW,
        (true, ReportStatus::WithinRange) => color::LIGHT_GREEN,
    }
}

/// One nutrient line, matching the Python frontend's formatting.
fn format_nutrient(report: &NutrientReport, detail: bool) -> String {
    // Python's "%.2g" style: two significant digits for the raw values.
    let mut text = String::new();
    if detail {
        let parts: Vec<String> = report
            .components
            .iter()
            .map(|component| {
                format!(
                    "{} {} {}",
                    component.food,
                    format::g(component.amount),
                    report.unit
                )
            })
            .collect();
        text = format!(" = {}", parts.join(" + "));
    }
    if report.components.len() != 1 || !detail {
        text.push_str(&format!(" = {} {}", format::g(report.value), report.unit));
    }

    let coverage = if report.missing.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = report.missing.iter().map(|food| food.name()).collect();
        format!("; incomplete data: {}", names.join(", "))
    };
    let upper_source = match (report.implicit_upper, report.implicit_multiplier) {
        (true, Some(multiplier)) => format!("; implicit soft upper {multiplier}x"),
        _ => String::new(),
    };
    let maximum = match report.maximum {
        Some(value) => format::fixed2(value),
        None => "inf".to_string(),
    };

    format!(
        "{color}  {nutrient}{text}, {status}: {minimum} ~ {maximum} {unit}{upper_source}{coverage}{reset}",
        color = color_for(report),
        nutrient = report.nutrient.name(),
        status = report.status.as_str(),
        minimum = format::fixed2(report.minimum),
        maximum = maximum,
        unit = report.unit,
        upper_source = upper_source,
        coverage = coverage,
        reset = color::RESET,
    )
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match main_body(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
