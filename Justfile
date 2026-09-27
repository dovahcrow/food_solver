# Python reference frontends (kept for data refresh and cross-checking).
opt +ARGS="":
  uv run python -m src opt {{ARGS}}

# Rust workspace: lib + CLI + MCP server.
build:
  cargo build --release

check:
  cargo check --workspace --all-targets

fmt:
  cargo fmt --all

clippy:
  cargo clippy --workspace --all-targets -- -D warnings

test:
  cargo test --workspace

# Run the Rust CLI, e.g. `just run -d 10`.
run +ARGS="":
  cargo run --quiet --release --bin food -- {{ARGS}}

# Package this checkout as the local Codex plugin `food-solver`.
# Examples:
#   just make-plugin                  # build + update the personal marketplace
#   just make-plugin --install        # also run `codex plugin add`
#   just make-plugin --skip-build --dest /tmp/plugins
make-plugin +ARGS="":
  uv run python scripts/make_plugin.py {{ARGS}}

# Build the static musl Linux plugin and pack an installable tarball.
# The tarball carries install.sh/uninstall.sh, so the target host only needs
# `tar -xzf` + `./food-solver/install.sh` (plus python3 or jq, and codex).
# Examples:
#   just release-plugin
#   just release-plugin --skip-build
#   just release-plugin --dest /tmp/out --archive /tmp/food-solver.tar.gz
release-plugin +ARGS="":
  uv run python scripts/make_plugin.py --musl --skip-marketplace --dest dist --archive dist/food-solver-musl.tar.gz {{ARGS}}

# Start the MCP server over stdio, the way Codex launches it.
mcp:
  cargo run --quiet --bin food-mcp
