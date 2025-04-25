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
    foods_hard = [
        # (Food.BAICAI, 200),
        # (Food.BANANA, 50),
        (Food.BEEF, 400),
        # (Food.BASA_FISH, 484),
        # (Food.BEEN_SPROUT, 497),
        # (Food.BELL_PEPER, 18),
        # (Food.BOCAI, 270),
        # (Food.BOKCHOY, 60),
        # (Food.BROCCOLI, 443),
        # (Food.CABBAGE, 521),
        (Food.CARROT, 300),
        # (Food.CELERY, 446),
        # (Food.CHICKEN_BREAST, 251),
        # (Food.CHICKEN_HEART, 37),
        # (Food.CHICKEN_THIGH, 60),
        # (Food.CHINESE_LETTUS, 50),
        # (Food.CUCUMBER, 38),
        # (Food.EGG, 50),
        # (Food.EGGPLANT, 451),
        # (Food.LUOBO, 868),
        # (Food.WHITE_MUSHROOM, 148),
        # (Food.WINTER_MELON, 415),
        # (Food.OYSTER, 50),
        # (Food.PORK, 528),
        # (Food.PORK_FAT, 50),
        # (Food.PORK_HEART, 75),
        # (Food.PORK_INTESTINE, 50),
        # (Food.PORK_LIVER, 40),
        # (Food.PORK_TONGUE, 50),
        # (Food.POTATO, 1100),
        # (Food.PUMPKIN, 504),
        # (Food.SHANYAO, 600),
        # (Food.SOYBEAN_GREEN, 80),
        (Food.SWEET_POTATO, 480),
        # (Food.TOFU_FIRM, 690),
        (Food.TOMATO, 450),
        (Food.ZUCCHINI, 360),
        #
    ]
    foods_opts =[
        (Food.RICE, 500),
        (Food.CANOLA_OIL, 10 * day),
        (Food.SALT, 2 * day),
        (Food.EGG_SHELL_POWDER, 2 * day),
        # (Food.BARF, 4),
    ]
    needs = dog(age=2, weight=7, active=False)
    needs = scale(needs, day)

    p = RecipeSolver()

    for food, ub in foods_hard:
        p.add_food((food, ub, ub))
    for food, ub in foods_opts:
        p.add_food((food, 0, ub))
    for nut, need in needs.items():
        p.add_need(nut, *need)
    optimal = p.solve()

    if optimal:
        p.print_foods()
        p.print_nutrition(needs, day, detail)
    else:
        print("Solution not found")


if __name__ == "__main__":
    main()
