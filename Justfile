# export HTTPS_PROXY:="http://127.0.0.1:6152"
export USDA_API_KEY:="Sb1Cy9W8buemWhJcD012yVVFnwNffiyudPOwI6rw"

# Refresh the food caches the Rust table embeds. With no FOOD args, fetches
# every food that has a remote source; name foods to refresh just those.
# e.g. `just refresh-foods PORK BEEF`, `just refresh-foods --list`.
refresh-foods +ARGS="":
  cargo run --quiet --release --bin food -- fetch {{ARGS}}

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
  python3 scripts/make_plugin.py {{ARGS}}

# Build the static musl Linux plugin and pack an installable tarball.
# The tarball carries install.sh/uninstall.sh, so the target host only needs
# `tar -xzf` + `./food-solver/install.sh` (plus python3 or jq, and codex).
# Examples:
#   just release-plugin
#   just release-plugin --skip-build
#   just release-plugin --dest /tmp/out --archive /tmp/food-solver.tar.gz
release-plugin +ARGS="":
  python3 scripts/make_plugin.py --musl --skip-marketplace --dest dist --archive dist/food-solver-musl.tar.gz {{ARGS}}

# Start the MCP server over stdio, the way Codex launches it.
mcp:
  cargo run --quiet --bin food-mcp

example:
  just run solve --detail true -d 14 -i PORK:596 -i CHICKEN_HEART:244 -i CHICKEN_BREAST:860 -i CARROT:566 -i JIANGDOU:700 -i BAICAI:562 -i BROCCOLI:500 -i PUMPKIN:500 -i EGG:700:minimize -i EGG_SHELL_POWDER:70:minimize -i CANOLA_OIL:70:minimize -i SALT:35:minimize -i RICE:7000:minimize -i BASA_FISH:500:minimize -i BONE_MEAL:100:minimize -i BEEF_LIVER:600

check-nut:
  just run report --detail false -d 7 -i PORK:218 -i CHICKEN_HEART:0 -i CHICKEN_BREAST:0 -i CARROT:91 -i JIANGDOU:700 -i BAICAI:224 -i BROCCOLI:206 -i PUMPKIN:195 -i EGG:350 -i EGG_SHELL_POWDER:9.3 -i CANOLA_OIL:7.49 -i SALT:0.78 -i RICE:427
