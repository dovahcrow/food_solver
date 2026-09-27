//! Command-line frontend for the food solver.
//!
//! |Flow| Batch ingredient weights plus a day count go in; a solved recipe and
//! its per-day nutrient report come out.

mod format;

use std::process::ExitCode;

use anyhow::{anyhow, Error};
use clap::{ArgAction, Parser};
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

#[derive(Parser, Debug)]
#[command(
    name = "food",
    about = "Solve dog food batches from ingredient amounts and a day count.",
    long_about = "Solve a batch from the ingredients given with --ingredient.\n\n\
                  With no --ingredient the CLI reproduces the historical CELERY, \
                  JIANGDOU, PORK, RICE, CANOLA_OIL, SALT, EGG_SHELL_POWDER and EGG batch."
)]
struct Cli {
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
        unit = report.unit,
        reset = color::RESET,
    )
}

#[throws(Error)]
fn run(cli: Cli) {
    if cli.foods {
        for name in FOOD_NAMES {
            println!("{name}");
        }
        return;
    }

    let ingredients = if cli.ingredient.is_empty() {
        default_ingredients(cli.day)
    } else {
        let mut specs = Vec::new();
        for raw in &cli.ingredient {
            specs.push(parse_ingredient(raw)?);
        }
        specs
    };

    let request = PlanRequest {
        ingredients,
        days: cli.day,
        profile: Profile {
            age: cli.age,
            weight: cli.weight,
            active: cli.active,
            daily_kcal: cli.daily_kcal,
        },
        implicit_soft_upper_multiplier: Some(1.5),
        detail: cli.detail,
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
        println!("{}", format_nutrient(report, cli.detail));
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
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
