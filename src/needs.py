from typing import Dict, Optional, Tuple

from .units import G, MCG, MG, KJ

from .nutrient import Nutrient


# From: https://nap.nationalacademies.org/resource/10668/dog_nutrition_final_fix.pdf
def dog(*, age: float, weight: float, active: bool) -> Dict[Nutrient, Tuple[float, Optional[float]]]:
    def energe_require(weight: float, active: bool) -> float:
        weight /= 0.453592  # convert kg to pound
        if active:
            return (14.15 * weight + 281.5) * 4.184 * KJ
        else:
            return (25.9 * weight + 145) * 4.184 * KJ

    energy = energe_require(weight, active)
    mod = 0.4776
    return {
        Nutrient.ENERGY: (energy * 0.9, energy * 1.2),  # in J
        Nutrient.PROTEIN: (45 * G * mod, None),
        Nutrient.VITAMIN_A: (375 * MCG * mod, 18750 * MCG * mod),
        Nutrient.VITAMIN_B1: (0.56 * MG * mod, None),
        Nutrient.VITAMIN_B6: (0.38 * MG * mod, None),
        Nutrient.VITAMIN_B12: (0.01 * MG * mod, None),
        Nutrient.VITAMIN_E: (8.375 * MCG * mod, None),
        Nutrient.VITAMIN_D: (3.125 * MCG * mod, 18.75 * MCG * mod),
        Nutrient.CALCIUM: (1.25 * G * mod, 6.25 * G * mod),
        Nutrient.COPPER: (1.83 * MG * mod, None),
        Nutrient.IODINE: (0.25 * MG * mod, 2.75 * MG * mod),
        Nutrient.ZINC: (20 * MG * mod, None),
        Nutrient.CHOLINE: (340 * MG * mod, None),
        Nutrient.IRON: (10 * MG * mod, None),
        Nutrient.SELENIUM: (0.08 * MG * mod, 0.5 * MG * mod),
        Nutrient.PHOSPHORUS: (1 * G * mod, 4 * G * mod),
        Nutrient.SODIUM: (0.2 * G * mod, 2.5 * G * mod),
        Nutrient.CHLORIDE: (0.3 * G * mod, None),
        Nutrient.POTASSIUM: (1.5 * G * mod, None),
        Nutrient.MANGANESE: (1.25 * MG * mod, None),
        Nutrient.MAGNESIUM: (0.15 * G * mod, None),
        Nutrient.RIBOFLAVIN: (1.3 * MG * mod, None),
        Nutrient.NIACIN: (3.4 * MG * mod, None),
    }
