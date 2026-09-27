from math import isfinite
from typing import Dict, Optional, Tuple

from .recipe import NeedRequired, NeedSoftness

from .nutrient import Nutrient
from .units import KCAL, MCG, MG, VITAMIN_A_IU, VITAMIN_D_IU, VITAMIN_E_IU, G


# FEDIAF 2025, Table III-3b (adult dogs, per 1000 kcal ME):
# https://europeanpetfood.org/wp-content/uploads/2025/09/FEDIAF-Nutritional-Guidelines_2025-ONLINE.pdf
# All numeric adult minimum rows in Table III-3b are represented. This is
# not a complete nutritional adequacy check (food data may be incomplete). Maxima below are nutritional (N), not EU legal (L)
# limits on a dry-matter basis; we cannot apply the latter without dry matter.
def dog(
    *,
    age: float,
    weight: float,
    active: bool,
    daily_kcal: Optional[float] = None,
) -> Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]:
    """Adult maintenance estimates; weight in kg and age in years.

    active=False selects the 95 kcal/kg**0.75 reference profile, True the
    110 profile (ordinary activity, not working/sport dogs). daily_kcal can
    override the energy estimate; active still selects the nutrient profile.
    Age is a scope check, not an invented continuous age correction. Dogs
    still growing, including large breeds older than one year, need another
    profile. This function is not for gestation or lactation.
    Returned masses are grams; energy is joules. Values describe a full day,
    before any allocation between homemade and commercial food.
    """
    if not isfinite(age) or age < 1:
        raise ValueError("Only adult maintenance is supported (age >= 1 year)")
    if not isfinite(weight) or weight <= 0:
        raise ValueError("weight must be a positive finite number in kg")
    if daily_kcal is None:
        daily_kcal = (110 if active else 95) * weight**0.75
    if not isfinite(daily_kcal) or daily_kcal <= 0:
        raise ValueError("daily_kcal must be positive and finite")
    energy = daily_kcal * KCAL
    mod = daily_kcal / 1000

    def minimum(low_activity: float, ordinary_activity: float) -> float:
        return ordinary_activity if active else low_activity

    return {
        # Additional adult minima (95 / 110 kcal profiles), all SOFT because
        # food composition data may be incomplete. Individual methionine and
        # phenylalanine minima apply alongside their combined targets.
        # Adult ALA, arachidonic acid, EPA+DHA, biotin and vitamin K have no
        # numeric minimum in this table: do not import puppy values or invent
        # zero targets (normalization requires a strictly positive target).
        Nutrient.FAT: (
            minimum(13.75, 13.75) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.LINOLEIC_ACID: (
            minimum(3.82, 3.27) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.ARGININE: (
            minimum(1.51, 1.3) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.HISTIDINE: (
            minimum(0.67, 0.58) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.ISOLEUCINE: (
            minimum(1.33, 1.15) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.LEUCINE: (
            minimum(2.37, 2.05) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.LYSINE: (
            minimum(1.22, 1.05) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.METHIONINE: (
            minimum(1.16, 1.0) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.METHIONINE_CYSTINE: (
            minimum(2.21, 1.91) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.PHENYLALANINE: (
            minimum(1.56, 1.35) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.PHENYLALANINE_TYROSINE: (
            minimum(2.58, 2.23) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.THREONINE: (
            minimum(1.51, 1.3) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.TRYPTOPHAN: (
            minimum(0.49, 0.43) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VALINE: (
            minimum(1.71, 1.48) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B5: (
            minimum(4.11, 3.55) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.FOLIC_ACID: (
            minimum(74.7, 64.5) * MCG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.CHLORIDE: (
            minimum(0.43, 0.38) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.ENERGY: (
            energy * 0.9,
            energy * 1.05,
            NeedRequired.REQUIRED,
            NeedSoftness.HARD,
        ),  # in J
        Nutrient.PROTEIN: (
            minimum(52.10, 45.00) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.HARD,
        ),
        Nutrient.VITAMIN_A: (
            minimum(1754, 1515) * VITAMIN_A_IU * mod,
            100000 * VITAMIN_A_IU * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B1: (
            minimum(0.62, 0.54) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B2: (
            minimum(1.74, 1.50) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B6: (
            minimum(0.42, 0.36) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_B12: (
            minimum(9.68, 8.36) * MCG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_E: (
            minimum(10.40, 9.00) * VITAMIN_E_IU * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.VITAMIN_D: (
            minimum(159, 138) * VITAMIN_D_IU * mod,
            800 * VITAMIN_D_IU * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.CALCIUM: (
            minimum(1.45, 1.25) * G * mod,
            6.25 * G * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.HARD,
        ),
        Nutrient.COPPER: (
            minimum(2.08, 1.80) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.IODINE: (
            minimum(0.30, 0.26) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.ZINC: (
            minimum(20.80, 18.00) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.CHOLINE: (
            minimum(474, 409) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.IRON: (
            minimum(10.40, 9.00) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.SELENIUM: (
            minimum(67.50, 57.50) * MCG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.PHOSPHORUS: (
            minimum(1.16, 1.00) * G * mod,
            4 * G * mod,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.SODIUM: (
            minimum(0.29, 0.25) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.POTASSIUM: (
            minimum(1.45, 1.25) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.MANGANESE: (
            minimum(1.67, 1.44) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.MAGNESIUM: (
            minimum(0.20, 0.18) * G * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
        Nutrient.NIACIN: (
            minimum(4.74, 4.09) * MG * mod,
            None,
            NeedRequired.REQUIRED,
            NeedSoftness.SOFT,
        ),
    }


def scale(
    d: Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]],
    day: int = 1,
) -> Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]:
    if isinstance(day, bool) or not isinstance(day, int) or day <= 0:
        raise ValueError("day must be a positive integer")
    ret = {}
    for k, v in d.items():
        ret[k] = (v[0] * day, v[1] * day if v[1] is not None else None, *v[2:])
    return ret
