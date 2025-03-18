from enum import Enum, auto
from typing import Dict, List, Optional, Tuple
import numpy as np
import cvxopt

from .units import MCG, MG, G
from .food import Food, get_or_load
from .nutrient import Nutrient, Nutrients

# https://cvxopt.org/userguide/coneprog.html#quadratic-programming


class NeedSoftness(Enum):
    SOFT = auto()
    HARD = auto()


class NeedRequired(Enum):
    REQUIRED = auto()
    NOT_REQUIRED = auto()


class RecipeSolver:
    food_limits: Dict[Food, Tuple[float, float]]
    food_nutrients: Dict[Food, Nutrients]
    food_names: List[Food]
    needs: Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]

    def __init__(self) -> None:
        self.food_limits = {}
        self.food_nutrients = {}
        self.needs = {}
        self.food_names = []

    def add_food(self, *foods: Tuple[Food, float]):
        for f, ub in foods:
            if f not in self.food_nutrients:
                self.food_nutrients[f] = get_or_load(f)
                self.food_limits[f] = (0, ub)

    def add_need(
        self,
        nut: Nutrient,
        lb: float,
        ub: Optional[float],
        required: NeedRequired,
        hard: NeedSoftness,
    ):
        if nut in self.needs:
            return

        self.needs[nut] = (lb, ub, required, hard)

    def solve(self, weight: Dict[Food, float] = {}) -> int:
        self.food_names = foods = list(self.food_limits.keys())

        G = np.zeros((len(foods) * 2, len(foods)))
        h = np.zeros(len(foods) * 2)

        for i, food in enumerate(foods):
            lb, ub = self.food_limits[food]
            G[i, i] = -1
            h[i] = -lb

            G[i + len(foods), i] = 1
            h[i + len(foods)] = ub

        P = np.zeros((len(foods), len(foods)))
        q = np.zeros((1, len(foods)))

        for need, (lb, ub, required, hard) in self.needs.items():
            # if isinstance(self.provides[n], int) and self.provides[n] == 0:
            #     logging.info(f"WARN: Food lacks {n}")
            #     continue
            if required != NeedRequired.REQUIRED:
                continue

            has = np.asarray([self.food_nutrients[food].get(need, 0) for food in foods])

            if hard == NeedSoftness.HARD:
                G = np.vstack([G, -has])
                h = np.append(h, -lb)
                if ub is not None:
                    G = np.vstack([G, has])
                    h = np.append(h, ub)
            else:
                if ub is None:
                    mid = lb * 1.05
                else:
                    mid = (lb + ub) / 2

                has = has[None, :]
                P += has.T @ has
                q += has * -mid

        w = np.asarray([weight.get(food, 0) for food in foods])
        G = cvxopt.matrix(G)
        h = cvxopt.matrix(h)
        P = cvxopt.matrix(P.T + np.diag(w ** 2))
        q = cvxopt.matrix(q.T)

        self.sol = cvxopt.solvers.qp(P, q, G, h, options={"show_progress": False})

        return self.sol["status"] == "optimal"

    def print_foods(self):
        print("Solution:")

        for i, f in enumerate(self.food_names):
            amount = float(f"{self.amount(i):.2g}")
            print(f"  {f} = {amount:.1f}")

    def amount(self, food: Food | int):
        if isinstance(food, Food):
            food = self.food_names.index(food)

        return self.sol["x"][food]

    def print_nutrition(
        self,
        needs: Dict[
            Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]
        ],
        day: int = 1,
        detail: bool = False,
    ):
        print("Nutrition:")
        for n in Nutrient:
            if n not in needs:
                continue

            lb, ub, required, _ = needs[n]
            ub = ub or float("+INF")

            value = 0
            comp = []
            for i, f in enumerate(self.food_names):
                nut = self.food_nutrients[f].get(n, 0)
                if nut != 0:
                    comp.append((f, self.amount(i) * nut))
                value += self.amount(i) * nut
            lb /= day
            ub /= day
            value /= day

            if n == Nutrient.ENERGY:
                scale = 1000
                unit = "kJ"
            else:
                if lb >= G:
                    scale = G
                    unit = "g"
                elif lb >= MG:
                    scale = MG
                    unit = "mg"
                else:
                    scale = MCG
                    unit = "μg"

            value /= scale
            lb /= scale
            ub /= scale

            valid = lb <= value <= ub

            if detail:
                comp_str = " = " + " + ".join(
                    [f"{f.name} {v / scale:g} {unit}" for f, v in comp if v != 0]
                )
            else:
                comp_str = ""

            if len(comp) != 1 or not detail:
                comp_str += f" = {value:g} {unit}"

            color = ""
            if required != NeedRequired.REQUIRED:
                if not valid:
                    color = BColors.LIGHT_YELLOW
                else:
                    color = BColors.GRAY
            elif not valid:
                color = BColors.RED
            else:
                color = BColors.LIGHT_GREEN

            print(
                f"{color}  {n}{comp_str}, valid: {lb:.2f} ~ {ub:.2f} {unit}{BColors.ENDC}"
            )


class BColors:
    HEADER = "\033[95m"
    RED = "\033[31m"
    YELLOW = "\033[33m"
    GRAY = "\033[37m"
    OKBLUE = "\033[94m"
    OKCYAN = "\033[96m"
    GREEN = "\033[32m"
    LIGHT_GREEN = "\033[92m"
    LIGHT_YELLOW = "\033[93m"
    LIGHT_RED = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"
