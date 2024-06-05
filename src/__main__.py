import logging

import click
from gurobipy import GRB

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
    # foods = [
    #     (Food.SALT, 1.88, 1.88),
    #     (Food.CHICKEN_BREAST, 82, 82),
    #     (Food.BROCCOLI, 20, 39),
    #     (Food.CARROT, 20, 28),
    #     (Food.SWEET_POTATO, 100, 238),
    #     (Food.CANOLA_OIL, 11, 11),
    #     # (Food.SOYBEAN, 0, 100),
    #     (Food.BALANCEIT, 0, 5.62),
    # ]
    foods = [
        (Food.SALT, 0, 1.88),
        (Food.CHICKEN_BREAST, 0, 200),
        (Food.PORK_LIVER, 0, 100),
        (Food.BROCCOLI, 0, 500),
        (Food.CARROT, 0, 500),
        (Food.SWEET_POTATO, 0, 300),
        (Food.CANOLA_OIL, 0, 11),
        (Food.CHINESE_LETTUS, 0, 200),
        (Food.BANANA, 0, 700),
        (Food.CELERY, 0, 700),
        (Food.EGG, 0, 300),
        (Food.SOY_MILK, 0, 500),
        (Food.BARF, 0, 4),
        # (Food.SOYBEAN, 0, 100),
        # (Food.BALANCEIT, 0, 5.62),
    ]
    needs = dog(age=2, weight=7, active=False)

    p = RecipeSolver()

    p.add_food(*foods)
    p.finalize(needs)
    status = p.solve()

    if status == GRB.OPTIMAL:
        p.print_foods(day=day)
        p.print_nutrition(needs, detail)
    else:
        print("Solution not found")



if __name__ == "__main__":
    main()
