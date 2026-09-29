//! The convex batch solver and its structured nutrition report.
//!
//! This mirrors the Python reference formulation: a quadratic program with
//! relative-boundary squared losses, solved by [Clarabel] — the same
//! interior-point solver CVXPY used.
//!
//! For a nutrient supply `y = H @ x`, minimum `L` and optional maximum `U`:
//!
//! ```text
//! loss = max(0, 1 - y/L)^2 + max(0, y/U - 1)^2
//! ```
//!
//! Below `L` the shortage is penalised relative to `L`; above `U` the excess
//! is penalised relative to `U`; inside `[L, U]` the loss is zero. With no
//! explicit `U`, an open-ended *soft* requirement gets a soft preference
//! `U = multiplier * L`, so protein-like minima do not drift arbitrarily high.
//! That preference only ranks feasible recipes; it never makes one infeasible.
//! A hard requirement without an explicit maximum is genuinely open-ended:
//! the implicit preference is a soft-objective device, so it does not apply.
//!
//! [Clarabel]: https://github.com/oxfordcontrol/Clarabel.rs

use clarabel::algebra::CscMatrix;
use clarabel::solver::{DefaultSettings, DefaultSolver, IPSolver, NonnegativeConeT, SolverStatus};

use anyhow::{anyhow, Error};
use culpa::{throw, throws};

use crate::foods::{Food, FoodName};
use crate::nutrient::{nutrient_value, Nutrient};
use crate::units::{G, MCG, MG};

/// Whether a nutrient must be satisfied, or is only reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedRequired {
    Required,
    NotRequired,
}

/// Whether a requirement bounds feasibility or only ranks solutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedSoftness {
    Soft,
    Hard,
}

/// One requirement: minimum, optional explicit maximum, and its kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Requirement {
    pub minimum: f64,
    pub maximum: Option<f64>,
    pub required: NeedRequired,
    pub softness: NeedSoftness,
}

/// One ingredient of the batch and its per-gram composition.
#[derive(Debug, Clone)]
pub struct FoodRow {
    pub name: FoodName,
    pub values: Food,
    /// Use exactly `lower` grams when `lower == upper`, otherwise any amount
    /// in `[lower, upper]`.
    pub lower: f64,
    pub upper: f64,
    /// Prefer less of this ingredient among otherwise equivalent recipes.
    pub minimize_usage: bool,
}

/// A nutrient mass ratio that must stay within `[minimum, maximum]`.
#[derive(Debug, Clone, Copy)]
pub struct RatioConstraint {
    pub numerator: Nutrient,
    pub denominator: Nutrient,
    pub minimum: f64,
    pub maximum: f64,
}

/// Solver inputs.
#[derive(Debug, Clone)]
pub struct Problem {
    pub foods: Vec<FoodRow>,
    pub needs: Vec<(Nutrient, Requirement)>,
    pub ratios: Vec<RatioConstraint>,
    /// `Some(m)` gives open-ended minima a soft maximum of `m * L`; `None`
    /// leaves them genuinely open-ended.
    pub implicit_soft_upper_multiplier: Option<f64>,
}

impl Problem {
    /// The bound above which this requirement pays an excess penalty.
    ///
    /// An explicit maximum always applies. The implicit `multiplier · L`
    /// preference exists to rank soft recipes, so it is only used for a soft
    /// requirement; a hard requirement without an explicit maximum is
    /// genuinely open-ended and pays nothing for excess.
    pub fn effective_upper_bound(&self, requirement: &Requirement) -> Option<f64> {
        if requirement.maximum.is_some() {
            return requirement.maximum;
        }
        if requirement.softness == NeedSoftness::Hard {
            return None;
        }
        match self.implicit_soft_upper_multiplier {
            Some(multiplier) if requirement.minimum > 0.0 => Some(requirement.minimum * multiplier),
            _ => None,
        }
    }
}

/// Solver termination, collapsed to what the frontends care about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveStatus {
    Optimal,
    Infeasible,
    /// Any other termination, carrying Clarabel's status text.
    Other(String),
}

