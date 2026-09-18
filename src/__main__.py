import logging

import click

from .food import Food, get_or_load
from .needs import dog, scale
from .nutrient import Nutrient
from .recipe import RecipeSolver

logging.basicConfig(
    format="[%(asctime)s %(name)s %(levelname)s] %(message)s",
    encoding="utf-8",
    level=logging.INFO,
)


@click.group()
def main():
    pass


@main.command("refresh-foods")
@click.argument(
    "foods", nargs=-1, required=True, type=click.Choice([f.name for f in Food])
)
def refresh_foods(foods):
    """Re-fetch named foods so older caches can include newly mapped nutrients."""
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
def opt(day: int, detail: bool, daily_kcal: float | None = None):
    foods_hard = [
        # (Food.BAICAI, 925),
        # (Food.JUANXINCAI, 661),
        # (Food.BANANA, 50),
        # (Food.BEEF, 635),
        # (Food.BASA_FISH, 484),
        # (Food.BEEN_SPROUT, 497),
        # (Food.BELL_PEPER, 427),
        # (Food.BOCAI, 253),
        # (Food.BOKCHOY, 60),
        # (Food.BROCCOLI, 933),
        # (Food.CABBAGE, 1006),
        # (Food.CARROT, 1147),
        (Food.CELERY, 245),
        # (Food.CHICKEN_BREAST, 665),
        # (Food.CHICKEN_HEART, 37),
        # (Food.CHICKEN_LIVER, 424),
        # (Food.CHICKEN_GIZZARD, 278),
        # (Food.CHICKEN_THIGH, 60),
        # (Food.CHINESE_LETTUS, 50),
        # (Food.CUCUMBER, 854),
        # (Food.EGG, 100),
        # (Food.EGGPLANT, 601),
        # (Food.FUGUA, 1438),
        # (Food.LUOBO, 863),
        (Food.JIANGDOU, 411),
        # (Food.JIEGUA, 664),
        # (Food.KUIGUA, 824),
        # (Food.KONGXINCAI, 439),
        # (Food.WHITE_MUSHROOM, 339),
        # (Food.WINTER_MELON, 415),
        # (Food.OYSTER, 50),
        (Food.PORK, 500),
        # (Food.PORK_FAT, 50),
        # (Food.PORK_HEART, 396),
        # (Food.PORK_INTESTINE, 50),
        # (Food.PORK_LIVER, 275),
        # (Food.PORK_TONGUE, 50),
        # (Food.POTATO, 1787),
        # (Food.PUMPKIN, 1182),
        # (Food.QINCAI, 315),
        # (Food.SIJIDOU, 308),
        # (Food.SIGUA, 650),
        # (Food.SHANYAO, 600),
        # (Food.SHITAKE, 222),
        # (Food.SOYBEAN_GREEN, 203),
        # (Food.SWEET_POTATO, 1486),
        # (Food.TOFU_FIRM, 467),
        # (Food.TOFU_SOFT, 660),
        # (Food.TOMATO, 1363),
        # (Food.DUCK_GIZZARD, 568),
        # (Food.ZIGANLAN, 719),
        # (Food.ZUCCHINI, 575),
        #
    ]
    foods_opts = [
        (Food.RICE, 1000 * day),
        (Food.CANOLA_OIL, 5 * day),
        (Food.SALT, 2 * day),
        (Food.EGG_SHELL_POWDER, 5 * day),
        (Food.EGG, 100 * day),
        # (Food.BARF, 4 * day),
    ]
    needs = dog(age=3, weight=7, active=False, daily_kcal=daily_kcal)
    needs = scale(needs, day)

    p = RecipeSolver()

    for food, ub in foods_hard:
        p.add_food(food, ub, ub)
    for food, ub in foods_opts:
        p.add_food(food, 0, ub, True)
    for nut, need in needs.items():
        p.add_need(nut, *need)
    # FEDIAF 2025 Table III-3b, adult maintenance: mass ratio Ca:P 1:1..2:1.
    # Dimensionless: do not scale these bounds by the batch's number of days.
    p.add_nutrient_ratio(Nutrient.CALCIUM, Nutrient.PHOSPHORUS, 1.0, 2.0)
    optimal = p.solve()

    if optimal:
        p.print_foods()
        p.print_nutrition(needs, day, detail)
    else:
        print("Solution not found")


if __name__ == "__main__":
    main()
