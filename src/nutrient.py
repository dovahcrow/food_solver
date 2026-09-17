from enum import Enum, unique
from typing import DefaultDict, Dict


@unique
class Nutrient(Enum):
    ENERGY = 1
    PROTEIN = 2
    FAT = 3
    CHOLESTEROL = 4
    ASH = 5
    CARB = 6
    FIBER = 7

    CALCIUM = 21
    PHOSPHORUS = 22
    POTASSIUM = 23
    SODIUM = 24
    MAGNESIUM = 25
    IRON = 26
    ZINC = 27
    SELENIUM = 28
    COPPER = 29
    MANGANESE = 30
    IODINE = 31
    CHLORIDE = 32

    VITAMIN_A = 41
    VITAMIN_C = 42
    VITAMIN_D = 43
    VITAMIN_E = 44
    VITAMIN_K = 45
    VITAMIN_B1 = 46  # Thiamin
    VITAMIN_B2 = 47  # Riboflavin
    VITAMIN_B5 = 48  # Pantothenic acid
    VITAMIN_B6 = 49
    VITAMIN_B7 = 50  # Biotin, Vitamin H
    VITAMIN_B12 = 51

    NIACIN = 61
    PANTOTHENIC_ACID = 62
    FOLIC_ACID = 63
    CHOLINE = 64
    CAROTENE = 65

    # Individual amino acids and FEDIAF's combined amino-acid targets.
    ARGININE = 71
    HISTIDINE = 72
    ISOLEUCINE = 73
    LEUCINE = 74
    LYSINE = 75
    METHIONINE = 76
    CYSTINE = 77
    PHENYLALANINE = 78
    TYROSINE = 79
    THREONINE = 80
    TRYPTOPHAN = 81
    VALINE = 82
    METHIONINE_CYSTINE = 83
    PHENYLALANINE_TYROSINE = 84

    LINOLEIC_ACID = 91
    ALPHA_LINOLENIC_ACID = 92
    ARACHIDONIC_ACID = 93
    EPA = 94
    DHA = 95
    EPA_DHA = 96


Nutrients = DefaultDict[Nutrient, float]


# Combined requirements are sums, not replacements for the individual minima.
COMBINED_NUTRIENTS = {
    Nutrient.METHIONINE_CYSTINE: (Nutrient.METHIONINE, Nutrient.CYSTINE),
    Nutrient.PHENYLALANINE_TYROSINE: (Nutrient.PHENYLALANINE, Nutrient.TYROSINE),
    Nutrient.EPA_DHA: (Nutrient.EPA, Nutrient.DHA),
}


def nutrient_value(nutrients, nutrient):
    """Return (known contribution, complete data), without inserting fake zeros.

    With one component missing, a combined target uses the known partial sum
    and remains marked incomplete. A legacy B5 name is a fallback, never an
    additional contribution to the same vitamin.
    """
    if nutrient in nutrients:
        return nutrients[nutrient], True
    if nutrient == Nutrient.VITAMIN_B5 and Nutrient.PANTOTHENIC_ACID in nutrients:
        return nutrients[Nutrient.PANTOTHENIC_ACID], True
    if nutrient in COMBINED_NUTRIENTS:
        parts = [nutrient_value(nutrients, n) for n in COMBINED_NUTRIENTS[nutrient]]
        return sum(v for v, _ in parts), all(known for _, known in parts)
    return 0., False
