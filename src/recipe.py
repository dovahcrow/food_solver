from enum import Enum, auto
from typing import Dict, List, Optional, Tuple
import numpy as np
import cvxopt

from .units import MCG, MG, G
from .food import Food, get_or_load
from .nutrient import Nutrient, Nutrients, nutrient_value

# https://cvxopt.org/userguide/coneprog.html#quadratic-programming


class NeedSoftness(Enum):
    SOFT = auto()
    HARD = auto()


class NeedRequired(Enum):
    REQUIRED = auto()
    NOT_REQUIRED = auto()


class RecipeSolver:
    food_limits: Dict[Food, Tuple[float, float]]
    food_nutrients: Dict[Food, Nutrients]
    food_minimize_usage: Dict[Food, bool]
    food_names: List[Food]
    needs: Dict[Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]]

    def __init__(self) -> None:
        self.food_limits = {}
        self.food_nutrients = {}
        self.needs = {}
        self.food_names = []
        self.food_minimize_usage = {}
        self.nutrient_ratios: Dict[Tuple[Nutrient, Nutrient], Tuple[float, float]] = {}

    def add_nutrient_ratio(
        self, numerator: Nutrient, denominator: Nutrient, lb: float, ub: float,
    ) -> None:
        """Add a hard mass-ratio interval; both nutrients must use the same unit.

        Keep a positive absolute requirement for the denominator separately:
        the linear inequalities alone also permit both totals to be zero.
        Ratio bounds are dimensionless and must not be multiplied by days.
        """
        if numerator == denominator:
            raise ValueError("Ratio nutrients must be different")
        if not np.isfinite(lb) or not np.isfinite(ub) or not 0 < lb <= ub:
            raise ValueError("Ratio bounds must be finite and satisfy 0 < lb <= ub")
        self.nutrient_ratios[numerator, denominator] = (lb, ub)

    def add_food(self, food: Food, lb: float, ub: float, minimize_usage: bool = False):
        if food not in self.food_nutrients:
            self.food_nutrients[food] = get_or_load(food)
            self.food_limits[food] = (lb, ub)
            self.food_minimize_usage[food] = minimize_usage

    def add_need(
        self,
        nut: Nutrient,
        lb: float,
        ub: Optional[float],
        required: NeedRequired,
        hard: NeedSoftness,
    ):
        if nut in self.needs:
            return

        self.needs[nut] = (lb, ub, required, hard)

    def solve(self) -> bool:
        self.food_names = foods = list(self.food_limits.keys())

        G = np.zeros((len(foods) * 2, len(foods)))
        h = np.zeros(len(foods) * 2)

        for i, food in enumerate(foods):
            lb, ub = self.food_limits[food]
            # The food should be bigger then lb, aka -food <= -lb
            G[i, i] = -1
            h[i] = -lb

            # The food should be less then ub, aka food <= ub
            G[i + len(foods), i] = 1
            h[i + len(foods)] = ub

        P = np.zeros((len(foods), len(foods)))
        q = np.zeros((1, len(foods)))

        # PROBLEM AND NOTATION
        # --------------------
        # Choose amounts of f foods to meet nutrient needs. Throughout this
        # derivation, vectors x and q are columns; H_i is a row vector.
        #
        #   x_j    = grams of food j in the entire recipe/batch
        #   H_ij   = amount of nutrient i per gram of food j
        #   H_i    = [H_i0, ..., H_i(f-1)], shape (1, f)
        #   H_i x  = sum_j H_ij*x_j = total amount of nutrient i supplied
        #   m_i    = positive target for nutrient i (called `mid` below)
        #
        # Nutrient masses use grams, energy uses joules. H_i x and m_i must
        # have matching units and cover the same number of days.
        # Only REQUIRED + SOFT nutrients enter the nutrient objective below.
        # REQUIRED + HARD nutrients instead become inequality constraints;
        # NOT_REQUIRED nutrients are skipped, including their bounds.
        #
        # 1. ORIGINAL OBJECTIVE: ABSOLUTE SQUARED ERROR
        # --------------------------------------------
        # Start with: minimize sum_i (sum_j H_ij*x_j - m_i)^2.
        # For one nutrient, expanding the square gives:
        #
        #   (H_i x - m_i)^2
        #     = (H_i x)^2 - 2*m_i*H_i x + m_i^2
        #     = x.T (H_i.T H_i) x - 2*m_i*H_i x + m_i^2.
        #
        # H_i.T H_i is an outer product, an (f, f) matrix whose (j, k)
        # entry is H_ij*H_ik; it includes cross terms between foods.
        # This absolute-error objective depends on measurement units:
        # a 1 g error contributes 1, while a 1 mg error contributes 1e-6
        # when both are stored in grams. Macronutrients can dominate.
        #
        # 2. IMPLEMENTED OBJECTIVE: RELATIVE SQUARED ERROR
        # -----------------------------------------------
        # Divide EACH nutrient's error by its own target before squaring:
        #
        #   minimize sum_i ((H_i x - m_i) / m_i)^2
        #          = sum_i (H_i x / m_i - 1)^2.
        #
        # The "1" is m_i/m_i, not an extra nutrient target. A 10% error
        # contributes 0.01 for any nutrient, regardless of its mass/unit.
        # This changes the relative weights: it is the original objective
        # weighted by 1/m_i^2, NOT an equivalent algebraic rewrite of it.
        #
        # Define R_i = H_i/m_i (a normalized row). Expanding again:
        #
        #   (R_i x - 1)^2 = x.T (R_i.T R_i) x - 2*R_i x + 1.
        #
        # 3. MATCH THE COEFFICIENTS TO CVXOPT
        # ----------------------------------
        # cvxopt.solvers.qp(P, q, G, h) solves:
        #
        #   minimize    0.5*x.T P x + q.T x
        #   subject to  G x <= h.
        #
        # Its quadratic matrix is called P and its linear vector is q
        # (lowercase). Matching the preceding expansion, each nutrient adds:
        #
        #   P_i =  2*R_i.T R_i =  2*H_i.T H_i / m_i^2
        #   q_i = -2*R_i.T     = -2*H_i.T / m_i.
        #
        # Sum these contributions over all participating soft nutrients.
        # The factor 2 in P cancels CVXOPT's factor 0.5. The constant +1
        # per nutrient can be dropped because it cannot change the best x.
        # P is positive semidefinite (a sum of outer products), so the
        # objective is convex, though the best recipe need not be unique.
        #
        # For comparison, the UNNORMALIZED objective would require BOTH:
        #   P_i = 2*H_i.T H_i,    q_i = -2*m_i*H_i.T.
        # Do not mix these formulas: normalizing P but not q changes the
        # target. Example: H_i=[0.001], m_i=0.0105 should select x=10.5 g
        # in the absence of other objectives or binding constraints.
        #
        # IMPLEMENTATION DETAILS AND CONSTRAINTS
        # --------------------------------------
        # Below, `H` starts as a 1-D array of raw nutrient densities. The
        # assignment H = H[None, :] / mid turns it into R_i, shape (1, f).
        # `@` is matrix multiplication. We accumulate q as a (1, f) ROW
        # for convenience, then transpose it into CVXOPT's (f, 1) column.
        # P is symmetric; its later transpose does not change its values.
        #
        # If no upper bound is given, m_i = 1.05*lb; otherwise m_i is the
        # midpoint of lb and ub. These are preferences, not hard limits:
        # this objective penalizes deviations even INSIDE the interval,
        # and it permits shortfalls/excesses when other terms compete.
        # A missing nutrient recorded as zero cannot be supplied by that
        # food in the model. If an entire row is zero, its error is constant
        # and adds no preference; a soft target does not make this infeasible.
        #
        # Hard bounds are expressed separately in G x <= h:
        #   food lb <= x_j <= ub:       -x_j <= -lb,    x_j <= ub
        #   nutrient lb <= H_i x <= ub: -H_i x <= -lb, H_i x <= ub.
        # Omit the upper inequality when ub is None. These use RAW H_i,
        # not normalized R_i. An "optimal" status certifies this mathematical
        # problem was solved, not that every soft nutrient bound was met.
        for need, (lb, ub, required, hard) in self.needs.items():
            # if isinstance(self.provides[n], int) and self.provides[n] == 0:
            #     logging.info(f"WARN: Food lacks {n}")
            #     continue
            if required != NeedRequired.REQUIRED:
                continue

            H = np.asarray([nutrient_value(self.food_nutrients[food], need)[0] for food in foods])

            if hard == NeedSoftness.HARD:
                # The nutrient should be bigger then lb, aka -has <= -lb
                G = np.vstack([G, -H])
                h = np.append(h, -lb)
                if ub is not None:
                    # The nutrient should be less then ub
                    G = np.vstack([G, H])
                    h = np.append(h, ub)
            else:
                if ub is None:
                    mid = lb * 1.05
                else:
                    mid = (lb + ub) / 2

                if not np.isfinite(mid) or mid <= 0:
                    raise ValueError(f"Soft target for {need.name} must be positive and finite")
                H = H[None, :] / mid
                P += 2 * H.T @ H
                q += -2 * H

        for (numerator, denominator), (lb, ub) in self.nutrient_ratios.items():
            N = np.asarray([self.food_nutrients[f].get(numerator, 0) for f in foods])
            D = np.asarray([self.food_nutrients[f].get(denominator, 0) for f in foods])
            # N and D are raw nutrient mass per gram of each food, NOT the
            # normalized objective rows above. For a positive total D @ x:
            #
            #   lb <= (N @ x)/(D @ x) <= ub
            #   N @ x >= lb*(D @ x)  ->  (lb*D - N) @ x <= 0
            #   N @ x <= ub*(D @ x)  ->  (N - ub*D) @ x <= 0
            #
            # Thus the ratio adds two linear rows to G x <= h, with h=0;
            # it does not change the quadratic objective P or q.
            # Adult dog Ca:P=1:1..2:1 gives (D-N)@x<=0, (N-2*D)@x<=0.
            # Absolute calcium/phosphorus bounds still apply independently.
            G = np.vstack([G, lb * D - N, N - ub * D])
            h = np.append(h, [0., 0.])

        for i, food in enumerate(foods):
            if self.food_minimize_usage[food]:
                # Penalize the fraction of available stock, not grams squared.
                # Scaling both inventory and needs preserves recipe proportions.
                # Desired extra objective: 0.05*(x_i/ub)**2.
                # Since CVXOPT uses 0.5*x.T P x, add 0.1/ub**2 to P[i, i].
                # There is no linear contribution to q. This preference can
                # trade off against nutrient targets. A zero upper bound
                # already fixes the food at zero (for valid nonnegative bounds).
                ub = self.food_limits[food][1]
                if not np.isfinite(ub) or ub < 0:
                    raise ValueError("minimize_usage requires a finite nonnegative upper bound")
                if ub > 0:
                    P[i, i] += 0.1 / ub**2
            
        G = cvxopt.matrix(G)
        h = cvxopt.matrix(h)
        P = cvxopt.matrix(P.T)
        q = cvxopt.matrix(q.T)

        self.sol = cvxopt.solvers.qp(P, q, G, h, options={"show_progress": False})

        return self.sol["status"] == "optimal"

    def print_foods(self):
        print("Solution:")

        for i, f in enumerate(self.food_names):
            amount = float(f"{self.amount(i):.2g}")
            print(f"  {f} = {amount:.1f}")

    def amount(self, food: Food | int):
        if isinstance(food, Food):
            food = self.food_names.index(food)

        return self.sol["x"][food]

    def print_nutrition(
        self,
        needs: Dict[
            Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]
        ],
        day: int = 1,
        detail: bool = False,
    ):
        print("Nutrition:")
        for n in Nutrient:
            if n not in needs:
                continue

            lb, ub, required, _ = needs[n]
            ub = ub or float("+INF")

            value = 0
            comp = []
            missing = []
            for i, f in enumerate(self.food_names):
                nut, known = nutrient_value(self.food_nutrients[f], n)
                if not known and self.amount(i) > 1e-6:
                    missing.append(f.name)
                if nut != 0:
                    comp.append((f, self.amount(i) * nut))
                value += self.amount(i) * nut
            lb /= day
            ub /= day
            value /= day

            if n == Nutrient.ENERGY:
                scale = 1000
                unit = "kJ"
            else:
                if lb >= G:
                    scale = G
                    unit = "g"
                elif lb >= MG:
                    scale = MG
                    unit = "mg"
                else:
                    scale = MCG
                    unit = "μg"

            value /= scale
            lb /= scale
            ub /= scale

            valid = lb <= value <= ub

            if detail:
                comp_str = " = " + " + ".join(
                    [f"{f.name} {v / scale:g} {unit}" for f, v in comp if v != 0]
                )
            else:
                comp_str = ""

            if len(comp) != 1 or not detail:
                comp_str += f" = {value:g} {unit}"

            color = ""
            if required != NeedRequired.REQUIRED:
                if not valid:
                    color = BColors.LIGHT_YELLOW
                else:
                    color = BColors.GRAY
            elif not valid:
                color = BColors.RED
            else:
                color = BColors.LIGHT_GREEN

            coverage = f"; incomplete data: {', '.join(missing)}" if missing else ""
            print(
                f"{color}  {n}{comp_str}, valid: {lb:.2f} ~ {ub:.2f} {unit}{coverage}{BColors.ENDC}"
            )


class BColors:
    HEADER = "\033[95m"
    RED = "\033[31m"
    YELLOW = "\033[33m"
    GRAY = "\033[37m"
    OKBLUE = "\033[94m"
    OKCYAN = "\033[96m"
    GREEN = "\033[32m"
    LIGHT_GREEN = "\033[92m"
    LIGHT_YELLOW = "\033[93m"
    LIGHT_RED = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"
