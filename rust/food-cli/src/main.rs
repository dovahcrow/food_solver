//! Command-line frontend for the food solver.
//!
//! |Flow| Batch ingredient weights plus a day count go in; a solved recipe and
//! its per-day nutrient report come out.
//!
//! The solver is the default command, so `food -d 10 -i RICE:500` solves a
//! batch directly. `food fetch` refreshes the `foods/*.json` caches the table
//! is built from, ported from the Python getters.

mod fetch;
mod format;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Error};
use clap::{ArgAction, Parser, Subcommand};
use culpa::{throw, throws};
use food_core::{
    parse_food, plan, IngredientSpec, NutrientReport, PlanRequest, Profile, ReportStatus, FOODS,
    FOOD_NAMES,
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
    long_about = "Solve a batch from the ingredients given with --ingredient, or \
                  refresh the food data the solver embeds with `food fetch`.\n\n\
                  With no --ingredient the solve command reproduces the historical \
                  CELERY, JIANGDOU, PORK, RICE, CANOLA_OIL, SALT, EGG_SHELL_POWDER \
                  and EGG batch."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Command>,

    #[command(flatten)]
    solve: CmdSolve,
}

#[derive(Subcommand)]
enum Command {
    /// Solve a batch (the default when no subcommand is given).
    Solve(CmdSolve),
    /// Download food nutrient caches from USDA and the China Food tables.
    Fetch(CmdFetch),
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
    #[arg(short = 'i', long = "ingredient", value_name = "FOOD:GRAMS[:optional]")]
    ingredient: Vec<String>,

    /// List every food the solver knows and exit.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    foods: bool,
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
        Some(Command::Fetch(fetch)) => fetch.run()?,
        Some(Command::Solve(solve)) => solve.run()?,
        // No subcommand: the historical bare invocation is a solve.
        None => cli.solve.run()?,
    }
}

impl CmdFetch {
    #[throws(Error)]
    fn run(&self) {
        if self.list {
            for entry in fetch::CATALOG {
                match entry.source {
                    fetch::Source::Usda(id) => println!("{} usda {id}", entry.name),
                    fetch::Source::Chinanutri(id) => {
                        println!("{} chinanutri {id}", entry.name);
                    }
                    fetch::Source::Inline => println!("{} inline", entry.name),
                }
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
            match fetch::refresh(entry, &directory) {
                Ok(row) => {
                    eprintln!(
                        "    {} -> {} ({} nutrients)",
                        row.name,
                        entry.name,
                        row.nutrients.len()
                    );
                    if !row.skipped.is_empty() {
                        eprintln!("    skipped unmapped rows: {}", row.skipped.join(", "));
                    }
                }
                // One bad page must not sink the whole run: report and continue.
                Err(error) => eprintln!("    error: {error:#}"),
            }
        }
    }
}

impl CmdSolve {
    #[throws(Error)]
    fn run(&self) {
        if self.foods {
            for name in FOOD_NAMES {
                println!("{name}");
            }
            return;
        }

        let ingredients = if self.ingredient.is_empty() {
            default_ingredients(self.day)
        } else {
            let mut specs = Vec::new();
            for raw in &self.ingredient {
                specs.push(parse_ingredient(raw)?);
            }
            specs
        };

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

/// Parse `FOOD:GRAMS[:optional|minimize|fixed]`.
#[throws(Error)]
fn parse_ingredient(spec: &str) -> IngredientSpec {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        throw!(anyhow!(
            "{spec:?} must be FOOD:GRAMS or FOOD:GRAMS:optional"
        ));
    }
    let food = parse_food(parts[0])
        .ok_or_else(|| anyhow!("Unknown food {:?}; run `food --foods`", parts[0]))?;
    let grams: f64 = parts[1]
        .parse()
        .map_err(|_| anyhow!("{:?} is not a number of grams", parts[1]))?;
    if !grams.is_finite() || grams < 0.0 {
        throw!(anyhow!("grams must be a non-negative number"));
    }
    if parts.len() == 2 {
        return IngredientSpec::fixed(food, grams);
    }
    match parts[2].to_ascii_lowercase().as_str() {
        "optional" | "opt" | "true" | "1" | "yes" => IngredientSpec::optional_upto(food, grams),
        "minimize" | "min" => IngredientSpec::minimize(food, grams),
        "fixed" | "false" | "0" | "no" => IngredientSpec::fixed(food, grams),
        other => throw!(anyhow!("{other:?} is not optional, minimize or fixed")),
    }
}

/// The reference batch used when no --ingredient is given.
fn default_ingredients(day: u32) -> Vec<IngredientSpec> {
    let scale = f64::from(day);
    vec![
        IngredientSpec::fixed("CELERY", 245.0),
        IngredientSpec::fixed("JIANGDOU", 411.0),
        IngredientSpec::fixed("PORK", 500.0),
        IngredientSpec::minimize("RICE", 1000.0 * scale),
        IngredientSpec::minimize("CANOLA_OIL", 5.0 * scale),
        IngredientSpec::minimize("SALT", 2.0 * scale),
        IngredientSpec::minimize("EGG_SHELL_POWDER", 5.0 * scale),
        IngredientSpec::minimize("EGG", 100.0 * scale),
    ]
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
        format!("; incomplete data: {}", report.missing.join(", "))
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

/// Keep the embedded table linked even when only names are printed.
#[allow(dead_code)]
fn _touch() -> usize {
    FOODS.len()
}
