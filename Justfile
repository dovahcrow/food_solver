# export HTTPS_PROXY:="http://127.0.0.1:6152"
export USDA_API_KEY:="Sb1Cy9W8buemWhJcD012yVVFnwNffiyudPOwI6rw"

# Python reference frontends (kept for cross-checking the Rust solver).
opt +ARGS="":
  uv run python -m src opt {{ARGS}}

# Refresh the food caches the Rust table embeds. With no FOOD args, fetches
# every food that has a remote source; name foods to refresh just those.
# e.g. `just refresh-foods PORK BEEF`, `just refresh-foods --list`.
refresh-foods +ARGS="":
  cargo run --quiet --release --bin food -- fetch {{ARGS}}

# Same, but through the Python reference frontend.
refresh-foods-py +ARGS="":
  uv run python -m src refresh-foods {{ARGS}}

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

# Run the Rust CLI, e.g. `just run solve -d 10 -i PORK:500 -i RICE:700:minimize`.
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

example:
  just run solve --detail true -d 14 -i PORK:596:optional -i CHICKEN_HEART:244:optional -i CHICKEN_BREAST:860:optional -i CARROT:566:optional -i JIANGDOU:700:optional -i BAICAI:562:optional -i BROCCOLI:500:optional -i PUMPKIN:500:optional -i EGG:700:minimize -i EGG_SHELL_POWDER:70:minimize -i CANOLA_OIL:70:minimize -i SALT:35:minimize -i RICE:7000:minimize -i BASA_FISH:500:minimize -i BONE_MEAL:100:minimize -i BEEF_LIVER:600:optional

check-nut:
  just run report --detail false -d 7 -i PORK:218:fixed -i CHICKEN_HEART:0:fixed -i CHICKEN_BREAST:0:fixed -i CARROT:91:fixed -i JIANGDOU:700:fixed -i BAICAI:224:fixed -i BROCCOLI:206:fixed -i PUMPKIN:195:fixed -i EGG:350:fixed -i EGG_SHELL_POWDER:9.3:fixed -i CANOLA_OIL:7.49:fixed -i SALT:0.78:fixed -i RICE:427:fixed
