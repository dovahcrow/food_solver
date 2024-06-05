from collections import defaultdict
from typing import DefaultDict, Dict, List, Optional, Tuple, cast
import logging

import gurobipy as gp

from .units import MCG, MG, G
from .food import Food, get_or_load
from .nutrient import Nutrient


class RecipeSolver:
    m: gp.Model
    nutrients: Dict[Food, Dict[Nutrient, float]]
    variables: Dict[Food, gp.Var]
    provides: DefaultDict[Nutrient, gp.LinExpr | int]
    constraints: List[gp.LinExpr]

    def __init__(self) -> None:
        self.m = gp.Model("Recipe")
        self.nutrients = {}
        self.variables = {}
        self.provides = defaultdict(int)
        self.constraints = []

    def add_food(self, *foods: Tuple[Food, float, float]):
        for f, lb, ub in foods:
            if f not in self.nutrients:
                self.nutrients[f] = get_or_load(f)

            if f not in self.variables:
                self.variables[f] = self.m.addVar(name=f.name, lb=lb, ub=ub)

        for n in Nutrient:
            for f, *_ in foods:
                self.provides[n] += self.variables[f] * self.nutrients[f].get(n, 0)

    def finalize(self, needs: Dict[Nutrient, Tuple[float, Optional[float]]]):
        for n in Nutrient:
            if n not in needs:
                continue

            if isinstance(self.provides[n], int) and self.provides[n] == 0:
                logging.info(f"WARN: Food lacks {n}")
                continue

            lb = needs[n][0]
            ub = needs[n][1]
            k1 = 10 # Slope for nutrient out of range
            k2 = 0.001 # Slope for nutrient in the range but away from the optimal point
            if ub is not None:
                rg = [lb - 1, lb, (lb + ub) / 2, ub, ub + 1], [k1, 0, - k2 * (ub - lb) / 2, 0, 10]
            else:
                rg = [lb - 1, lb, lb + 1], [k1, 0, -k2]

            z = self.m.addVar(lb=0, name=n.name)
            self.m.addConstr(z == self.provides[n])
            self.m.setPWLObj(z, rg[0], rg[1])

        for c in self.constraints:
            self.m.addConstr(cast(gp.TempLConstr, c))

        self.m.write("problem.lp")

    def solve(self) -> int:
        self.m.optimize()
        logging.info(f"Objective: {self.m.ObjVal}")
        return self.m.Status

    def print_foods(self, day: float = 1):
        logging.info("Solution:")

        for k, v in self.variables.items():
            logging.info(f"  {k.name} = {v.X * day:.2f}")

    def print_nutrition(self, needs: Dict[Nutrient, Tuple[float, Optional[float]]], detail: bool = False):
        logging.info("Nutrition:")
        for n in Nutrient:
            if n not in needs:
                continue

            value = 0
            comp = []
            for f, v in self.variables.items():
                nut = self.nutrients[f].get(n, 0)
                if nut != 0:
                    comp.append((f, v.X * nut))
                value += v.X * nut

            lb = needs[n][0]
            ub = needs[n][1] or float("+INF")

            valid = lb <= value <= ub

            if n == Nutrient.ENERGY:
                scale = 1000
                unit = "kJ"
            else:
                if value >= G:
                    scale = G
                    unit = "g"
                elif value >= MG:
                    scale = MG
                    unit = "mg"
                else:
                    scale = MCG
                    unit = "μg"

            value /= scale
            lb /= scale
            ub /= scale

            if detail:
                comp_str = " = " + " + ".join([f"{f.name} {v / scale:g} {unit}" for f, v in comp])
            else:
                comp_str = ""

            if len(comp) != 1 or not detail:
                comp_str += f" = {value:g} {unit}"

            if valid:
                violate_str = ""
            else:
                violate_str = " [VIOLATE]"
                
            logging.info(f"  {n}{comp_str}, valid: {lb:.2f} ~ {ub:.2f} {unit}{violate_str}")
