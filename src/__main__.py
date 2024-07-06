import logging

import click

from .recipe import RecipeSolver
from .needs import dog
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
        (Food.BAICAI, 0, 50),
        (Food.BROCCOLI, 0, 50),
        (Food.CARROT, 0, 50),
        (Food.CELERY, 0, 50),
        (Food.CHINESE_LETTUS, 0, 50),
        (Food.CUCUMBER, 0, 50),
        (Food.SOYBEAN_GREEN, 0, 70),

        (Food.SWEET_POTATO, 0, 150),
        (Food.POTATO, 0, 150),
        (Food.BANANA, 0, 50),

        (Food.PORK_HEART, 0, 100),
        (Food.EGG, 0, 50),
        # (Food.OYSTER, 0, 50),
        # (Food.PORK_TONGUE, 0, 50),
        (Food.BEEF, 0, 50),
        (Food.CHICKEN_BREAST, 0, 50),
        (Food.PORK_LIVER, 0, 50),

        (Food.CANOLA_OIL, 0, 10),
        (Food.SALT, 0, 2),
        (Food.BARF, 0, 4),
    ]
    needs = dog(age=2, weight=7, active=False)

    p = RecipeSolver()

    p.add_food(*foods)
    for nut, need in needs.items():
        p.add_need(nut, *need)
    optimal = p.solve()

    if optimal:
        p.print_foods(day=day)
        p.print_nutrition(needs, detail)
    else:
        print("Solution not found")


if __name__ == "__main__":
    main()
