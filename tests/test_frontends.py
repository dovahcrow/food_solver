"""The shared planner plus the Python CLI frontend over it.

MCP lives only in Rust now (`rust/food-mcp`), so its Python-side payload tests
were removed with the module they covered.
"""

import unittest

import click

from src import report
from src.food import Food
from src.__main__ import parse_ingredient
from src.planner import (
    IngredientSpec,
    PlanRequest,
    PlannerError,
    Profile,
    plan,
    resolve_food,
)

# The reference batch, pinned so the planner tests are reproducible.
FIXED = [(Food.CELERY, 245.0), (Food.JIANGDOU, 411.0), (Food.PORK, 500.0)]


def reference_request(days: int, **profile) -> PlanRequest:
    ingredients = [IngredientSpec.fixed(food, grams) for food, grams in FIXED]
    ingredients += [
        IngredientSpec.optional_upto(Food.RICE, 1000.0 * days, minimize_usage=True),
        IngredientSpec.optional_upto(Food.CANOLA_OIL, 5.0 * days, minimize_usage=True),
        IngredientSpec.optional_upto(Food.SALT, 2.0 * days, minimize_usage=True),
        IngredientSpec.optional_upto(
            Food.EGG_SHELL_POWDER, 5.0 * days, minimize_usage=True
        ),
        IngredientSpec.optional_upto(Food.EGG, 100.0 * days, minimize_usage=True),
    ]
    return PlanRequest(
        ingredients=ingredients,
        days=days,
        profile=Profile(**{"weight": 7.0, "age": 3.0, **profile}),
    )


class ResolveFoodTests(unittest.TestCase):
    def test_accepts_common_spellings(self):
        for name in ("CHICKEN_BREAST", "chicken breast", "chicken-breast",
                     " Food.Chicken_Breast "):
            with self.subTest(name=name):
                self.assertEqual(resolve_food(name).name, "CHICKEN_BREAST")

    def test_dotted_prefix_is_stripped(self):
        self.assertEqual(resolve_food("Food.RICE"), Food.RICE)

    def test_unknown_food_is_a_planner_error(self):
        with self.assertRaises(PlannerError):
            resolve_food("NOT_A_FOOD")


class PlanTests(unittest.TestCase):
    def test_reference_batch_solves(self):
        result = plan(reference_request(10))
        self.assertTrue(result.optimal, result.status)
        self.assertEqual({line.food for line in result.recipe}, {food for food, _ in FIXED}
                         | {Food.RICE, Food.CANOLA_OIL, Food.SALT,
                            Food.EGG_SHELL_POWDER, Food.EGG})
        self.assertGreater(result.batch_grams, 0.0)
        self.assertTrue(result.nutrition)

    def test_fixed_weights_ignore_the_day_count(self):
        # Only the requirements scale with days, so pinned ingredients keep the
        # exact batch weight while their per-day figure tracks the day count.
        for days in (5, 20):
            with self.subTest(days=days):
                result = plan(reference_request(days))
                self.assertTrue(result.optimal, result.status)
                for food, grams in FIXED:
                    self.assertAlmostEqual(result.amount(food), grams, places=6)
                line = next(l for l in result.recipe if l.food == Food.CELERY)
                self.assertAlmostEqual(line.grams_per_day, 245.0 / days, places=6)

    def test_duplicate_ingredient_is_rejected(self):
        request = PlanRequest(
            ingredients=[
                IngredientSpec.fixed(Food.RICE, 10.0),
                IngredientSpec.fixed(Food.RICE, 20.0),
            ]
        )
        with self.assertRaises(PlannerError):
            plan(request)

    def test_infeasible_result_carries_no_recipe(self):
        result = plan(reference_request(1))
        self.assertFalse(result.optimal)
        self.assertEqual(result.recipe, [])
        self.assertEqual(result.nutrition, [])
        self.assertEqual(len(result.attempted), 8)


class CliOutputTests(unittest.TestCase):
    def test_text_report_has_both_sections(self):
        result = plan(reference_request(10))
        self.assertTrue(result.optimal)
        lines = report.format_recipe(result) + report.format_nutrition(result)
        self.assertEqual(lines[0], "Solution:")
        self.assertIn("Nutrition (per day):", lines)
        self.assertTrue(any("Food.CELERY" in line for line in lines))

    def test_infeasible_render_is_the_cli_notice(self):
        self.assertEqual(report.render(plan(reference_request(1))), "Solution not found")


if __name__ == "__main__":
    unittest.main()

class ParseIngredientTests(unittest.TestCase):
    """The `-i` value accepts fixed, optional and minimize suffixes."""

    def test_bare_grams_is_optional(self):
        spec = parse_ingredient("PORK:500")
        self.assertEqual((spec.lower, spec.upper), (0.0, 500.0))
        self.assertFalse(spec.minimize_usage)

    def test_optional_opens_the_lower_bound(self):
        spec = parse_ingredient("PORK:500:optional")
        self.assertEqual((spec.lower, spec.upper), (0.0, 500.0))
        self.assertFalse(spec.minimize_usage)

    def test_minimize_is_optional_with_a_preference(self):
        for suffix in ("minimize", "min"):
            with self.subTest(suffix=suffix):
                spec = parse_ingredient(f"PORK:500:{suffix}")
                self.assertEqual((spec.lower, spec.upper), (0.0, 500.0))
                self.assertTrue(spec.minimize_usage)

    def test_explicit_fixed(self):
        spec = parse_ingredient("PORK:500:fixed")
        self.assertEqual((spec.lower, spec.upper), (500.0, 500.0))
        self.assertFalse(spec.minimize_usage)

    def test_unknown_suffix_is_rejected(self):
        with self.assertRaises(click.BadParameter):
            parse_ingredient("PORK:500:weird")

    def test_unknown_food_is_rejected(self):
        with self.assertRaises(click.BadParameter):
            parse_ingredient("NOPE:10")

    def test_non_numeric_grams_is_rejected(self):
        with self.assertRaises(click.BadParameter):
            parse_ingredient("PORK:abc")