impl SolveStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SolveStatus::Optimal => "optimal",
            SolveStatus::Infeasible => "infeasible",
            SolveStatus::Other(text) => text,
        }
    }
}

/// Outcome of a solve.
#[derive(Debug, Clone)]
pub struct Solution {
    pub status: SolveStatus,
    /// Batch grams per food, in `Problem::foods` order; only for an optimal
    /// solve.
    pub grams: Option<Vec<f64>>,
    pub objective: Option<f64>,
}

impl Solution {
    pub fn is_optimal(&self) -> bool {
        matches!(self.status, SolveStatus::Optimal)
    }
}

/// One row of `A z <= b`.
struct Row {
    terms: Vec<(usize, f64)>,
    bound: f64,
}

impl Row {
    fn new() -> Self {
        Self {
            terms: Vec::new(),
            bound: 0.0,
        }
    }

    fn push(&mut self, column: usize, value: f64) {
        if value != 0.0 {
            self.terms.push((column, value));
        }
    }
}

fn csc_from_rows(nrows: usize, ncols: usize, rows: &[Row]) -> CscMatrix<f64> {
    let mut col_counts = vec![0usize; ncols];
    for row in rows {
        for (col, _) in &row.terms {
            col_counts[*col] += 1;
        }
    }
    let mut colptr = vec![0usize; ncols + 1];
    for col in 0..ncols {
        colptr[col + 1] = colptr[col] + col_counts[col];
    }
    let mut cursor = colptr.clone();
    let rowval = vec![0usize; rows.iter().map(|r| r.terms.len()).sum()];
    let mut rowval = rowval;
    let mut nzval = vec![0.0; rowval.len()];
    for (index, row) in rows.iter().enumerate() {
        for (col, value) in &row.terms {
            let slot = cursor[*col];
            rowval[slot] = index;
            nzval[slot] = *value;
            cursor[*col] += 1;
        }
    }
    CscMatrix::new(nrows, ncols, colptr, rowval, nzval)
}

