//! China Food Composition Tables getter.
//!
//! `https://nlc.chinanutri.cn/fq/foodinfo/{id}.html` renders the food as an
//! HTML table. The food name is the `h1` inside `div.food_introduce_top`; each
//! nutrient is a row of `div.nutrition_table_content` whose label cell carries
//! the `td_left` class and whose next cell holds a value like `280kJ`.
//!
//! Values are per 100 g, so the result is divided by 100 to get grams per gram.

use anyhow::{anyhow, Context, Error};
use culpa::{throw, throws};
use scraper::{Html, Selector};

use super::nutrients::{CHINA_IGNORED, CHINA_NAMES};
use super::units::normalize;
use super::Row;

/// A label cell's class attribute contains the `td_left` class.
fn is_label(cell: &scraper::ElementRef) -> bool {
    cell.attr("class")
        .is_some_and(|classes| classes.split_whitespace().any(|class| class == "td_left"))
}

/// Parse one table cell's text, the way BeautifulSoup's `.text` concatenates
/// it: every descendant text node, with no separator.
fn cell_text(cell: &scraper::ElementRef) -> String {
    cell.text().collect()
}

/// Parse a `<value><unit>` cell such as `280kJ` into a base-unit amount.
fn parse_amount(text: &str) -> Option<f64> {
    let captures = super::AMOUNT_RE.captures(text)?;
    let amount: f64 = captures.get(1)?.as_str().parse().ok()?;
    normalize(amount, captures.get(3)?.as_str())
}

/// Parse the food page HTML into a nutrient row.
///
/// Split out from the network call so tests can exercise it on a fixture.
#[throws(Error)]
pub fn parse(body: &str) -> Row {
    let document = Html::parse_document(body);
    let title = Selector::parse("div.food_introduce_top > h1").expect("title selector");
    let row_selector = Selector::parse("div.nutrition_table_content tr").expect("row selector");
    let cell_selector = Selector::parse("td").expect("cell selector");

    let name = document
        .select(&title)
        .next()
        .map(|element| element.text().collect::<String>())
        .ok_or_else(|| anyhow!("food page has no title"))?;

    let mut nutrients: Vec<(&'static str, f64)> = Vec::new();
    for row in document.select(&row_selector).skip(1) {
        let cells: Vec<scraper::ElementRef> = row.select(&cell_selector).collect();
        // The label cell is the first one tagged `td_left`; the value is the
        // cell right after it. Rows without one carry nothing we map.
        let Some(index) = cells.iter().position(is_label) else {
            continue;
        };
        let label = cell_text(&cells[index]);
        let Some(value_cell) = cells.get(index + 1) else {
            continue;
        };
        let value = cell_text(value_cell);
        if value.is_empty() || CHINA_IGNORED.contains(&label.as_str()) {
            continue;
        }

        let Some((_, key)) = CHINA_NAMES.iter().find(|(known, _)| *known == label) else {
            throw!(anyhow!("China nutrient {label:?} is not in the name table"));
        };
        let Some(key) = key else {
            continue;
        };
        // A cell that is not `<number><unit>` is a footnote or a dash.
        let Some(amount) = parse_amount(&value) else {
            continue;
        };

        nutrients.push((key, amount / 100.0));
    }

    Row {
        name,
        nutrients,
        // China rows are mapped by direct lookup, so an unmapped label is an
        // error above rather than a skippable row.
        skipped: Vec::new(),
    }
}

/// Fetch one food by its China Food Composition Tables id.
#[throws(Error)]
pub fn get(id: u64) -> Row {
    let url = format!("https://nlc.chinanutri.cn/fq/foodinfo/{id}.html");
    let body = ureq::get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/120.0 Safari/537.36",
        )
        .call()
        .with_context(|| format!("GET {url}"))?
        .body_mut()
        .read_to_string()
        .with_context(|| format!("read {url}"))?;
    parse(&body)?
}
