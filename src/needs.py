from typing import Dict, Optional, Tuple

from .units import G, MCG, MG, KJ

from .nutrient import Nutrient


# From: https://nap.nationalacademies.org/resource/10668/dog_nutrition_final_fix.pdf
def dog(*, age: float, weight: float, active: bool, day: int = 1) -> Dict[Nutrient, Tuple[float, Optional[float]]]:
    def energe_require(weight: float, active: bool) -> float:
        weight /= 0.453592  # convert kg to pound
        if active:
            return (14.15 * weight + 281.5) * 4.184 * KJ
        else:
            return (25.9 * weight + 145) * 4.184 * KJ

    energy = energe_require(weight, active)
    return mul_day(
        {
            Nutrient.ENERGY: (energy * 0.9, energy * 1.2),  # in J
            Nutrient.PROTEIN: (45 * G, None),
            Nutrient.VITAMIN_A: (375 * MCG, 18750 * MCG),
            Nutrient.VITAMIN_B1: (0.56 * MG, None),
            Nutrient.VITAMIN_B6: (0.38 * MG, None),
            # Nutrient.VITAMIN_B12: (0.01 * MG, None),
            Nutrient.VITAMIN_E: (8.375 * MCG, None),
            # Nutrient.VITAMIN_D: (3.125 * MCG, 18.75 * MCG),
            Nutrient.CALCIUM: (1.25 * G, 6.25 * G),
            Nutrient.COPPER: (1.83 * MG, None),
            Nutrient.IODINE: (0.25 * MG, 2.75 * MG),
            Nutrient.ZINC: (20 * MG, None),
            # Nutrient.CHOLINE: (340 * MG, None),
            Nutrient.IRON: (10 * MG, None),
            Nutrient.SELENIUM: (0.08 * MG, 0.5 * MG),
            Nutrient.PHOSPHORUS: (1 * G, 4 * G),
            Nutrient.SODIUM: (0.2 * G, 2.5 * G),
            # Nutrient.CHLORIDE: (0.3 * G, None),
            Nutrient.POTASSIUM: (1.5 * G, None),
            Nutrient.MANGANESE: (1.25 * MG, None),
            Nutrient.MAGNESIUM: (0.15 * G, None),
            Nutrient.RIBOFLAVIN: (1.3 * MG, None),
            Nutrient.NIACIN: (3.4 * MG, None),
        },
        day,
    )


def mul_day(
    d: Dict[Nutrient, Tuple[float, Optional[float]]], day: float
) -> Dict[Nutrient, Tuple[float, Optional[float]]]:
    return {key: (value[0] * day, value[1] * day if value[1] is not None else None) for key, value in d.items()}