/// Solve the batch problem.
///
/// # The original problem
///
/// Let `x_j` be the batch grams of food `j` (`n` foods, one variable each), and
/// let `h_i` be the nutrient-`i` density row, i.e. the grams of nutrient `i`
/// (joules for energy) each food supplies per gram. Then the batch supplies
/// `y_i = h_i · x` of nutrient `i`. Every required nutrient has a minimum `L_i`
/// and an optional explicit maximum `U_i`, and every food has a gram range
/// `[lower_j, upper_j]` that pins the weight when the two ends are equal.
///
/// The recipe we want is the one that best meets those requirements:
///
/// ```text
/// minimize   Σ_i soft_loss_i(y_i)  +  Σ_j 0.05 · (x_j / upper_j)²
///
/// subject to y_i >= L_i                       for each HARD requirement
///            y_i <= U_i                       for each HARD requirement with a maximum
///            lower_j <= x_j <= upper_j        for every food
///            L_r <= (N_r·x) / (D_r·x) <= U_r  for every nutrient ratio
/// ```
///
/// with the relative-boundary loss
///
/// ```text
/// soft_loss_i(y) = max(0, 1 - y/L_i)² + max(0, y/U_i - 1)²
/// ```
///
/// where `U_i` is the explicit maximum or, for an open-ended soft minimum, the
/// implicit preference `multiplier · L_i`. A hard requirement has no implicit
/// maximum: without an explicit one its excess term is absent entirely, so
/// `soft_loss_i` is then just the shortage side. A shortage is measured
/// against `L_i`, an excess against `U_i`, and the whole interval
/// `[L_i, U_i]` costs nothing, so the loss is zero at both ends and inside it. The small
/// `0.05 · (x_j / upper_j)²` term does not encode a requirement at all: it only
/// breaks ties between otherwise equivalent recipes in favour of using less of
/// a `minimize_usage` ingredient.
///
/// This is a convex quadratic program, not a linear one: each `soft_loss_i` is a
/// convex, non-negative, non-decreasing function of the linear form `y_i`, and
/// every constraint is linear. `max(0, ·)` preserves convexity here because the
/// squaring is monotone on the non-negative side, and sums and non-negative
/// scalings of convex functions stay convex.
///
/// # Clarabel's standard form
///
/// [Clarabel] solves
///
/// ```text
/// minimize   (1/2) z'Pz + q'z
/// subject to A z + s = b,   s ∈ K
/// ```
///
/// `z` are the free primal variables, `P` is symmetric positive semidefinite
/// (the quadratic part), `q` is the linear part, and the slack `s` must land in
/// the cone `K`. With `K` the non-negative orthant `R_+^m`, the constraint is
/// exactly the elementwise inequality `A z <= b` (the slack is `b - A z`), so
/// any `A z <= b` system maps over once the objective is in `(1/2) z'Pz + q'z`
/// shape. This formulation has no linear term in the recipe objective, so
/// `q = 0` and everything lives in `P`.
///
/// # Converting the recipe problem
///
/// A squared `max(0, ·)` is not linear, but it becomes a squared *variable* on
/// the left of a linear row. Add one auxiliary variable per soft-loss side:
///
/// ```text
/// shortage  s_i >= 1 - y_i/L_i   contributes  s_i²
/// excess    t_i >= y_i/U_i - 1   contributes  t_i²
/// ```
///
/// and write them as rows of `A z <= b` in `z = [x, s…, t…]`:
///
/// ```text
/// -h_i/L_i · x - s_i <= -1     ⇒  s_i = max(0, 1 - y_i/L_i) at the optimum
///  h_i/U_i · x - t_i <=  1     ⇒  t_i = max(0, y_i/U_i - 1) at the optimum
/// ```
///
/// At the optimum each auxiliary sits exactly on `max(0, ·)`. The rows permit
/// the auxiliary to reach that value, and `s_i²`, `t_i²` increase with their
/// arguments on the non-negative side, so minimising the square pins the
/// variable to the smallest value the row allows — which is `max(0, ·)`. That
/// is why no extra `s_i >= 0` / `t_i >= 0` rows are needed: the squared cost
/// does that work, and the `Python` reference uses the same trick (`cp.pos`).
/// The food variables and the auxiliaries are therefore all free in Clarabel;
/// the non-negativity lives in the cone applied to the slack, and the row
/// `x_j >= lower_j >= 0` re-establishes `x_j >= 0`.
///
/// In this `z`, with `P` diagonal:
///
/// ```text
/// P = diag( 0.1/upper_j² for the foods, 2 for every shortage, 2 for every excess )
/// q = 0
/// ```
///
/// `P_ss = 2` makes `(1/2) z'Pz` contribute `s²`, i.e. the loss exactly. The
/// `0.05·(x_j/upper_j)²` preference becomes `P_jj = 0.1/upper_j²`, because
/// `(1/2)·(0.1/upper_j²)·x² = 0.05·(x/upper_j)²`. (The Python CVXPY reference
/// assembles `P[j,j] += 0.1/ub²` for the same reason; CVXPY then generates the
/// distance from the objective it was given, so the two formulations agree.)
///
/// The remaining rows are linear and need no auxiliaries:
///
/// ```text
/// -x_j            <= -lower_j      food lower bound (also x_j >= 0 when lower_j = 0)
///  x_j            <=  upper_j      food upper bound
/// -h_i/Ln · x     <= -L_i/Ln       HARD minimum
///  h_i/Ln · x     <=  U_i/Ln       HARD explicit maximum
/// -(N_r - L_r·D_r)·x <= 0          ratio minimum
///  (N_r - U_r·D_r)·x <= 0          ratio maximum
/// ```
///
/// The HARD rows are divided by `Ln = max(L_i, U_i, max_j |h_ij|)`, which
/// rescales the row without moving the feasible set but keeps energy (millions
/// of joules) and trace nutrients (micrograms) in a comparable numeric range
/// for the interior-point iterations. The ratio rows come from clearing the
/// denominator: `L_r <= (N_r·x)/(D_r·x) <= U_r` with `D_r·x > 0` is the pair
/// `L_r·D_r·x <= N_r·x` and `N_r·x <= U_r·D_r·x`, two linear rows (the
/// denominator keeps its own positive absolute requirement, so it cannot be
/// zero). Both food bounds and requirements are batch quantities, so the batch
/// day count never enters the solver — the frontends scale the requirements
/// before calling, not here.
///
/// `csc_from_rows` assembles the collected rows into the compressed sparse
/// column matrix Clarabel wants, `b` is the right-hand side, and the single
/// `NonnegativeConeT(rows)` cone is the non-negative orthant. The solution's
/// `x` is sliced back to the first `n` entries to drop the auxiliaries, giving
/// the batch grams only.
///
/// # Returns
///
/// `grams` is `Some` only for an optimal solve, so an infeasible or failed
/// solve cannot leak a stale recipe. `SolveStatus::Infeasible` folds in
/// Clarabel's `AlmostPrimalInfeasible`; every other status is passed through by
/// name in `SolveStatus::Other`.
///
/// [Clarabel]: https://github.com/oxfordcontrol/Clarabel.rs
#[throws(Error)]
pub fn solve(problem: &Problem) -> Solution {
    let foods = &problem.foods;
    let n = foods.len();
    if n == 0 {
        throw!(anyhow!("Add at least one food before solving"));
    }
    for food in foods {
        if !food.lower.is_finite()
            || !food.upper.is_finite()
            || food.lower < 0.0
            || food.upper < food.lower
        {
            throw!(anyhow!("Invalid food bounds for {}", food.name.name()));
        }
    }

    // Use nutrient_value, not a raw lookup: combined targets (methionine +
    // cystine, phenylalanine + tyrosine, EPA + DHA) are sums of their parts,
    // and vitamin B5 falls back to the legacy pantothenic-acid row.
    let value_of = |values: &Food, nutrient: Nutrient| -> f64 {
        nutrient_value(values, nutrient).0.unwrap_or(0.0)
    };

    let mut rows: Vec<Row> = Vec::new();
    // Diagonal of P, one entry per variable (foods then auxiliaries).
    let mut p_diagonal: Vec<f64> = vec![0.0; n];
    // Auxiliary variables appended after the food variables.
    let mut aux_index = n;

    // Ingredient bounds and the stock-fraction preference.
    for (index, food) in foods.iter().enumerate() {
        // The Python reference declares x non-negative and adds x >= lower, so
        // an optional ingredient (lower == 0) still needs the x >= 0 row:
        // Clarabel variables are free unless constrained.
        let mut lower_row = Row::new();
        lower_row.push(index, -1.0);
        lower_row.bound = -food.lower;
        rows.push(lower_row);

        let mut upper_row = Row::new();
        upper_row.push(index, 1.0);
        upper_row.bound = food.upper;
        rows.push(upper_row);

        if food.minimize_usage && food.upper > 0.0 {
            // 0.05 * (x / ub)^2 = 0.5 * P * x^2  =>  P = 0.1 / ub^2.
            p_diagonal[index] += 0.1 / (food.upper * food.upper);
        }
    }

    for (nutrient, requirement) in &problem.needs {
        if requirement.required != NeedRequired::Required {
            continue;
        }
        if !requirement.minimum.is_finite() || requirement.minimum < 0.0 {
            throw!(anyhow!("Invalid minimum for {}", nutrient.name()));
        }
        if let Some(maximum) = requirement.maximum {
            if !maximum.is_finite() || maximum < requirement.minimum {
                throw!(anyhow!("Invalid maximum for {}", nutrient.name()));
            }
        }

        let column: Vec<f64> = foods
            .iter()
            .map(|food| value_of(&food.values, *nutrient))
            .collect();
        let effective_upper = problem.effective_upper_bound(requirement);

        if requirement.softness == NeedSoftness::Hard {
            // Scale each row without changing the feasible set: energy is in
            // millions of joules while trace nutrients may be micrograms.
            let largest = column.iter().fold(0.0_f64, |acc, v| acc.max(v.abs()));
            let normalizer = requirement
                .minimum
                .max(requirement.maximum.unwrap_or(0.0))
                .max(largest);
            let normalizer = if normalizer == 0.0 { 1.0 } else { normalizer };

            let mut row = Row::new();
            for (index, value) in column.iter().enumerate() {
                row.push(index, -value / normalizer);
            }
            row.bound = -requirement.minimum / normalizer;
            rows.push(row);

            if let Some(maximum) = requirement.maximum {
                let mut row = Row::new();
                for (index, value) in column.iter().enumerate() {
                    row.push(index, value / normalizer);
                }
                row.bound = maximum / normalizer;
                rows.push(row);
            }
        } else if requirement.minimum > 0.0 {
            // s >= 1 - H@x/L  =>  -H/L @ x - s <= -1.
            let mut row = Row::new();
            for (index, value) in column.iter().enumerate() {
                row.push(index, -value / requirement.minimum);
            }
            row.push(aux_index, -1.0);
            row.bound = -1.0;
            rows.push(row);
            p_diagonal.push(2.0);
            aux_index += 1;
        }

        // Excess is a soft preference. `effective_upper` is `None` for a hard
        // requirement without an explicit maximum, so no excess term is built.
        if let Some(upper) = effective_upper {
            if upper <= 0.0 {
                throw!(anyhow!(
                    "Soft maximum for {} must be positive",
                    nutrient.name()
                ));
            }
            // t >= H@x/U - 1  =>  H/U @ x - t <= 1.
            let mut row = Row::new();
            for (index, value) in column.iter().enumerate() {
                row.push(index, value / upper);
            }
            row.push(aux_index, -1.0);
            row.bound = 1.0;
            rows.push(row);
            p_diagonal.push(2.0);
            aux_index += 1;
        }
    }

    for ratio in &problem.ratios {
        let numerator: Vec<f64> = foods
            .iter()
            .map(|food| value_of(&food.values, ratio.numerator))
            .collect();
        let denominator: Vec<f64> = foods
            .iter()
            .map(|food| value_of(&food.values, ratio.denominator))
            .collect();
        // minimum <= (N@x)/(D@x) <= maximum becomes two linear inequalities.
        // The denominator keeps its own positive absolute requirement, so it
        // cannot be zero. Ratio bounds are dimensionless: never scaled by days.
        let mut lower_row = Row::new();
        let mut upper_row = Row::new();
        for index in 0..n {
            lower_row.push(
                index,
                -(numerator[index] - ratio.minimum * denominator[index]),
            );
            upper_row.push(index, numerator[index] - ratio.maximum * denominator[index]);
        }
        rows.push(lower_row);
        rows.push(upper_row);
    }

    let total = aux_index;
    let a = csc_from_rows(rows.len(), total, &rows);
    let b: Vec<f64> = rows.iter().map(|row| row.bound).collect();

    // P is diagonal: 2 per squared auxiliary, plus the ingredient preferences.
    let mut p_rows: Vec<Row> = (0..total).map(|_| Row::new()).collect();
    for (index, weight) in p_diagonal.iter().enumerate() {
        p_rows[index].push(index, *weight);
    }
    let p = csc_from_rows(total, total, &p_rows);

    let q = vec![0.0; total];
    let cones = [NonnegativeConeT(rows.len())];
    // Clarabel prints its banner and iteration table to stdout by default,
    // which would corrupt both the CLI report and the MCP stdio protocol.
    let settings = DefaultSettings {
        verbose: false,
        ..DefaultSettings::default()
    };
    let mut solver = DefaultSolver::new(&p, &q, &a, &b, &cones, settings)
        .map_err(|error| anyhow!("failed to build the solver: {error}"))?;
    solver.solve();

    let status = match solver.solution.status {
        SolverStatus::Solved => SolveStatus::Optimal,
        SolverStatus::PrimalInfeasible | SolverStatus::AlmostPrimalInfeasible => {
            SolveStatus::Infeasible
        }
        other => SolveStatus::Other(format!("{other:?}")),
    };
    let optimal = status == SolveStatus::Optimal;
    Solution {
        grams: if optimal {
            Some(solver.solution.x[..n].to_vec())
        } else {
            None
        },
        objective: if optimal {
            Some(solver.solution.obj_val)
        } else {
            None
        },
        status,
    }
}

