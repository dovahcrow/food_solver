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
        # (Food.BAICAI, 200),
        # (Food.BROCCOLI, 90),
        # (Food.BELL_PEPER, 18),
        # (Food.EGGPLANT, 451),
        # (Food.CABBAGE, 90),
        (Food.CARROT, 395),
        # (Food.BOKCHOY, 60),
        # (Food.TOMATO, 919),
        (Food.CELERY, 446),
        # (Food.CHINESE_LETTUS, 50),
        # (Food.ZUCCHINI, 75),
        # (Food.CUCUMBER, 38),
        # (Food.PORK_FAT, 50),
        # (Food.LUOBO, 841),
        #
        # (Food.SOYBEAN_GREEN, 80),
        # (Food.SWEET_POTATO, 1041),
        # (Food.PUMPKIN, 1220),
        (Food.POTATO, 563),
        # (Food.BOCAI, 11),
        # (Food.BANANA, 50),
        #
        # (Food.PORK_HEART, 75),
        # (Food.EGG, 50),
        # (Food.OYSTER, 50),
        # (Food.PORK_TONGUE, 50),
        # (Food.BEEF, 27),
        (Food.CHICKEN_BREAST, 297),
        # (Food.CHICKEN_THIGH, 60),
        # (Food.CHICKEN_HEART, 37),
        # (Food.PORK_INTESTINE, 50),
        # (Food.PORK, 513),
        # (Food.PORK_LIVER, 40),
        # (Food.TOFU_FIRM, 419),
        # (Food.RICE, 20),
        #
        (Food.CANOLA_OIL, 10 * day),
        (Food.SALT, 2 * day),
        (Food.EGG_SHELL_POWDER, 2 * day),
        # (Food.BARF, 4),
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
