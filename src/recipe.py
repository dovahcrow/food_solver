from collections import defaultdict
from typing import DefaultDict, Dict, List, Optional, Tuple, cast

from pulp import GUROBI_CMD, PULP_CBC_CMD, LpAffineExpression, LpMaximize, LpProblem, LpVariable

from .units import MCG, MG, G
from .food import Food, get_or_load
from .nutrient import Nutrient


class RecipeSolver:
    nutrients: Dict[Food, Dict[Nutrient, float]]
    variables: Dict[Food, LpVariable]
    provides: DefaultDict[Nutrient, LpAffineExpression]
    constraints: List[LpAffineExpression]

    def __init__(self) -> None:
        self.nutrients = {}
        self.variables = {}
        self.provides = defaultdict(lambda: LpAffineExpression(constant=0))
        self.constraints = []

    def add_food(self, *foods: Tuple[Food, float, float]):
        for f, lb, ub in foods:
            if f not in self.nutrients:
                self.nutrients[f] = get_or_load(f)

            if f not in self.variables:
                self.variables[f] = LpVariable(f.name, lowBound=lb, upBound=ub)

        for n in Nutrient:
            for f, *_ in foods:
                self.provides[n] += self.variables[f] * self.nutrients[f].get(n, 0)

    def finalize(self, needs: Dict[Nutrient, Tuple[float, Optional[float]]], scores: Dict[Food, float]) -> LpProblem:
        prob = LpProblem("recipe", LpMaximize)
        prob += sum(scores[k] * v for k, v in self.variables.items())

        for n in Nutrient:
            if n not in needs:
                continue

            if self.provides[n].isNumericalConstant():
                print(f"WARN: Food lacks {n}")
                continue

            prob += needs[n][0] <= self.provides[n]

            upper_bound = needs[n][1]
            if upper_bound is not None:
                prob += self.provides[n] <= upper_bound

        for c in self.constraints:
            prob += c

        prob.writeLP("problem.lp")

        self.prob = prob
        return prob

    def solve(self) -> int:
        status = self.prob.solve(GUROBI_CMD(msg=True))
        return status

    def print_foods(self):
        print("Solution:\n")
        for v in self.variables.values():
            print(f"{v.name} = {v.varValue}")

    def print_nutrients(self):
        print("Nutrient profile:")
        for n in Nutrient:
            value = sum(cast(float, v.varValue) * self.nutrients[f].get(n, 0) for f, v in self.variables.items())
            if value == 0:
                continue
            if n == Nutrient.ENERGY:
                value /= 1000
                unit = "kJ"
            else:
                if value >= G:
                    value /= G
                    unit = "g"
                elif value >= MG:
                    value /= MG
                    unit = "mg"
                else:
                    value /= MCG
                    unit = "μg"
            print(f"{n} = {value:.2f} {unit}")
