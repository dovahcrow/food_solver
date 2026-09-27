//! MCP server frontend for the food solver.
//!
//! Exposes the shared batch-planning flow as MCP tools: ingredient amounts
//! plus a day count go in, a solved recipe and its per-day nutrient report come
//! back as structured data. The CLI (`food`) is the other frontend over the
//! same `food-core` library.
//!
//! Run with the stdio transport, which is what the Codex plugin config uses.

mod tools;

use rmcp::{transport::stdio, ServiceExt};

use tools::FoodSolver;

#[culpa::throws(anyhow::Error)]
#[tokio::main]
async fn main() {
    // The server speaks JSON-RPC over stdout, so nothing else may write there.
    let service = FoodSolver::new().serve(stdio()).await?;
    service.waiting().await?;
}
