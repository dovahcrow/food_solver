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
    foods = [
        (Food.SALT, 0, 2),
        (Food.BEEF, 0, 50),
        (Food.PORK_LIVER, 0, 50),
        (Food.BROCCOLI, 0, 50),
        (Food.EGG, 0, 50),
        (Food.CARROT, 0, 50),
        (Food.CUCUMBER, 0, 50),
        (Food.SWEET_POTATO, 0, 300),
        (Food.CANOLA_OIL, 0, 10),
        (Food.BARF, 0, 4),
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
