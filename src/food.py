from json import load, dump
from pathlib import Path
from enum import Enum, auto, unique

from .food_getters.usda import convert_cooked_chicken_breast_to_uncoocked
from .food_getters.usda import usda
from .units import KG, MCG, MG, G
from .food_getters.chinanutri import chinanutri
from .nutrient import Nutrient, Nutrients


@unique
class Food(Enum):
    BALANCEIT = auto()
    BANANA = auto()
    BARF = auto()
    BAICAI = auto()
    BEEF = auto()
    BELL_PEPER = auto()
    BOCAI = auto() # Slightly different from spinach, the root of this one is red.
    BOKCHOY = auto()
    BROCCOLI = auto()
    CABBAGE = auto()
    CANOLA_OIL = auto()
    CARROT = auto()
    CELERY = auto()
    CHICKEN_BREAST = auto()
    CHICKEN_HEART = auto()
    CHICKEN_THIGH = auto()
    CHINESE_LETTUS = auto()
    CUCUMBER = auto()
    EGG = auto()
    EGGPLANT = auto()
    EGG_SHELL_POWDER = auto()
    LUOBO = auto() # O shape, fist size
    OYSTER = auto()
    OKRA = auto()
    PUMPKIN = auto()
    PORK = auto()
    PORK_INTESTINE = auto()
    PORK_LIVER = auto()
    PORK_HEART = auto()
    PORK_TONGUE = auto()
    PORK_FAT = auto()
    POTATO = auto()
    SALT = auto()
    RICE = auto()
    SOYBEAN_GREEN = auto()
    SOYBEAN_YELLOW = auto()
    SOY_MILK = auto()
    TOMATO = auto()
    SWEET_POTATO = auto()
    TOFU_FIRM = auto()
    ZUCCHINI = auto()


GETTERS = {
    Food.BALANCEIT: {
        Nutrient.CALCIUM: 3.036 * G / 20,
        Nutrient.PHOSPHORUS: 1.596 * G / 20,
        Nutrient.POTASSIUM: 2.0184 * G / 20,
        Nutrient.SODIUM: 0.208 * G / 20,
        Nutrient.CHLORIDE: 0.3504 * G / 20,
        Nutrient.MAGNESIUM: 0.16 * G / 20,
        Nutrient.IRON: 32.92 * MG / 20,
        Nutrient.COPPER: 3.0264 * MG / 20,
        Nutrient.MANGANESE: 2.264 * MG / 20,
        Nutrient.ZINC: 53 * MG / 20,
        Nutrient.IODINE: 0.6864 * MG / 20,
        Nutrient.SELENIUM: 0.072 * MG / 20,
        Nutrient.VITAMIN_A: 2268 * 0.3 * MCG / 20,
        Nutrient.VITAMIN_D: 220 * 0.025 * MCG / 20,
        Nutrient.VITAMIN_E: 89.6 * 0.67 * MG / 20,
        Nutrient.VITAMIN_B1: 0.6424 * MG / 20,
        Nutrient.VITAMIN_B2: 1.82 * MG / 20,
        Nutrient.VITAMIN_B5: 3.472 * MG / 20,
        Nutrient.NIACIN: 4.8064 * MG / 20,
        Nutrient.FOLIC_ACID: 0.12 * MG / 20,
        Nutrient.VITAMIN_B12: 0.0114 * MG / 20,
        Nutrient.CHOLINE: 490.18 * MG / 20,
    },
    Food.BAICAI: chinanutri(450),
    # Food.BANANA: chinanutri(726),
    Food.BANANA: usda(1105314),
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
        Nutrient.VITAMIN_B12: 7 * MCG / KG,
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
        Nutrient.SELENIUM: 0.36 * MG / KG,
        Nutrient.ZINC: 37.5 * MG / KG,
        Nutrient.MANGANESE: 45.6 * MG / KG,
        Nutrient.IODINE: 28.4 * MG / KG,
    },
    # Food.BROCCOLI: chinanutri(465),
    Food.BROCCOLI: usda(747447),
    Food.BEEF: usda(2646173),
    Food.BELL_PEPER: usda(2258590),
    Food.BOCAI: chinanutri(473),
    Food.BOKCHOY: usda(2685572),
    Food.CANOLA_OIL: chinanutri(1495),
    Food.CABBAGE: usda(2346407),
    # Food.CARROT: chinanutri(380),
    Food.CARROT: usda(2258586),
    Food.CELERY: usda(2346405),
    # Food.CHICKEN_BREAST: chinanutri(880),
    # Food.CHICKEN_BREAST: usda(2646170),
    Food.CHICKEN_BREAST: convert_cooked_chicken_breast_to_uncoocked(usda(331960)),
    Food.CHICKEN_THIGH: usda(2646171),
    Food.CHICKEN_HEART: chinanutri(885),
    Food.CHINESE_LETTUS: chinanutri(482),
    Food.CUCUMBER: usda(2346406),
    # Food.EGG: chinanutri(978),
    Food.EGG_SHELL_POWDER: {
        Nutrient.CALCIUM: 0.35 * G,
        Nutrient.MAGNESIUM: 0.014 * G
    },
    Food.EGG: usda(748967),
    # Food.EGGPLANT: chinanutri(404),
    Food.EGGPLANT: usda(2685577),
    Food.LUOBO: chinanutri(371),
    Food.OKRA: chinanutri(416),
    Food.OYSTER: chinanutri(1112),
    Food.PUMPKIN: chinanutri(426),
    Food.PORK: chinanutri(788),
    Food.PORK_FAT: chinanutri(780),
    Food.PORK_INTESTINE: chinanutri(790),
    Food.PORK_LIVER: chinanutri(797),
    Food.PORK_HEART: chinanutri(802),
    Food.PORK_TONGUE: chinanutri(799),
    Food.POTATO: usda(2346403),
    Food.RICE: usda(2512381),
    Food.SALT: chinanutri(1565),
    # Food.SWEET_POTATO: chinanutri(316),
    Food.SWEET_POTATO: usda(2346404),
    Food.SOYBEAN_GREEN: chinanutri(391),
    Food.SOYBEAN_YELLOW: chinanutri(326),
    Food.SOY_MILK: usda(1999630),
    Food.TOMATO: chinanutri(405),
    Food.TOFU_FIRM: chinanutri(334),
    # Food.ZUCCHINI: chinanutri(431),
    Food.ZUCCHINI: usda(2685568),
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
            nuts = {Nutrient[k]: v for k, v in serde.items() if k != "__name__"}
            return nuts
    except FileNotFoundError:
        pass

    print(f"Getting nutrients for {food.name}")
    name, nuts = getter()

    with open(f"foods/{food.name}.json", "w+") as f:
        dump(
            {"__name__": name, **{k.name: v for k, v in nuts.items()}},
            f,
            indent=4,
            sort_keys=True,
            ensure_ascii=False,
        )

    return nuts
