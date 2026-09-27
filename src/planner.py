"""Shared recipe-planning library behind both the CLI and the MCP server.

The CLI (``python -m src opt``) and the MCP server are two frontends over the
same batch-planning flow defined here: ingredient weights plus a day count in,
a solved recipe and a per-day nutrient report out.

Food amounts are batch totals in grams. The day count scales the nutrient
requirements and the display basis; it never scales fixed ingredient weights.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Optional

import numpy as np

from .food import Food
from .needs import dog, scale
from .nutrient import Nutrient
from .recipe import NutrientReport, RecipeSolver
from .units import KCAL

# FEDIAF 2025 Table III-3b, adult maintenance: mass ratio Ca:P 1:1..2:1.
# Dimensionless: never scaled by the batch day count.
CA_P_RATIO = (1.0, 2.0)

MAX_DAYS = 3650


class PlannerError(ValueError):
    """Invalid planning input that the caller can fix."""


@dataclass
class IngredientSpec:
    """One ingredient of the batch.

    ``lower`` and ``upper`` are batch grams. Equal bounds pin a mandatory
    weight; ``lower == 0 < upper`` lets the solver choose freely up to
    ``upper``. ``minimize_usage`` prefers less of a free ingredient among
    otherwise equivalent recipes.
    """

    food: Food
    lower: float
    upper: float
    minimize_usage: bool = False
    optional: bool = False

    @classmethod
    def optional_upto(cls, food: Food, grams: float, minimize_usage: bool = False):
        return cls(food=food, lower=0.0, upper=grams,
                   minimize_usage=minimize_usage, optional=True)

    @classmethod
    def fixed(cls, food: Food, grams: float, minimize_usage: bool = False):
        return cls(food=food, lower=grams, upper=grams,
                   minimize_usage=minimize_usage, optional=False)


def resolve_food(name: str) -> Food:
    """Map a user-supplied name to a ``Food`` member.

    Accepts case-insensitive names, spaces or hyphens instead of underscores,
    and a leading ``Food.`` qualifier.
    """
    key = name.strip()
    if "." in key:
        key = key.rsplit(".", 1)[-1]
    key = key.upper().replace(" ", "_").replace("-", "_")
    try:
        return Food[key]
    except KeyError:
        raise PlannerError(
            f"Unknown food {name!r}; call list_foods for valid names"
        ) from None


@dataclass
class Profile:
    """Dog profile selecting the FEDIAF adult maintenance requirement table."""

    age: float = 3.0
    weight: float = 7.0
    active: bool = False
    daily_kcal: Optional[float] = None


@dataclass
class PlanRequest:
    """A full batch-planning request."""

    ingredients: List[IngredientSpec]
    days: int = 1
    profile: Profile = field(default_factory=Profile)
    implicit_soft_upper_multiplier: Optional[float] = 1.5
    detail: bool = False


@dataclass
class RecipeLine:
    """Solved grams of one ingredient."""

    food: Food
    grams: float
    grams_per_day: float
    optional: bool
    upper_bound: float


@dataclass
class PlanResult:
    """Outcome of a planning request, shared by the CLI and MCP frontends."""

    optimal: bool
    status: str
    objective: Optional[float]
    days: int
    recipe: List[RecipeLine]
    batch_grams: float
    nutrition: List[NutrientReport]
    attempted: List[IngredientSpec] = field(default_factory=list)

    def amount(self, food: Food) -> float:
        for line in self.recipe:
            if line.food == food:
                return line.grams
        raise KeyError(food)

    def nutrient(self, nutrient: Nutrient) -> Optional[NutrientReport]:
        for report in self.nutrition:
            if report.nutrient == nutrient:
                return report
        return None

    def energy_kcal_per_day(self) -> Optional[float]:
        report = self.nutrient(Nutrient.ENERGY)
        if report is None or not np.isfinite(report.value):
            return None
        return report.value * report.scale / KCAL


def build_solver(request: PlanRequest) -> RecipeSolver:
    """Create a solver holding the request's ingredient bounds."""
    if isinstance(request.days, bool) or not isinstance(request.days, int):
        raise PlannerError("days must be a positive integer")
    if not 1 <= request.days <= MAX_DAYS:
        raise PlannerError(f"days must be between 1 and {MAX_DAYS}")

    solver = RecipeSolver(
        implicit_soft_upper_multiplier=request.implicit_soft_upper_multiplier
    )
    seen = set()
    for spec in request.ingredients:
        if spec.food in seen:
            raise PlannerError(
                f"{spec.food.name} was listed more than once; merge its weights"
            )
        seen.add(spec.food)
        if not (np.isfinite(spec.lower) and np.isfinite(spec.upper)):
            raise PlannerError(f"Bounds for {spec.food.name} must be finite")
        if spec.lower < 0 or spec.upper < spec.lower:
            raise PlannerError(
                f"Invalid bounds for {spec.food.name}: "
                f"{spec.lower} to {spec.upper}"
            )
        solver.add_food(spec.food, spec.lower, spec.upper, spec.minimize_usage)
    if not seen:
        raise PlannerError("Add at least one ingredient")
    return solver


def requirements(request: PlanRequest) -> Dict[
    Nutrient, tuple
]:
    """Scaled FEDIAF requirements for the request's profile and day count."""
    return scale(
        dog(
            age=request.profile.age,
            weight=request.profile.weight,
            active=request.profile.active,
            daily_kcal=request.profile.daily_kcal,
        ),
        request.days,
    )


def attach_needs(solver: RecipeSolver, needs) -> None:
    for nutrient, need in needs.items():
        solver.add_need(nutrient, *need)
    solver.add_nutrient_ratio(Nutrient.CALCIUM, Nutrient.PHOSPHORUS, *CA_P_RATIO)


def plan(request: PlanRequest) -> PlanResult:
    """Solve a batch and collect its recipe and per-day nutrient report."""
    solver = build_solver(request)
    needs = requirements(request)
    attach_needs(solver, needs)
    optimal = solver.solve()

    if not optimal:
        # amount() is only valid for an optimal solve, so report the declared
        # bounds instead of fabricating a recipe.
        return PlanResult(
            optimal=False,
            status=solver.problem.status if solver.problem else "not_solved",
            objective=solver.sol["primal objective"],
            days=request.days,
            recipe=[],
            batch_grams=0.0,
            nutrition=[],
            attempted=list(request.ingredients),
        )

    recipe = [
        RecipeLine(
            food=food,
            grams=solver.amount(food),
            grams_per_day=solver.amount(food) / request.days,
            optional=solver.food_limits[food][0] == 0,
            upper_bound=solver.food_limits[food][1],
        )
        for food in solver.food_names
    ]
    return PlanResult(
        optimal=True,
        status=solver.problem.status,
        objective=solver.sol["primal objective"],
        days=request.days,
        recipe=recipe,
        batch_grams=float(sum(line.grams for line in recipe)),
        nutrition=solver.nutrition_report(needs, request.days, request.detail),
    )
