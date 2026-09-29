---
name: food-solver
description: Plan and check homemade dog food batches with the food_solver MCP tools. Use when the user asks to work out a dog food recipe, decide how much of each ingredient to use, balance ingredients against FEDIAF adult dog nutrient requirements, review a batch's per-day nutrients, or explain why a batch is short on a nutrient.
---

# Food Solver

The `food_solver` MCP server turns ingredient amounts plus a dog profile into a
solved recipe and a per-day nutrient report for adult dogs.

## Workflow

1. Map the user's ingredients to exact names with `list_foods` before solving.
2. Call `solve_recipe` with the ingredients, the day count, the dog's `weight`
   (kg) and `age` (years). These are required: there are no defaults, so ask
   the user when the profile is unknown.
3. Check `optimal`. When it is false, follow `hint` and retry: a pinned
   ingredient weight is usually the blocker, so mark it `optional` to let the
   solver pick its weight within 0..grams.
4. Report the recipe, and call out the nutrients that come back `below minimum`
   or `above maximum`.

## Amount conventions

- `grams` is the batch total, never the per-day amount.
- `days` scales the nutrient requirements and the report basis. Fixed ingredient
  weights are not scaled, so multiply a per-day weight by the day count first.
- `weight`, `days` and `age` are required on both `solve_recipe` and
  `get_needs`; `optional`, `minimize_usage`, `active`, `daily_kcal`,
  `implicit_soft_upper_multiplier` and `detail` default when omitted.
- `optional: true` lets the solver choose anything from 0 up to `grams`.
  `minimize_usage: true` prefers less of that ingredient among equally good
  recipes.
- When a nutrient line reports `maximum_is_implicit: true`, treat its maximum
  as `inf`: the value shown is only the solver's soft preference, not a real
  ceiling, so exceeding it is not a problem.
- When the user says an ingredient should be given "in a suitable amount"
  (适量), set `optional: true` and let the solver pick the weight.
- When the user says an ingredient should be given "without limit" (不限量),
  set `grams` to 1000 * `days` and `minimize_usage: true`, with
  `optional: true`; the solver then has room to use as much as the recipe
  needs while still preferring less.
- Use `get_needs` to show the raw requirement table for a profile.
- Use `build_info` when a result needs to be tied to a specific build: it
  returns the build date and git revision the server was compiled from.

## Ground rules

- Only adult maintenance is modelled, age 1 year and up. Do not apply it to
  puppies, pregnancy or lactation.
- Requirements come from FEDIAF 2025 Table III-3b. Describe the recipe as a
  feasible solution, not as nutritionally complete: the food database has gaps
  and the report flags those gaps in `incomplete_data_from`.
- `maximum_is_implicit` marks the solver's soft preference above an open-ended
  minimum. It is not a toxicity limit.
- The food database is embedded in the server at build time from
  `foods/{FOOD}_{Source}.json`; refresh those caches in the repository
  (`just refresh-foods`) and rebuild.
- This is recipe arithmetic, not veterinary advice.
