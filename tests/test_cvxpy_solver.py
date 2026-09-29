import unittest
from collections import defaultdict
from contextlib import redirect_stdout
from io import StringIO

import cvxpy as cp

from src.food import Food
from src.nutrient import Nutrient
from src.recipe import NeedRequired, NeedSoftness, RecipeSolver


class CvxpySolverTests(unittest.TestCase):
    def recipe(self):
        p = RecipeSolver()
        p.food_limits = {Food.RICE: (0., 100.)}
        p.food_nutrients = {Food.RICE: defaultdict(float, {Nutrient.ZINC: .001})}
        p.food_minimize_usage = {Food.RICE: False}
        return p

    def test_native_model_is_convex_qp_and_compatible_result(self):
        p = self.recipe()
        p.add_need(Nutrient.ZINC, .01, .02, NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertTrue(p.problem.is_dcp())
        self.assertTrue(p.problem.is_qp())
        self.assertEqual(p.problem.solver_stats.solver_name, 'CLARABEL')
        self.assertEqual(p.sol['status'], cp.OPTIMAL)
        self.assertEqual(p.sol['primal objective'], p.problem.value)
        self.assertEqual(len(p.sol['x']), 1)  # no auxiliary variables exposed
        self.assertAlmostEqual(p.problem.value, 0., delta=1e-6)

    def test_failed_resolve_does_not_expose_stale_food_amounts(self):
        p = self.recipe()
        with self.assertRaises(RuntimeError):
            p.amount(0)
        p.add_need(Nutrient.ZINC, .01, .02, NeedRequired.REQUIRED, NeedSoftness.HARD)
        self.assertTrue(p.solve())
        p.food_limits[Food.RICE] = (0., 1.)
        self.assertFalse(p.solve())
        self.assertEqual(p.problem.status, cp.INFEASIBLE)
        self.assertIsNone(p.sol['x'])
        with self.assertRaises(RuntimeError):
            p.amount(Food.RICE)

    def test_empty_model(self):
        with self.assertRaises(ValueError):
            RecipeSolver().solve()

    def test_soft_open_ended_need_gets_configurable_soft_upper(self):
        # A SOFT need may use the implicit preference. Fixed supply is 20 mg;
        # with L=10 mg and default U=1.5*L=15 mg, relative excess is 1/3 and
        # its squared penalty is 1/9.
        p = self.recipe()
        p.food_limits[Food.RICE] = (20., 20.)
        p.add_need(Nutrient.ZINC, .01, None,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertAlmostEqual(p.problem.value, 1 / 9, delta=1e-6)

        # None restores the genuinely open-ended behavior.
        p = RecipeSolver(implicit_soft_upper_multiplier=None)
        p.food_limits = {Food.RICE: (20., 20.)}
        p.food_nutrients = {
            Food.RICE: defaultdict(float, {Nutrient.ZINC: .001})
        }
        p.food_minimize_usage = {Food.RICE: False}
        p.add_need(Nutrient.ZINC, .01, None,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertEqual(p.problem.value, 0.)

    def test_hard_open_ended_need_gets_no_implicit_soft_upper(self):
        # A HARD need without an explicit maximum is genuinely open-ended: the
        # implicit preference only ranks soft recipes, so it must not apply.
        # Supply is 20 mg against L=10 mg, yet the excess costs nothing.
        p = self.recipe()
        p.food_limits[Food.RICE] = (20., 20.)
        p.add_need(Nutrient.ZINC, .01, None,
                   NeedRequired.REQUIRED, NeedSoftness.HARD)
        self.assertTrue(p.solve())
        self.assertEqual(p.problem.value, 0.)

    def test_invalid_implicit_soft_upper_multiplier(self):
        for multiplier in (0., 1., float("inf"), float("nan")):
            with self.subTest(multiplier=multiplier), self.assertRaises(ValueError):
                RecipeSolver(implicit_soft_upper_multiplier=multiplier)

    def test_report_distinguishes_below_inside_and_above(self):
        cases = (
            (5., "\033[31m", "below minimum", "10.00 ~ 15.00 mg"),
            (12., "\033[92m", "within range", "10.00 ~ 15.00 mg"),
            (20., "\033[33m", "above maximum", "10.00 ~ 15.00 mg"),
        )
        for amount, color, status, displayed_range in cases:
            with self.subTest(amount=amount):
                p = self.recipe()
                p.food_limits[Food.RICE] = (amount, amount)
                p.add_need(Nutrient.ZINC, .01, None,
                           NeedRequired.REQUIRED, NeedSoftness.SOFT)
                self.assertTrue(p.solve())
                output = StringIO()
                with redirect_stdout(output):
                    p.print_nutrition(p.needs)
                line = output.getvalue().splitlines()[1]
                self.assertTrue(line.startswith(color))
                self.assertIn(status, line)
                self.assertIn(displayed_range, line)
                self.assertIn("implicit soft upper 1.5x", line)

    def test_report_uses_explicit_upper_without_implicit_label(self):
        p = self.recipe()
        p.food_limits[Food.RICE] = (30., 30.)
        p.add_need(Nutrient.ZINC, .01, .02,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        output = StringIO()
        with redirect_stdout(output):
            p.print_nutrition(p.needs)
        line = output.getvalue().splitlines()[1]
        self.assertTrue(line.startswith("\033[33m"))
        self.assertIn("above maximum: 10.00 ~ 20.00 mg", line)
        self.assertNotIn("implicit soft upper", line)

    def test_detailed_components_are_reported_per_day(self):
        # A four-day batch supplies 3.98524 g total, or 0.99631 g/day.
        # The status and displayed component must use the same daily basis.
        p = RecipeSolver()
        p.food_limits = {Food.EGG: (272.9616438356164, 272.9616438356164)}
        p.food_nutrients = {
            Food.EGG: defaultdict(float, {Nutrient.LINOLEIC_ACID: .0146})
        }
        p.food_minimize_usage = {Food.EGG: False}
        p.add_need(Nutrient.LINOLEIC_ACID, 1.56 * 4, None,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())

        output = StringIO()
        with redirect_stdout(output):
            p.print_nutrition(p.needs, day=4, detail=True)
        line = output.getvalue().splitlines()[1]
        self.assertTrue(line.startswith("\033[31m"))
        self.assertIn("EGG 0.99631 g", line)
        self.assertIn("below minimum: 1.56 ~ 2.34 g", line)
        self.assertNotIn("EGG 3.98524 g", line)