/// One food's contribution to a nutrient, in display units.
#[derive(Debug, Clone)]
pub struct Component {
    pub food: FoodName,
    pub amount: f64,
}

/// One nutrient line of the daily report, shared by both frontends.
///
/// `value`, `minimum` and `maximum` are already divided by the batch day count
/// and expressed in `unit`; `scale` converts them back to the solver's base
/// unit. `maximum` is `None` when the requirement is open-ended; `implicit`
/// marks a soft preference above an open-ended minimum (a preference, not a
/// toxicity limit).
#[derive(Debug, Clone)]
pub struct NutrientReport {
    pub nutrient: Nutrient,
    pub required: NeedRequired,
    pub status: ReportStatus,
    pub unit: &'static str,
    pub scale: f64,
    pub value: f64,
    pub minimum: f64,
    pub maximum: Option<f64>,
    pub implicit_upper: bool,
    pub implicit_multiplier: Option<f64>,
    pub missing: Vec<FoodName>,
    pub components: Vec<Component>,
}

/// Where the supply sits relative to its requirement interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportStatus {
    BelowMinimum,
    WithinRange,
    AboveMaximum,
}

impl ReportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ReportStatus::BelowMinimum => "below minimum",
            ReportStatus::WithinRange => "within range",
            ReportStatus::AboveMaximum => "above maximum",
        }
    }
}

