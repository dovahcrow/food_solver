---
name: food-solver
description: Plan and check homemade dog food batches with the food_solver MCP tools. Use when the user asks to work out a dog food recipe, decide how much of each ingredient to use, balance ingredients against FEDIAF adult dog nutrient requirements, review a batch's per-day nutrients, or explain why a batch is short on a nutrient.
---

# Food Solver

The `food_solver` MCP server turns ingredient amounts plus a day count into a
solved recipe and a per-day nutrient report for adult dogs.

## Workflow

1. Map the user's ingredients to exact names with `list_foods` before solving.
2. Call `solve_recipe` with the ingredients and the day count.
3. Check `optimal`. When it is false, follow `hint` and retry: a pinned
   ingredient weight is usually the blocker, so mark it `optional` to let the
   solver pick its weight within 0..grams.
4. Report the recipe, and call out the nutrients that come back `below minimum`
   or `above maximum`.

## Amount conventions

- `grams` is the batch total, never the per-day amount.
- `days` scales the nutrient requirements and the report basis. Fixed ingredient
  weights are not scaled, so multiply a per-day weight by the day count first.
- `optional: true` lets the solver choose anything from 0 up to `grams`.
  `minimize_usage: true` prefers less of that ingredient among equally good
  recipes.
- Use `get_needs` to show the raw requirement table for a profile.

## Ground rules

- Only adult maintenance is modelled, age 1 year and up. Do not apply it to
  puppies, pregnancy or lactation.
- Requirements come from FEDIAF 2025 Table III-3b. Describe the recipe as a
  feasible solution, not as nutritionally complete: the food database has gaps
  and the report flags those gaps in `incomplete_data_from`.
- `maximum_is_implicit` marks the solver's soft preference above an open-ended
  minimum. It is not a toxicity limit.
- The food database is embedded in the server at build time from `foods/*.json`;
  refresh those caches with the Python frontend and rebuild.
- This is recipe arithmetic, not veterinary advice.
