"""Command-line frontend for the food solver.

The planning logic lives in :mod:`src.planner`; this module only parses
arguments and renders the result.
"""

import logging

import click

from . import report
from .food import Food
from .planner import (
    IngredientSpec,
    PlanRequest,
    PlannerError,
    Profile,
    plan,
    resolve_food,
)

logging.basicConfig(
    format="[%(asctime)s %(name)s %(levelname)s] %(message)s",
    encoding="utf-8",
    level=logging.INFO,
)


def parse_ingredient(value: str) -> IngredientSpec:
    """Parse ``FOOD:GRAMS[:optional|minimize|fixed]`` into a spec.

    The upper bound is the batch weight. ``fixed`` (the default) pins the
    weight exactly; ``optional`` allows any amount from 0 up to it; ``minimize``
    is optional and additionally prefers less of it among equally good recipes.
    ``minimize`` is only meaningful for an ingredient that may be left out, so
    it behaves like ``optional`` plus the preference.
    """
    parts = value.split(":")
    if len(parts) not in (2, 3):
        raise click.BadParameter(
            f"{value!r} must be FOOD:GRAMS or FOOD:GRAMS:optional"
        )
    try:
        food = resolve_food(parts[0])
    except PlannerError as error:
        raise click.BadParameter(str(error)) from None
    try:
        grams = float(parts[1])
    except ValueError:
        raise click.BadParameter(f"{parts[1]!r} is not a number of grams") from None
    if grams < 0:
        raise click.BadParameter("grams must not be negative")
    if len(parts) == 2:
        return IngredientSpec.fixed(food, grams)
    match parts[2].lower():
        case "optional" | "opt" | "true" | "1" | "yes":
            return IngredientSpec.optional_upto(food, grams)
        case "minimize" | "min":
            return IngredientSpec.optional_upto(food, grams, minimize_usage=True)
        case "fixed" | "false" | "0" | "no":
            return IngredientSpec.fixed(food, grams)
        case other:
            raise click.BadParameter(f"{other!r} is not optional, minimize or fixed")


@click.group()
def main():
    pass


@main.command("foods")
def foods_command():
    """List every food the solver knows."""
    for food in Food:
        click.echo(food.name)


@main.command("refresh-foods")
@click.argument(
    "foods", nargs=-1, required=True, type=click.Choice([f.name for f in Food])
)
def refresh_foods(foods):
    """Re-fetch named foods so older caches can include newly mapped nutrients."""
    from .food import get_or_load

    for name in foods:
        get_or_load(Food[name], refresh=True)
        click.echo(f"Refreshed {name}")


@main.command()
@click.option("-d", "--day", type=click.IntRange(min=1), default=1)
@click.option("--detail", required=False, type=bool, default=False)
@click.option(
    "--daily-kcal",
    type=click.FloatRange(min=0, min_open=True),
    default=None,
    help="Override the adult dog daily energy estimate (kcal, before mixing).",
)
@click.option("--weight", type=click.FloatRange(min=0, min_open=True), default=7.0,
              help="Dog body weight in kg.")
@click.option("--age", type=click.FloatRange(min=1), default=3.0,
              help="Dog age in years; only adult maintenance is supported.")
@click.option("--active/--no-active", default=False,
              help="Select the 110 kcal/kg^0.75 profile instead of 95.")
@click.option(
    "-i", "--ingredient", "ingredients", multiple=True, required=True,
    metavar="FOOD:GRAMS[:optional]",
    help="Add an ingredient as FOOD:GRAMS[:optional|minimize|fixed]. A fixed "
         "weight must be used exactly; an optional one may be any amount up to "
         "GRAMS; `minimize` marks an optional ingredient to use minimally. "
         "Repeat for every ingredient; at least one is required.",
)
def opt(day, detail, daily_kcal, weight, age, active, ingredients):
    """Solve a batch from the ingredients given with --ingredient."""
    specs = [parse_ingredient(value) for value in ingredients]

    request = PlanRequest(
        ingredients=specs,
        days=day,
        profile=Profile(age=age, weight=weight, active=active, daily_kcal=daily_kcal),
        detail=detail,
    )
    try:
        result = plan(request)
    except PlannerError as error:
        raise click.ClickException(str(error)) from None

    if not result.optimal:
        click.echo("Solution not found")
        return
    # color=True keeps the report's ANSI codes even when piped, matching the
    # historical plain-print output.
    for line in report.format_recipe(result) + report.format_nutrition(result, detail):
        click.echo(line, color=True)


if __name__ == "__main__":
    main()
