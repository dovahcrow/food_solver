# FROM https://fdc.nal.usda.gov/index.html

from collections.abc import Callable
from typing import Dict, Tuple

import requests

from ..food_getters.chinanutri import normalize
from ..nutrient import Nutrient
from ..units import VITAMIN_D_IU


def usda(id: int) -> Callable[[], Tuple[str, Dict[Nutrient, float]]]:
    def inner() -> Tuple[str, Dict[Nutrient, float]]:
        ret = {}

        resp = requests.get(f"https://fdc.nal.usda.gov/portal-data/external/{id}")
        data = resp.json()
        food_name = data["description"]

        for nut in data["foodNutrients"]:
            if "value" not in nut:
                continue

            name = nut["nutrient"]["name"]
            value = nut["value"]
            unit = nut["nutrient"]["nutrientUnit"]["name"].strip()

            # print(name, value, f"'{(unit)}'")

            if (
                name.startswith("MUFA")
                or name.startswith("SFA")
                or name.startswith("PUFA")
            ):
                continue

            try:
                nut = NAME_TO_ENUM[name]
            except KeyError:
                print(name)
                continue

            if nut is None:
                continue

            if name == "Vitamin D (D2 + D3), International Units" and unit == "IU":
                value *= VITAMIN_D_IU
            else:
                value = normalize(value, unit)

            ret[nut] = value / 100  # Food on this website is per 100g

        return food_name, ret

    return inner


def convert_cooked_chicken_breast_to_uncoocked(
    f: Callable[[], Tuple[str, Dict[Nutrient, float]]],
) -> Callable[[], Tuple[str, Dict[Nutrient, float]]]:
    def inner():
        name, ret = f()
        # Dirty fix for chicken breast. We use cooked chicken breast because some nutrients of raw checken breast are missing.
        return name, {k: v / 1.49 for k, v in ret.items()}

    return inner


NAME_TO_ENUM = {
    "Water": None,
    "Nitrogen": None,
    "Protein": Nutrient.PROTEIN,
    "Total lipid (fat)": Nutrient.FAT,
    "Total fat (NLEA)": None,
    "Ash": Nutrient.ASH,
    "Carbohydrate, by difference": Nutrient.CARB,
    "Carbohydrate, by summation": None,
    "Fiber, total dietary": Nutrient.FIBER,
    "Sugars, Total": None,
    "Sucrose": None,
    "Glucose": None,
    "Fructose": None,
    "Lactose": None,
    "Maltose": None,
    "Starch": None,
    "Calcium, Ca": Nutrient.CALCIUM,
    "Iron, Fe": Nutrient.IRON,
    "Magnesium, Mg": Nutrient.MAGNESIUM,
    "Phosphorus, P": Nutrient.PHOSPHORUS,
    "Potassium, K": Nutrient.POTASSIUM,
    "Sodium, Na": Nutrient.SODIUM,
    "Zinc, Zn": Nutrient.ZINC,
    "Copper, Cu": Nutrient.COPPER,
    "Manganese, Mn": Nutrient.MANGANESE,
    "Iodine, I": Nutrient.IODINE,
    "Selenium, Se": Nutrient.SELENIUM,
    "Thiamin": Nutrient.VITAMIN_B1,
    "Riboflavin": Nutrient.VITAMIN_B2,
    "Niacin": Nutrient.NIACIN,
    "Vitamin B-6": Nutrient.VITAMIN_B6,
    "Folate, total": Nutrient.FOLIC_ACID,
    "Choline, total": Nutrient.CHOLINE,
    "Choline, free": None,
    "Choline, from phosphocholine": None,
    "Choline, from phosphotidyl choline": None,
    "Choline, from glycerophosphocholine": None,
    "Choline, from sphingomyelin": None,
    "Betaine": None,
    "Vitamin B-12": Nutrient.VITAMIN_B12,
    "Vitamin A, RAE": Nutrient.VITAMIN_A,
    "Retinol": None,  # Also VA
    "Carotene, beta": Nutrient.CAROTENE,
    "cis-beta-Carotene": None,
    "trans-beta-Carotene": None,
    "Carotene, alpha": None,
    "Cryptoxanthin, beta": None,
    "Cryptoxanthin, alpha": None,
    "Lycopene": None,
    "cis-Lycopene": None,
    "trans-Lycopene": None,
    "cis-Lutein/Zeaxanthin": None,
    "Lutein": None,
    "Zeaxanthin": None,
    "Vitamin D (D2 + D3), International Units": None,
    "Vitamin D (D2 + D3)": Nutrient.VITAMIN_D,
    "Vitamin D2 (ergocalciferol)": None,
    "Vitamin D3 (cholecalciferol)": None,
    "25-hydroxycholecalciferol": None,
    "Fatty acids, total saturated": None,
    "Fatty acids, total monounsaturated": None,
    "Fatty acids, total polyunsaturated": None,
    "Cholesterol": Nutrient.CHOLESTEROL,
    "Tryptophan": None,
    "Threonine": None,
    "Isoleucine": None,
    "Leucine": None,
    "Lysine": None,
    "Methionine": None,
    "Phenylalanine": None,
    "Tyrosine": None,
    "Valine": None,
    "Arginine": None,
    "Histidine": None,
    "Alanine": None,
    "Aspartic acid": None,
    "Glutamic acid": None,
    "Glycine": None,
    "Proline": None,
    "Serine": None,
    "Hydroxyproline": None,
    "Cysteine": None,
    "Energy (Atwater Specific Factors)": None,
    "Energy (Atwater General Factors)": Nutrient.ENERGY,
    "Fatty acids, total trans": None,
    "Fiber, soluble": None,
    "Fiber, insoluble": None,
    "Galactose": None,
    "Vitamin C, total ascorbic acid": Nutrient.VITAMIN_C,
    "Pantothenic acid": Nutrient.VITAMIN_B5,
    "Lutein + zeaxanthin": None,
    "Phytoene": None,
    "Phytofluene": None,
    "Vitamin E (alpha-tocopherol)": Nutrient.VITAMIN_E,
    "Tocopherol, beta": None,
    "Tocopherol, gamma": None,
    "Tocopherol, delta": None,
    "Tocotrienol, alpha": None,
    "Tocotrienol, beta": None,
    "Tocotrienol, gamma": None,
    "Tocotrienol, delta": None,
    "Cystine": None,
    "Biotin": None,
    "Total dietary fiber (AOAC 2011.25)": None,
    "High Molecular Weight Dietary Fiber (HMWDF)": None,
    "Low Molecular Weight Dietary Fiber (LMWDF)": None,
    "Molybdenum, Mo": None,
    "Vitamin K (phylloquinone)": None,
    "Vitamin K (Dihydrophylloquinone)": None,
    "Vitamin K (Menaquinone-4)": None,
    "Citric acid": None,
    "Malic acid": None,
}
