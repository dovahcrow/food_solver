"""Text rendering of a solved batch for the CLI frontend."""

from __future__ import annotations

from typing import List

from .planner import PlanResult
from .recipe import BColors, NutrientReport


def format_grams(grams: float) -> float:
    """Match the CLI's two-significant-digit rounding of a food amount."""
    return float(f"{grams:.2g}")


def format_recipe(result: PlanResult) -> List[str]:
    lines = ["Solution:"]
    for line in result.recipe:
        lines.append(f"  {line.food} = {format_grams(line.grams):.1f}")
    return lines


def format_nutrition(result: PlanResult, detail: bool = False) -> List[str]:
    lines = ["Nutrition (per day):"]
    for report in result.nutrition:
        lines.append(format_nutrient_line(report, detail))
    return lines


def format_nutrient_line(report: NutrientReport, detail: bool) -> str:
    if detail:
        comp_str = " = " + " + ".join(
            f"{name} {value:g} {report.unit}" for name, value in report.components
        )
    else:
        comp_str = ""
    if not report.has_single_component or not detail:
        comp_str += f" = {report.value:g} {report.unit}"

    coverage = f"; incomplete data: {', '.join(report.missing)}" if report.missing else ""
    upper_source = (
        f"; implicit soft upper {report.implicit_multiplier:g}x"
        if report.implicit_upper
        else ""
    )
    return (
        f"{report.color}  {report.nutrient}{comp_str}, {report.status}: "
        f"{report.minimum:.2f} ~ {report.maximum:.2f} {report.unit}"
        f"{upper_source}{coverage}{BColors.ENDC}"
    )


def render(result: PlanResult) -> str:
    """Full CLI text for a plan, or the infeasibility notice."""
    if not result.optimal:
        return "Solution not found"
    lines = format_recipe(result) + format_nutrition(result)
    return "\n".join(lines)
