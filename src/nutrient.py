# FROM https://nlc.chinanutri.cn/fq/

from enum import Enum
from typing import Dict


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
    VITAMIN_B1 = 46
    VITAMIN_B2 = 47
    VITAMIN_B5 = 48 # Pantothenic acid
    VITAMIN_B6 = 49
    VITAMIN_B7 = 50 # Biotin, Vitamin H
    VITAMIN_B12 = 51

    RIBOFLAVIN = 60
    NIACIN = 61
    PANTOTHENIC_ACID = 62
    FOLIC_ACID = 63
    CHOLINE = 64
    CAROTENE = 65

Nutrients = Dict[Nutrient, float]