/// Build the daily report for a solved problem.
///
/// `needs` is the scaled requirement table used for the solve; `grams` are the
/// solved batch weights. Values are per day, so the whole batch is divided by
/// `days`.
pub fn nutrition_report(
    problem: &Problem,
    needs: &[(Nutrient, Requirement)],
    grams: &[f64],
    days: u32,
    detail: bool,
) -> Vec<NutrientReport> {
    let day = days as f64;
    let mut reports = Vec::new();

    for nutrient in Nutrient::all() {
        let Some((_, requirement)) = needs.iter().find(|(n, _)| n == nutrient) else {
            continue;
        };
        let effective_upper = problem.effective_upper_bound(requirement);

        let mut total = 0.0;
        let mut components = Vec::new();
        let mut missing = Vec::new();
        for (index, food) in problem.foods.iter().enumerate() {
            let (value, known) = nutrient_value(&food.values, *nutrient);
            let contribution = value.unwrap_or(0.0) * grams[index];
            if !known && grams[index] > 1e-6 {
                // Report incompleteness only for foods actually in the recipe.
                missing.push(food.name);
            }
            if contribution != 0.0 && detail {
                components.push(Component {
                    food: food.name,
                    amount: contribution / day,
                });
            }
            total += contribution;
        }

        let minimum = requirement.minimum / day;
        let maximum = effective_upper.map(|value| value / day);
        let value = total / day;

        let (unit, scale) = if *nutrient == Nutrient::Energy {
            ("kJ", 1000.0)
        } else if minimum >= G {
            ("g", G)
        } else if minimum >= MG {
            ("mg", MG)
        } else {
            ("μg", MCG)
        };

        let value = value / scale;
        let minimum = minimum / scale;
        let maximum = maximum.map(|value| value / scale);

        let status = if value < minimum {
            ReportStatus::BelowMinimum
        } else if maximum.is_some_and(|bound| value > bound) {
            ReportStatus::AboveMaximum
        } else {
            ReportStatus::WithinRange
        };

        // Scale the per-food daily contributions into the display unit too.
        for component in &mut components {
            component.amount /= scale;
        }

        reports.push(NutrientReport {
            nutrient: *nutrient,
            required: requirement.required,
            status,
            unit,
            scale,
            value,
            minimum,
            maximum,
            implicit_upper: requirement.maximum.is_none() && effective_upper.is_some(),
            implicit_multiplier: problem.implicit_soft_upper_multiplier,
            missing,
            components,
        });
    }
    reports
}
