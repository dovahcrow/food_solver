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
                self.assertAlmostEqual(self.solve_amount(unit), 10.5, places=4)

    def test_bounded_soft_target(self):
        self.assertAlmostEqual(self.solve_amount(upper=0.03), 20., places=4)

    def test_batch_invariance_with_usage_penalty(self):
        self.assertAlmostEqual(self.solve_amount(days=10, penalize=True) / 10,
                               self.solve_amount(penalize=True), places=4)

    def test_soft_shortfall_remains_feasible(self):
        # Impossible target (1.05 g vs stock containing 0.1 g) stays soft.
        p = RecipeSolver()
        p.food_limits = {Food.RICE: (0., 100.)}
        p.food_nutrients = {Food.RICE: defaultdict(float, {Nutrient.ZINC: 0.001})}
        p.food_minimize_usage = {Food.RICE: False}
        p.add_need(Nutrient.ZINC, 1., None, NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertAlmostEqual(p.amount(0), 100., places=3)


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
