from enum import Enum, auto
from typing import Dict, List, Optional, Tuple
import numpy as np
import cvxpy as cp

from .units import MCG, MG, G
from .food import Food, get_or_load
from .nutrient import Nutrient, Nutrients, nutrient_value

# https://www.cvxpy.org/tutorial/dcp/index.html


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

    def __init__(self, implicit_soft_upper_multiplier: Optional[float] = 1.5) -> None:
        """Create a recipe solver.

        ``implicit_soft_upper_multiplier`` controls open-ended requirements.
        If a required nutrient has a positive minimum ``L`` and no explicit
        maximum, supply above ``multiplier * L`` receives the same relative
        squared excess penalty used by an explicit soft upper bound.  It is a
        preference, not a hard safety limit, so a constrained recipe can still
        exceed it.  Pass ``None`` to retain a truly open-ended requirement.
        """
        if implicit_soft_upper_multiplier is not None and (
            not np.isfinite(implicit_soft_upper_multiplier)
            or implicit_soft_upper_multiplier <= 1
        ):
            raise ValueError(
                "implicit_soft_upper_multiplier must be finite and greater than 1, or None"
            )
        self.food_limits = {}
        self.food_nutrients = {}
        self.needs = {}
        self.food_names = []
        self.food_minimize_usage = {}
        self.problem = None
        self.sol = {"status": "not_solved", "x": None, "primal objective": None}
        self.nutrient_ratios: Dict[Tuple[Nutrient, Nutrient], Tuple[float, float]] = {}
        self.implicit_soft_upper_multiplier = implicit_soft_upper_multiplier

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

    def effective_upper_bound(self, lb: float, ub: Optional[float]) -> Optional[float]:
        """Return the explicit upper bound or the configured implicit soft one."""
        if ub is not None:
            return ub
        if lb > 0 and self.implicit_soft_upper_multiplier is not None:
            return lb * self.implicit_soft_upper_multiplier
        return None

    def solve(self) -> bool:
        self.food_names = foods = list(self.food_limits)
        # Clear previous results: an infeasible subsequent solve must not
        # expose food quantities left over from an earlier successful solve.
        self.problem = None
        self.sol = {"status": "not_solved", "x": None, "primal objective": None}
        if not foods:
            raise ValueError("Add at least one food before solving")

        # x[j] is grams of food j in the batch, shape (number of foods,).
        # H[j] is nutrient mass (g), or energy (J), per gram of food j.
        # H @ x is total supply; bounds must use the same units/batch size.
        # Use @ for a dot product, not * (elementwise multiplication).
        x = cp.Variable(len(foods), nonneg=True, name="food_grams")
        constraints = []
        penalties = []
        for j, food in enumerate(foods):
            lb, ub = self.food_limits[food]
            if not np.isfinite(lb) or not np.isfinite(ub) or not 0 <= lb <= ub:
                raise ValueError(f"Invalid food bounds for {food.name}")
            constraints.extend([x[j] >= lb, x[j] <= ub])
            if self.food_minimize_usage[food] and ub > 0:
                # Old normalized QP diagonal: P[j,j] += 0.1/ub**2.
                # Since a QP objective has a factor 1/2, this is 0.05*(x/ub)^2.
                # Penalize stock fractions so scaling the batch changes no weights.
                penalties.append(0.05 * cp.square(x[j] / ub))

        # SOFT INTERVAL OBJECTIVE (our agreed relative boundary penalties)
        # ----------------------------------------------------------------
        # For supply y=H@x, lower bound L and optional upper bound U:
        #   loss = max(0, 1-y/L)^2 + max(0, y/U-1)^2.
        # Below L, penalize shortage relative to L. Above U, penalize excess
        # relative to U. Inside [L,U], including endpoints, the loss is zero.
        # E.g. L=10 mg, U=100 mg: 9 mg and 110 mg each cost 0.01.
        # If the caller supplies no upper bound, the solver normally creates a
        # SOFT preference U = implicit_soft_upper_multiplier * L. This keeps
        # open-ended requirements such as protein from drifting arbitrarily far
        # above their minimum merely because another food is useful. It remains
        # feasible to exceed U: the objective only adds an excess penalty.
        # Pass multiplier=None to omit this implicit preference. For L=0 we
        # cannot derive U and skip it. An explicit U always takes precedence.
        # This is not the historical (y/mid-1)^2 midpoint preference.
        #
        # CVXPY's cp.pos(a) means max(a,0); cp.square(cp.pos(a)) is convex:
        # pos is convex and nonnegative, and square is increasing there.
        # CVXPY automatically introduces auxiliary variables during canonicalization.
        # For shortage s and excess t the equivalent explicit QP is:
        #   minimize s^2+t^2
        #   s>=0, s>=1-H@x/L; t>=0, t>=H@x/U-1.
        # For z=[x,s,t], the standard 0.5*z.T@P@z + q.T@z has diagonal
        # P[s,s]=P[t,t]=2, q=0, and linear constraints G@z<=h:
        #   [-H/L, -1,  0] @ z <= -1
        #   [ H/U,  0, -1] @ z <=  1
        #   [   0, -1,  0] @ z <=  0
        #   [   0,  0, -1] @ z <=  0.
        # We no longer manually assemble or transpose P, q, G and h.
        #
        # HARD requirements remain inequalities; NOT_REQUIRED is skipped.
        # Missing data contributes zero without inserting keys into a defaultdict.
        # A missing soft row has constant shortage loss, not infeasibility.
        for need, (lb, ub, required, hard) in self.needs.items():
            if required != NeedRequired.REQUIRED:
                continue
            if not np.isfinite(lb) or lb < 0:
                raise ValueError(f"Invalid minimum for {need.name}")
            if ub is not None and (not np.isfinite(ub) or ub < lb):
                raise ValueError(f"Invalid maximum for {need.name}")
            H = np.asarray([nutrient_value(self.food_nutrients[f], need)[0] for f in foods])
            supply = H @ x
            effective_ub = self.effective_upper_bound(lb, ub)
            if hard == NeedSoftness.HARD:
                # Scale each row without changing the feasible set: energy is
                # in millions of joules while trace nutrients may be micrograms.
                normalizer = max(lb, ub or 0., float(np.max(np.abs(H)))) or 1.
                constraints.append(supply / normalizer >= lb / normalizer)
                if ub is not None:
                    constraints.append(supply / normalizer <= ub / normalizer)
            else:
                if lb > 0:
                    penalties.append(cp.square(cp.pos(1 - supply / lb)))
            # Excess is a soft preference for both SOFT and HARD lower bounds.
            # HARD controls feasibility; this term only ranks feasible recipes.
            if effective_ub is not None:
                if effective_ub <= 0:
                    raise ValueError(f"Soft maximum for {need.name} must be positive")
                penalties.append(cp.square(cp.pos(supply / effective_ub - 1)))

        for (numerator, denominator), (lb, ub) in self.nutrient_ratios.items():
            N = np.asarray([nutrient_value(self.food_nutrients[f], numerator)[0] for f in foods])
            D = np.asarray([nutrient_value(self.food_nutrients[f], denominator)[0] for f in foods])
            # For a positive denominator total, lb <= (N@x)/(D@x) <= ub
            # is equivalent to two LINEAR inequalities. Do not divide two
            # CVXPY expressions: that would obscure the convex formulation.
            # Keep the denominator's positive absolute requirement separately.
            # Adult Ca:P 1..2 means Ca >= P and Ca <= 2*P.
            constraints.extend([N @ x >= lb * (D @ x), N @ x <= ub * (D @ x)])

        objective = cp.Minimize(sum(penalties, cp.Constant(0.)))
        self.problem = cp.Problem(objective, constraints)
        # CLARABEL ships with CVXPY; no CVXOPT dependency or license is needed.
        # Solver errors propagate, while infeasibility returns False. Do not
        # silently accept OPTIMAL_INACCURATE as a verified optimal solution.
        self.problem.solve(solver=cp.CLARABEL, verbose=False)
        optimal = self.problem.status == cp.OPTIMAL
        # Small compatibility view for existing callers. The native CVXPY
        # result is available as problem.status / problem.value; x.value is
        # already a food-only vector, with no auxiliary variables to strip.
        self.sol = {
            "status": self.problem.status,
            "x": np.array(x.value, copy=True) if optimal else None,
            "primal objective": self.problem.value,
        }
        return optimal

    def print_foods(self):
        print("Solution:")

        for i, f in enumerate(self.food_names):
            amount = float(f"{self.amount(i):.2g}")
            print(f"  {f} = {amount:.1f}")

    def amount(self, food: Food | int):
        if isinstance(food, Food):
            food = self.food_names.index(food)

        if self.sol["x"] is None:
            raise RuntimeError("No optimal recipe is available; call solve() successfully first")
        return float(self.sol["x"][food])

    def print_nutrition(
        self,
        needs: Dict[
            Nutrient, Tuple[float, Optional[float], NeedRequired, NeedSoftness]
        ],
        day: int = 1,
        detail: bool = False,
    ):
        print("Nutrition (per day):")
        for n in Nutrient:
            if n not in needs:
                continue

            lb, explicit_ub, required, _ = needs[n]
            effective_ub = self.effective_upper_bound(lb, explicit_ub)

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
            display_ub = (
                effective_ub / day if effective_ub is not None else float("+INF")
            )
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
            display_ub /= scale

            below = value < lb
            above = value > display_ub

            if detail:
                # comp stores batch contributions; convert them to the same
                # daily basis as value and the displayed requirement bounds.
                comp_str = " = " + " + ".join(
                    [
                        f"{f.name} {v / day / scale:g} {unit}"
                        for f, v in comp
                        if v != 0
                    ]
                )
            else:
                comp_str = ""

            if len(comp) != 1 or not detail:
                comp_str += f" = {value:g} {unit}"

            # Required deficits are red because a minimum is missed. Excess is
            # yellow: it may be above an explicit or implicit preferred upper
            # bound, but soft excess alone does not make the QP infeasible.
            # Optional/not-required nutrients use muted variants.
            if required != NeedRequired.REQUIRED:
                if below:
                    color = BColors.LIGHT_YELLOW
                    status = "below minimum"
                elif above:
                    color = BColors.YELLOW
                    status = "above maximum"
                else:
                    color = BColors.GRAY
                    status = "within range"
            elif below:
                color = BColors.RED
                status = "below minimum"
            elif above:
                color = BColors.YELLOW
                status = "above maximum"
            else:
                color = BColors.LIGHT_GREEN
                status = "within range"

            coverage = f"; incomplete data: {', '.join(missing)}" if missing else ""
            upper_source = (
                f"; implicit soft upper {self.implicit_soft_upper_multiplier:g}x"
                if explicit_ub is None and effective_ub is not None
                else ""
            )
            print(
                f"{color}  {n}{comp_str}, {status}: "
                f"{lb:.2f} ~ {display_ub:.2f} {unit}{upper_source}{coverage}{BColors.ENDC}"
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
