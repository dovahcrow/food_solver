from .recipe import RecipeSolver
from .needs import dog
from .food import Food


def main():
    foods = {
        Food.SALT: (0, 15, 50000.0),
        Food.CHICKEN_BREAST: (30, 200, 30),
        Food.PORK_LIVER: (30, 100, 65),
        Food.BROCOLLI: (20, 200, 22),
        Food.CARROT: (20, 200, 25),
        Food.SWEET_POTATO: (10, 1000, 36),
        Food.CANOLA_OIL: (5, 10, 20),
        Food.LETTUS: (30, 100, 31),
        Food.BANANA: (10, 100, 31),
        Food.BARF: (0, 10, 1),
        # Food.BALANCEIT: (0, 10, 1),
    }

    p = RecipeSolver()
    p.add_food(*[(k, v[0], v[1]) for k, v in foods.items()])

    p.finalize(dog(age=2, weight=7, active=False, day=1), {k: v[2] for k, v in foods.items()})
    status = p.solve()

    if status == 1:
        print()
        p.print_foods()

        # print()
        # p.print_nutrients()

    else:
        print("Solution not found")

    # nuts = get_or_load("brocolli", "https://nlc.chinanutri.cn/fq/foodinfo/465.html")
    # print(nuts)


if __name__ == "__main__":
    main()
