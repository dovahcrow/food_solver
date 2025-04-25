from typing import Dict, Optional, Tuple

from src.recipe import NeedRequired, NeedSoftness

from .units import G, KCAL, MCG, MG

from .nutrient import Nutrient


# From: https://nap.nationalacademies.org/resource/10668/dog_nutrition_final_fix.pdf
def dog(
    *, age: float, weight: float, active: bool,
) -> Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]:
    def energe_require(weight: float, active: bool) -> float:
        # According to the Association for Pet Obesity and Prevention, you can use this formula to calculate a dog's caloric needs. Every pound of weight is equivalent to 0.45 kilograms. So for example, a 60-pound (27.2-kilogram) dog would need this calculation: (27.2 x 30) + 70 = 886 calories needed p.
        # return (weight * 30 + 70) * KCAL
        weight /= 0.453592  # convert kg to pound
        if active:
            return (14.15 * weight + 281.5) * KCAL
        else:
            return (25.9 * weight + 145) * KCAL

    energy = energe_require(weight, active)
    mod = 0.4776  # BALANCEIT tells use nutrients per calorie

    return {
        Nutrient.ENERGY: (
            energy * 0.9,
            energy * 1.05,
            NeedRequired.REQUIRED,
            NeedSoftness.HARD,
        ),  # in J
        Nutrient.PROTEIN: (
            45 * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.HARD,
        ),
        Nutrient.VITAMIN_A: (
            375 * MCG * mod,
            18750 * MCG * mod,
            NeedRequired.NOT_REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B1: (
            0.56 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B2: (
            1.3 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B6: (
            0.38 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B12: (
            0.01 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_E: (
            8.375 * MCG * mod,
            None,
            NeedRequired.NOT_REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_D: (
            3.125 * MCG * mod,
            18.75 * MCG * mod,
            NeedRequired.NOT_REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.CALCIUM: (
            1.25 * G * mod,
            6.25 * G * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.HARD,
        ),
        Nutrient.COPPER: (
            1.83 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.IODINE: (
            0.25 * MG * mod,
            2.75 * MG * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.ZINC: (
            20 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.CHOLINE: (
            340 * MG * mod,
            None,
            NeedRequired.NOT_REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.IRON: (
            10 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.SELENIUM: (
            0.08 * MG * mod,
            0.5 * MG * mod,
            NeedRequired.NOT_REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.PHOSPHORUS: (
            1 * G * mod,
            4 * G * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.SODIUM: (
            0.2 * G * mod,
            2.5 * G * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        # Nutrient.CHLORIDE: (
        #     0.3 * G * mod,
        #     None,
        #     NeedRequired.NOT_REQUIRED,
        #     NeedSoftness.SOFT,
        # ),
        Nutrient.POTASSIUM: (
            1.5 * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.MANGANESE: (
            1.25 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.MAGNESIUM: (
            0.15 * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.NIACIN: (
            3.4 * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
    }

    
def scale(
    d: Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]],
    day: int = 1,
) -> Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]:
    ret = {}
    for k, v in d.items():
        ret[k] = (v[0] * day, v[1] * day if v[1] is not None else None, *v[2:]) 
    return ret
