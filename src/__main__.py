import logging

import click

from .recipe import RecipeSolver
from .needs import dog, scale
from .food import Food

logging.basicConfig(
    format="[%(asctime)s %(name)s %(levelname)s] %(message)s",
    encoding="utf-8",
    level=logging.INFO,
)


@click.group()
def main():
    pass


@main.command()
@click.option("-d", "--day", required=False, type=int, default="1")
@click.option("--detail", required=False, type=bool, default=False)
def opt(day: int, detail: bool):
    foods = [
        (Food.BAICAI, 0, 200),
        # (Food.BROCCOLI, 0, 90),
        # (Food.BELL_PEPER, 0, 18),
        (Food.EGGPLANT, 0, 451),
        # (Food.CABBAGE, 0, 90),
        (Food.CARROT, 0, 332),
        # (Food.BOKCHOY, 0, 60),
        (Food.TOMATO, 0, 919),
        # (Food.CELERY, 0, 50),
        # (Food.CHINESE_LETTUS, 0, 50),
        # (Food.ZUCCHINI, 0, 75),
        # (Food.CUCUMBER, 0, 38),
        # (Food.PORK_FAT, 0, 50),
        (Food.LUOBO, 0, 841),
        #
        # (Food.SOYBEAN_GREEN, 0, 80),
        (Food.SWEET_POTATO, 0, 1041),
        (Food.PUMPKIN, 0, 1220),
        (Food.POTATO, 0, 1242),
        # (Food.BOCAI, 0, 11),
        # (Food.BANANA, 0, 50),
        #
        # (Food.PORK_HEART, 0, 75),
        # (Food.EGG, 0, 50),
        # (Food.OYSTER, 0, 50),
        # (Food.PORK_TONGUE, 0, 50),
        # (Food.BEEF, 0, 27),
        (Food.CHICKEN_BREAST, 0, 419),
        # (Food.CHICKEN_THIGH, 0, 60),
        # (Food.CHICKEN_HEART, 0, 37),
        # (Food.PORK_INTESTINE, 0, 50),
        (Food.PORK, 0, 513),
        # (Food.PORK_LIVER, 0, 40),
        (Food.TOFU_FIRM, 0, 419),
        # (Food.RICE, 0, 20),
        #
        (Food.CANOLA_OIL, 0, 10 * day),
        (Food.SALT, 0, 2 * day),
        (Food.EGG_SHELL_POWDER, 0, 2 * day),
        # (Food.BARF, 0, 4),
    ]
    needs = dog(age=2, weight=7, active=False)
    needs = scale(needs, day)

    p = RecipeSolver()

    p.add_food(*foods)
    for nut, need in needs.items():
        p.add_need(nut, *need)
    optimal = p.solve(weight={Food.BARF: 0})

    if optimal:
        p.print_foods()
        p.print_nutrition(needs, day, detail)
    else:
        print("Solution not found")


if __name__ == "__main__":
    main()
