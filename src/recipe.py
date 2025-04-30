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
    food_minimize_usage: Dict[Food, bool]
    food_names: List[Food]
    needs: Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]

    def __init__(self) -> None:
        self.food_limits = {}
        self.food_nutrients = {}
        self.needs = {}
        self.food_names = []
        self.food_minimize_usage = {}

    def add_food(self, food: Food, lb: float, ub: float, minimize_usage: bool = False):
        if food not in self.food_nutrients:
            self.food_nutrients[food] = get_or_load(food)
            self.food_limits[food] = (lb, ub)
            self.food_minimize_usage[food] = minimize_usage

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

    def solve(self) -> int:
        self.food_names = foods = list(self.food_limits.keys())

        G = np.zeros((len(foods) * 2, len(foods)))
        h = np.zeros(len(foods) * 2)

        for i, food in enumerate(foods):
            lb, ub = self.food_limits[food]
            # The food should be bigger then lb, aka -food <= -lb
            G[i, i] = -1
            h[i] = -lb

            # The food should be less then ub, aka food <= ub
            G[i + len(foods), i] = 1
            h[i + len(foods)] = ub

        P = np.zeros((len(foods), len(foods)))
        q = np.zeros((1, len(foods)))

        # In QP's form minimize \frac12 x^TPx + q^Tx
        #            subject to Gx <= h
        #                       Ax = b
        #
        # total f types of food, n types of nutrient
        # x_j: amount of food j, m_i: mid of lb and ub nutrient i
        #
        # H_{ij} the food j contains how much nutrient i
        # Minimize \sum_{i=0..n} (\sum_{j=0..f} H_{ij}x_j - m_i)^2
        # i.e. Minimize \sum_{i=0..n} (\sum_{j=0..f,k=0..f} H_{ij}x_jH_{ik}x_k - 2 m_i \sum_{j=0..f}H_{ij}x_j + m_i^2)
        #
        # let P = \sum_{i=0..n} P_i
        # let q = \sum_{i=0..n} q_i
        #
        # i.e. P_i = 2 H_i^T H_i
        #      q_i = - 2 m_i H_i
        #
        # Normalized:
        # Minimize (\sum_{j=0..f} \frac{H_{ij}}{m_i}x_j - 1)^2
        # i.e. Minimize \sum_{j=0..f,k=0..f} \frac{H_{ij}H_{ik}}{m_i^2}x_jx_k - 2 \sum_{j=0}^{f}H_{ij}x_j + 1
        # i.e. P_i = 2 \frac{H_i^T H_i}{m_i^2}
        #      q_i = - 2 H_i
        for need, (lb, ub, required, hard) in self.needs.items():
            # if isinstance(self.provides[n], int) and self.provides[n] == 0:
            #     logging.info(f"WARN: Food lacks {n}")
            #     continue
            if required != NeedRequired.REQUIRED:
                continue

            H = np.asarray([self.food_nutrients[food][need] for food in foods])

            if hard == NeedSoftness.HARD:
                # The nutrient should be bigger then lb, aka -has <= -lb
                G = np.vstack([G, -H])
                h = np.append(h, -lb)
                if ub is not None:
                # The nutrient should be less then ub
                    G = np.vstack([G, H])
                    h = np.append(h, ub)
            else:
                if ub is None:
                    mid = lb * 1.05
                else:
                    mid = (lb + ub) / 2

                H = H[None, :]
                P += 2 * H.T @ H / mid / mid
                q += -2 * H
                # P += H.T @ H
                # q += H * -mid

        for i, food in enumerate(foods):
            if self.food_minimize_usage[food]:
                P[i, i] += 1
            
        G = cvxopt.matrix(G)
        h = cvxopt.matrix(h)
        P = cvxopt.matrix(P.T)
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
