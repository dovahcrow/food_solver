import unittest
from collections import defaultdict

from src.food import Food
from src.needs import dog, scale
from src.nutrient import Nutrient
from src.recipe import NeedRequired, NeedSoftness, RecipeSolver
from src.units import G, MG, MCG, KCAL


class ObjectiveTests(unittest.TestCase):
    def solve_amount(self, unit=1., days=1, penalize=False, upper=None):
        p = RecipeSolver()
        p.food_limits = {Food.RICE: (0., 100. * days)}
        p.food_nutrients = {Food.RICE: defaultdict(float, {Nutrient.ZINC: 0.001 * unit})}
        p.food_minimize_usage = {Food.RICE: penalize}
        p.add_need(Nutrient.ZINC, 0.01 * unit * days,
                   upper * unit * days if upper is not None else None,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        return p.amount(0)

    def test_single_target_and_unit_invariance(self):
        for unit in (G, MG, MCG, 1000.):
            with self.subTest(unit=unit):
                self.assertAlmostEqual(self.solve_amount(unit, penalize=True),
                                       10 / 1.0005, places=3)

    def test_bounded_soft_target(self):
        # Every amount from 10 to 30 meets the interval; no midpoint target.
        amount = self.solve_amount(upper=0.03)
        self.assertGreaterEqual(amount, 10. - 1e-3)
        self.assertLessEqual(amount, 30. + 1e-3)

    def test_interval_cost_uses_each_boundary_as_denominator(self):
        # Synthetic density 0.001 g/g, with an allowed interval 10..100 mg.
        for amount, expected in ((9., .01), (10., 0.), (20., 0.), (55., 0.),
                                 (100., 0.), (101., .0001), (110., .01)):
            for unit, days in ((1., 1), (1000., 1), (1., 10)):
                with self.subTest(amount=amount, unit=unit, days=days):
                    p = RecipeSolver()
                    p.food_limits = {Food.RICE: (amount * days, amount * days)}
                    p.food_nutrients = {Food.RICE: defaultdict(float, {Nutrient.ZINC: .001 * unit})}
                    p.food_minimize_usage = {Food.RICE: False}
                    p.add_need(Nutrient.ZINC, .01 * unit * days, .1 * unit * days,
                               NeedRequired.REQUIRED, NeedSoftness.SOFT)
                    self.assertTrue(p.solve())
                    self.assertAlmostEqual(p.sol['primal objective'], expected, delta=1e-6)

    def test_invalid_soft_intervals(self):
        for lb, ub in ((-1., 2.), (2., 1.), (0., 0.), (1., float('inf')),
                       (1., float('nan')), (float('nan'), 2.)):
            with self.subTest(lb=lb, ub=ub), self.assertRaises(ValueError):
                p = RecipeSolver()
                p.food_limits = {Food.RICE: (0., 100.)}
                p.food_nutrients = {Food.RICE: defaultdict(float)}
                p.food_minimize_usage = {Food.RICE: False}
                p.add_need(Nutrient.ZINC, lb, ub, NeedRequired.REQUIRED, NeedSoftness.SOFT)
                p.solve()

    def test_batch_invariance_with_usage_penalty(self):
        self.assertAlmostEqual(self.solve_amount(days=10, penalize=True) / 10,
                               self.solve_amount(penalize=True), delta=1e-3)

    def test_shortage_cost_is_zero_at_and_above_minimum(self):
        # Isolate the shortage-only loss by explicitly disabling the solver's
        # default implicit upper preference.
        for amount, expected in ((5., .25), (10., 0.), (30., 0.)):
            with self.subTest(amount=amount):
                p = RecipeSolver(implicit_soft_upper_multiplier=None)
                p.food_limits = {Food.RICE: (amount, amount)}
                p.food_nutrients = {Food.RICE: defaultdict(float, {Nutrient.ZINC: .001})}
                p.food_minimize_usage = {Food.RICE: False}
                p.add_need(Nutrient.ZINC, .01, None,
                           NeedRequired.REQUIRED, NeedSoftness.SOFT)
                self.assertTrue(p.solve())
                self.assertAlmostEqual(p.sol['primal objective'], expected, delta=1e-6)
                self.assertAlmostEqual(p.amount(Food.RICE), amount, places=4)

    def test_zero_minimum_adds_no_penalty(self):
        p = RecipeSolver()
        p.food_limits = {Food.RICE: (1., 1.)}
        p.food_nutrients = {Food.RICE: defaultdict(float)}
        p.food_minimize_usage = {Food.RICE: False}
        p.add_need(Nutrient.ZINC, 0., None,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertEqual(p.sol['primal objective'], 0.)

    def test_soft_shortfall_remains_feasible(self):
        # Impossible minimum (1 g vs stock containing 0.1 g) stays soft.
        p = RecipeSolver()
        p.food_limits = {Food.RICE: (0., 100.)}
        p.food_nutrients = {Food.RICE: defaultdict(float, {Nutrient.ZINC: 0.001})}
        p.food_minimize_usage = {Food.RICE: False}
        p.add_need(Nutrient.ZINC, 1., None, NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertAlmostEqual(p.amount(0), 100., places=3)


class RatioTests(unittest.TestCase):
    def solve_recipe(self, calcium_target, days=1, with_ratio=True):
        p = RecipeSolver()
        # Synthetic foods isolate the ratio from real database incompleteness.
        p.food_limits = {Food.EGG_SHELL_POWDER: (0., 400. * days),
                         Food.RICE: (100. * days, 100. * days)}
        p.food_nutrients = {
            Food.EGG_SHELL_POWDER: defaultdict(float, {Nutrient.CALCIUM: .01}),
            Food.RICE: defaultdict(float, {Nutrient.PHOSPHORUS: .01}),
        }
        p.food_minimize_usage = {food: False for food in p.food_limits}
        # Absolute limits alone allow the intentionally bad soft targets.
        for nutrient in (Nutrient.CALCIUM, Nutrient.PHOSPHORUS):
            p.add_need(nutrient, .1 * days, 4. * days,
                       NeedRequired.REQUIRED, NeedSoftness.HARD)
        # A separate soft nutrient supplied by the calcium food steers the
        # unconstrained calcium total to calcium_target grams per day.
        p.food_nutrients[Food.EGG_SHELL_POWDER][Nutrient.ZINC] = .01
        p.add_need(Nutrient.ZINC, calcium_target * days, calcium_target * days,
                   NeedRequired.REQUIRED, NeedSoftness.SOFT)
        if with_ratio:
            p.add_nutrient_ratio(Nutrient.CALCIUM, Nutrient.PHOSPHORUS, 1., 2.)
        self.assertTrue(p.solve())
        calcium = .01 * p.amount(Food.EGG_SHELL_POWDER)
        phosphorus = .01 * p.amount(Food.RICE)
        return calcium / phosphorus

    def test_ratio_bounds_override_soft_preferences(self):
        for target, expected in ((.5, 1.), (1.5, 1.5), (3., 2.)):
            for days in (1, 10):
                with self.subTest(target=target, days=days):
                    ratio = self.solve_recipe(target, days)
                    # A soft point target is accurate only to QP tolerance;
                    # verify hard ratio bounds independently and more tightly.
                    self.assertAlmostEqual(ratio, expected, delta=3e-4)
                    self.assertGreaterEqual(ratio, 1. - 1e-6)
                    self.assertLessEqual(ratio, 2. + 1e-6)

    def test_absolute_bounds_alone_allow_bad_ratios(self):
        self.assertAlmostEqual(self.solve_recipe(.5, with_ratio=False), .5, places=4)
        self.assertAlmostEqual(self.solve_recipe(3., with_ratio=False), 3., places=4)

    def test_invalid_ratio_bounds(self):
        for lb, ub in ((0, 2), (-1, 2), (2, 1), (1, float('inf')), (float('nan'), 2)):
            with self.subTest(lb=lb, ub=ub), self.assertRaises(ValueError):
                RecipeSolver().add_nutrient_ratio(Nutrient.CALCIUM, Nutrient.PHOSPHORUS, lb, ub)


class NeedsTests(unittest.TestCase):
    def test_reference_profile_and_units(self):
        n = dog(age=3, weight=7, active=False, daily_kcal=1000)
        self.assertEqual(n[Nutrient.PROTEIN][0], 52.10 * G)
        self.assertAlmostEqual(n[Nutrient.VITAMIN_E][0] / MG, 6.968)
        self.assertAlmostEqual(n[Nutrient.VITAMIN_D][0] / MCG, 3.975)
        self.assertAlmostEqual(n[Nutrient.VITAMIN_B12][0] / MCG, 9.68)
        self.assertEqual(n[Nutrient.VITAMIN_E][2], NeedRequired.NOT_REQUIRED)
        self.assertEqual(n[Nutrient.ZINC][3], NeedSoftness.SOFT)

    def test_energy_activity_and_weight(self):
        low = dog(age=3, weight=7, active=False)
        high = dog(age=3, weight=7, active=True)
        heavy = dog(age=3, weight=14, active=False)
        self.assertAlmostEqual(low[Nutrient.ENERGY][0] / KCAL / .9, 95 * 7**.75)
        self.assertGreater(high[Nutrient.ENERGY][0], low[Nutrient.ENERGY][0])
        for nutrient in low:
            self.assertAlmostEqual(heavy[nutrient][0] / low[nutrient][0], 2**.75)

    def test_override_and_batch_scaling(self):
        one = dog(age=3, weight=7, active=False, daily_kcal=500)
        two = dog(age=3, weight=7, active=False, daily_kcal=1000)
        batch = scale(one, 10)
        for nutrient, need in one.items():
            self.assertAlmostEqual(two[nutrient][0], 2 * need[0])
            self.assertAlmostEqual(batch[nutrient][0], 10 * need[0])
            self.assertEqual(batch[nutrient][2:], need[2:])
            if need[1] is not None:
                self.assertAlmostEqual(batch[nutrient][1], 10 * need[1])

    def test_invalid_inputs(self):
        for change in ({'weight': 0}, {'weight': float('nan')}, {'age': .5},
                       {'daily_kcal': -1}, {'daily_kcal': float('inf')}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                dog(**({'age': 3, 'weight': 7, 'active': False} | change))
        for days in (0, -1, 1.5, True):
            with self.assertRaises(ValueError):
                scale({}, days)


if __name__ == '__main__':
    unittest.main()
