import io
import unittest
from collections import defaultdict
from contextlib import redirect_stdout
from unittest.mock import Mock, patch

from src.food import Food, get_or_load
from src.food_getters.usda import usda
from src.needs import dog
from src.nutrient import Nutrient as N, nutrient_value
from src.recipe import NeedRequired, NeedSoftness, RecipeSolver
from src.units import MG, MCG


class ExtendedNutrientTests(unittest.TestCase):
    def test_adult_table_both_profiles(self):
        rows = {
            N.FAT: (13.75, 13.75), N.LINOLEIC_ACID: (3.82, 3.27),
            N.ARGININE: (1.51, 1.30), N.HISTIDINE: (.67, .58),
            N.ISOLEUCINE: (1.33, 1.15), N.LEUCINE: (2.37, 2.05),
            N.LYSINE: (1.22, 1.05), N.METHIONINE: (1.16, 1.),
            N.METHIONINE_CYSTINE: (2.21, 1.91), N.PHENYLALANINE: (1.56, 1.35),
            N.PHENYLALANINE_TYROSINE: (2.58, 2.23), N.THREONINE: (1.51, 1.30),
            N.TRYPTOPHAN: (.49, .43), N.VALINE: (1.71, 1.48),
            N.VITAMIN_B5: (4.11 * MG, 3.55 * MG),
            N.FOLIC_ACID: (74.70 * MCG, 64.50 * MCG), N.CHLORIDE: (.43, .38),
        }
        for active in (False, True):
            needs = dog(age=3, weight=7, active=active, daily_kcal=1000)
            for nutrient, values in rows.items():
                with self.subTest(active=active, nutrient=nutrient):
                    self.assertAlmostEqual(needs[nutrient][0], values[int(active)])
                    self.assertEqual(needs[nutrient][2:], (NeedRequired.REQUIRED, NeedSoftness.SOFT))
            for nutrient in (N.ALPHA_LINOLENIC_ACID, N.EPA_DHA, N.ARACHIDONIC_ACID,
                             N.VITAMIN_B7, N.VITAMIN_K):
                self.assertNotIn(nutrient, needs)

    def test_usda_import_units_and_specific_fatty_acids(self):
        values = [('Methionine', .3, 'g'), ('Cystine', .2, 'g'),
                  ('Phenylalanine', .4, 'g'), ('Tyrosine', .2, 'g'),
                  ('PUFA 18:2 n-6 c,c', 2., 'g'), ('PUFA 18:2', 99., 'g'),
                  ('PUFA 18:3 n-3 c,c,c (ALA)', .1, 'g'),
                  ('PUFA 20:5 n-3 (EPA)', .05, 'g'), ('PUFA 22:6 n-3 (DHA)', .1, 'g'),
                  ('Pantothenic acid', 2., 'mg'), ('Folate, total', 40., 'µg')]
        response = Mock()
        response.json.return_value = {'description': 'fixture', 'foodNutrients': [
            {'value': value, 'nutrient': {'name': name, 'nutrientUnit': {'name': unit}}}
            for name, value, unit in values]}
        with patch('src.food_getters.usda.requests.get', return_value=response):
            _, nutrients = usda(123)()
        self.assertAlmostEqual(nutrients[N.LINOLEIC_ACID], .02)
        self.assertAlmostEqual(nutrients[N.VITAMIN_B5], 2 * MG / 100)
        self.assertAlmostEqual(nutrients[N.FOLIC_ACID], 40 * MCG / 100)
        for nutrient, expected in ((N.METHIONINE_CYSTINE, .005),
                                   (N.PHENYLALANINE_TYROSINE, .006), (N.EPA_DHA, .0015)):
            value, known = nutrient_value(nutrients, nutrient)
            self.assertTrue(known)
            self.assertAlmostEqual(value, expected)

    def test_partial_sums_and_legacy_b5_do_not_mutate_data(self):
        data = defaultdict(float, {N.METHIONINE: .01, N.PANTOTHENIC_ACID: .001})
        self.assertEqual(nutrient_value(data, N.METHIONINE_CYSTINE), (.01, False))
        self.assertNotIn(N.CYSTINE, data)
        self.assertEqual(nutrient_value(data, N.VITAMIN_B5), (.001, True))
        data[N.VITAMIN_B5] = .002
        self.assertEqual(nutrient_value(data, N.VITAMIN_B5), (.002, True))

    def test_combined_target_affects_solver_and_missing_data_is_reported(self):
        p = RecipeSolver()
        p.food_limits = {Food.RICE: (0., 100.)}
        p.food_nutrients = {Food.RICE: defaultdict(float, {N.METHIONINE: .01, N.CYSTINE: .01})}
        p.food_minimize_usage = {Food.RICE: False}
        p.add_need(N.METHIONINE_CYSTINE, .2, None, NeedRequired.REQUIRED, NeedSoftness.SOFT)
        p.add_need(N.LINOLEIC_ACID, .1, None, NeedRequired.REQUIRED, NeedSoftness.SOFT)
        self.assertTrue(p.solve())
        self.assertAlmostEqual(p.amount(0), 10.5, places=4)
        self.assertNotIn(N.LINOLEIC_ACID, p.food_nutrients[Food.RICE])
        output = io.StringIO()
        with redirect_stdout(output):
            p.print_nutrition(p.needs)
        self.assertIn('incomplete data: RICE', output.getvalue())

    def test_explicit_cache_refresh_bypasses_old_data(self):
        import tempfile
        import os
        from pathlib import Path
        import json
        with tempfile.TemporaryDirectory() as directory:
            previous = os.getcwd()
            try:
                os.chdir(directory)
                Path('foods').mkdir()
                Path('foods/RICE.json').write_text('{"__name__":"old","PROTEIN":0.1}')
                getter = Mock(return_value=('new', {N.LYSINE: .01}))
                with patch.dict('src.food.GETTERS', {Food.RICE: getter}):
                    self.assertNotIn(N.LYSINE, get_or_load(Food.RICE))
                    getter.assert_not_called()
                    with redirect_stdout(io.StringIO()):
                        self.assertEqual(get_or_load(Food.RICE, refresh=True)[N.LYSINE], .01)
                    self.assertEqual(json.loads(Path('foods/RICE.json').read_text())['LYSINE'], .01)
            finally:
                os.chdir(previous)
