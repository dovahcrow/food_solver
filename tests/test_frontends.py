"""The shared planner plus the CLI and MCP frontends over it."""

import json
import unittest

from src import report
from src.food import Food
from src.mcp_server import Ingredient, plan_payload, spec_from_model
from src.planner import (
    IngredientSpec,
    PlanRequest,
    PlannerError,
    Profile,
    plan,
    resolve_food,
)

# Mirrors the reference batch the CLI uses when no --ingredient is given.
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


class McpPayloadTests(unittest.TestCase):
    def test_payload_serialises_recipe_and_nutrition(self):
        payload = plan_payload(reference_request(10))
        json.dumps(payload)  # must stay JSON-serialisable for the transport
        self.assertTrue(payload["optimal"])
        self.assertEqual(len(payload["recipe"]), 8)
        self.assertTrue(payload["nutrition"])
        self.assertGreater(payload["energy_per_day_kcal"], 0)
        self.assertIsInstance(payload["objective"], float)
        energy = next(n for n in payload["nutrition"] if n["nutrient"] == "ENERGY")
        self.assertIn("amount_per_day", energy)
        self.assertIn("status", energy)

    def test_ingredient_model_maps_optional_and_fixed(self):
        optional = spec_from_model(Ingredient(food="rice", grams=5, optional=True))
        self.assertEqual((optional.lower, optional.upper), (0.0, 5.0))
        self.assertTrue(optional.optional)

        fixed = spec_from_model(Ingredient(food="Food.EGG", grams=7))
        self.assertEqual((fixed.lower, fixed.upper), (7.0, 7.0))
        self.assertFalse(fixed.optional)

    def test_infeasible_payload_explains_the_attempt(self):
        payload = plan_payload(reference_request(1))
        json.dumps(payload)
        self.assertFalse(payload["optimal"])
        self.assertEqual(payload["recipe"], [])
        self.assertEqual(payload["nutrition"], [])
        self.assertEqual(len(payload["attempted_recipe"]), 8)
        self.assertIn("infeasible", payload["hint"])


if __name__ == "__main__":
    unittest.main()
