from json import load, dump
from pathlib import Path
from enum import Enum, auto

from src.units import KG, MCG, MG, G

from .food_getters.chinanutri import chinanutri
from .nutrient import Nutrient, Nutrients


class Food(Enum):
    BROCOLLI = auto()
    CANOLA_OIL = auto()
    CARROT = auto()
    CHICKEN_BREAST = auto()
    PORK_LIVER = auto()
    SALT = auto()
    SWEET_POTATO = auto()
    SOYBEAN = auto()
    LETTUS = auto()
    BANANA = auto()
    BARF = auto()
    BALANCEIT= auto()


GETTERS = {
    Food.BROCOLLI: chinanutri(465),
    Food.CANOLA_OIL: chinanutri(1495),
    Food.CARROT: chinanutri(380),
    Food.CHICKEN_BREAST: chinanutri(880),
    Food.PORK_LIVER: chinanutri(797),
    Food.SALT: chinanutri(1565),
    Food.SWEET_POTATO: chinanutri(316),
    Food.SOYBEAN: chinanutri(326),
    Food.LETTUS: chinanutri(482),
    Food.BANANA: chinanutri(726),
    Food.BARF: {
        Nutrient.ASH: 182 * G / KG,
        Nutrient.FIBER: 70 * G / KG,
        Nutrient.PROTEIN: 27.5 * G / KG,
        Nutrient.FAT: 4.7 * G / KG,
        Nutrient.VITAMIN_A: 300 * MCG / KG,
        Nutrient.VITAMIN_C: 20 * MG / KG,
        Nutrient.VITAMIN_B1: 22 * MG / KG,
        Nutrient.VITAMIN_B2: 10 * MG / KG,
        Nutrient.VITAMIN_B5: 13 * MG / KG,
        Nutrient.VITAMIN_B6: 16 * MG / KG,
        Nutrient.VITAMIN_B7: 436 * MCG / KG,
        Nutrient.VITAMIN_B12: 7 * MG / KG,
        Nutrient.VITAMIN_D: 7.5 * MCG / KG,  # D3
        Nutrient.VITAMIN_E: 23 * MG / KG, 
        Nutrient.FOLIC_ACID: 2.05 * MG / KG,
        Nutrient.NIACIN: 118 * MG / KG,
        Nutrient.SODIUM: 1800 * MG / KG,
        Nutrient.POTASSIUM: 16500 * MG / KG,
        Nutrient.MAGNESIUM: 4800 * MG / KG,
        Nutrient.CALCIUM: 45600 * MG / KG,
        Nutrient.PHOSPHORUS: 7600 * MG / KG,
        Nutrient.COPPER: 8.78 * MG / KG,
        Nutrient.IRON: 614 * MG / KG,
        Nutrient.ZINC: 37.5 * MG / KG,
        Nutrient.MANGANESE: 45.6 * MG / KG,
        Nutrient.IODINE: 28.4 * MG / KG,
    },
    Food.BALANCEIT : {
        Nutrient.CALCIUM: 3.036* G / 20,
        Nutrient.PHOSPHORUS: 1.596* G / 20,
        Nutrient.POTASSIUM: 2.0184* G / 20,
        Nutrient.SODIUM: 0.208* G / 20,
        Nutrient.CHLORIDE: 0.3504* G / 20,
        Nutrient.MAGNESIUM: 0.16* G / 20,
        Nutrient.IRON: 32.92* MG / 20,
        Nutrient.COPPER: 3.0264* MG / 20,
        Nutrient.MANGANESE: 2.264 * MG / 20,
        Nutrient.ZINC: 53* MG / 20,
        Nutrient.IODINE: 0.6864* MG / 20,
        Nutrient.SELENIUM: 0.072* MG / 20,
        Nutrient.VITAMIN_A: 2268*0.3* MCG / 20,
        Nutrient.VITAMIN_D: 220*0.025 * MCG / 20,
        Nutrient.VITAMIN_E: 89.6*0.67* MG / 20, 
        Nutrient.VITAMIN_B1: 0.6424* MG / 20,
        Nutrient.RIBOFLAVIN: 1.82* MG / 20,
        Nutrient.VITAMIN_B5: 3.472* MG / 20,
        Nutrient.NIACIN: 4.8064* MG / 20,
        Nutrient.FOLIC_ACID: 0.12* MG / 20,
        Nutrient.VITAMIN_B12: 0.0114* MG / 20,
        Nutrient.CHOLINE: 490.18* MG / 20,
    },
}


def get_or_load(food: Food) -> Nutrients:
    getter = GETTERS[food]
    if isinstance(getter, dict):
        return getter
    elif callable(getter):
        pass
    else:
        raise NotImplementedError

    Path("foods").mkdir(parents=True, exist_ok=True)

    try:
        with open(f"foods/{food.name}.json") as f:
            serde = load(f)
            nuts = {Nutrient[k]: v for k, v in serde.items()}
    except FileNotFoundError:
        pass

    nuts = getter()

    with open(f"foods/{food.name}.json", "w+") as f:
        dump({k.name: v for k, v in nuts.items()}, f, indent=4)

    return nuts
