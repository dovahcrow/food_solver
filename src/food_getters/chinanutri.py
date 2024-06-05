# FROM https://nlc.chinanutri.cn/fq/

import re
from collections.abc import Callable
from typing import Dict

from bs4 import BeautifulSoup
import requests

from ..nutrient import Nutrient
from ..units import G, KCAL, MCG, MG, KJ


def chinanutri(id: int) -> Callable[[], Dict[Nutrient, float]]:
    def inner() -> Dict[Nutrient, float]:
        ret = {}

        resp = requests.get(f"https://nlc.chinanutri.cn/fq/foodinfo/{id}.html")
        soup = BeautifulSoup(resp.text, "html.parser")

        rows = soup.select("div.nutrition_table_content tr")
        for row in rows[1:]:
            iter = row.select("td")
            for i in range(len(iter)):
                elem = iter[i]
                if elem is None:
                    continue
                cls = elem.attrs.get("class", "")
                if "td_left" in cls:
                    break

            name = iter[i].text
            contains = iter[i + 1].text

            if contains == "":
                continue

            if name in NAME_IGNORED:
                continue

            nut = NAME_TO_ENUM[name]

            match = re.match(r"((\d*\.)?\d+)(g|mg|μg|kJ)$", contains)
            if match is None:
                continue
            amount = float(match.group(1))
            unit = match.group(3)
            # print(nut, amount, unit)

            amount = normalize(amount, unit)

            ret[nut] = amount / 100  # Food on this website is per 100g

        return ret

    return inner

def normalize(amount: float, unit: str) -> float:
    if unit == "g":
        amount *= G
    elif unit == "mg":
        amount *= MG
    elif unit == "μg":
        amount *= MCG
    elif unit == "µg":
        amount *= MCG
    elif unit == "kJ":
        amount *= KJ
    elif unit == "kcal":
        amount *= KCAL
    else:
        raise NotImplementedError(unit)

    return amount

NAME_TO_ENUM = {
    "能量(Energy)": Nutrient.ENERGY,
    "蛋白质(Protein)": Nutrient.PROTEIN,
    "脂肪(Fat)": Nutrient.FAT,
    "胆固醇(Cholesterol)": Nutrient.CHOLESTEROL,
    "灰分(Ash)": Nutrient.ASH,
    "碳水化合物(CHO)": Nutrient.CARB,
    "总膳食纤维(Dietary fiber)": Nutrient.FIBER,
    "胡萝卜素(Carotene)": Nutrient.CAROTENE,
    "维生素A(Vitamin)": Nutrient.VITAMIN_A,
    "硫胺素(Thiamin)": Nutrient.VITAMIN_B1,
    "核黄素(Riboflavin)": Nutrient.RIBOFLAVIN,
    "烟酸(Niacin)": Nutrient.NIACIN,
    "维生素C(Vitamin C)": Nutrient.VITAMIN_C,
    "钙(Ca)": Nutrient.CALCIUM,
    "磷(P)": Nutrient.PHOSPHORUS,
    "钾(K)": Nutrient.POTASSIUM,
    "钠(Na)": Nutrient.SODIUM,
    "镁(Mg)": Nutrient.MAGNESIUM,
    "铁(Fe)": Nutrient.IRON,
    "锌(Zn)": Nutrient.ZINC,
    "硒(Se)": Nutrient.SELENIUM,
    "铜(Cu)": Nutrient.COPPER,
    "锰(Mn)": Nutrient.MANGANESE,
    "碘(I)": Nutrient.IODINE,
}

NAME_IGNORED = {
    "水分(Water)",
    "合计(Total)",
    "饱和脂肪酸(SFA)",
    "单不饱和脂肪酸(MUFA)",
    "多不饱和脂肪酸(PUFA)",
    "α-TE",
    "食部(Edible)",
}
