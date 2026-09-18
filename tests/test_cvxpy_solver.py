import unittest
from collections import defaultdict

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
