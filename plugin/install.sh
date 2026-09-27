#!/bin/sh
# Install the Food Solver Codex plugin from an unpacked release tarball.
#
# Run this from inside the unpacked `food-solver/` directory (or pass it to a
# shell). It copies the plugin into ~/plugins/food-solver, merges it into the
# personal marketplace, and registers it with `codex plugin add`.
#
# Usage: ./install.sh [options]
#   --plugins-dir DIR      where to copy the plugin (default: ~/plugins)
#   --marketplace FILE     marketplace JSON to update
#                          (default: ~/.agents/plugins/marketplace.json)
#   --marketplace-name N   marketplace name, only used to seed a new file
#                          (default: personal)
#   --no-codex             do not run `codex plugin add`
#   -h, --help             show this help
#
# Environment fallbacks: FOOD_SOLVER_PLUGINS_DIR, FOOD_SOLVER_MARKETPLACE,
# FOOD_SOLVER_MARKETPLACE_NAME.
set -eu

PLUGIN_NAME="food-solver"
PLUGINS_DIR="${FOOD_SOLVER_PLUGINS_DIR:-$HOME/plugins}"
MARKETPLACE="${FOOD_SOLVER_MARKETPLACE:-$HOME/.agents/plugins/marketplace.json}"
MARKETPLACE_NAME="${FOOD_SOLVER_MARKETPLACE_NAME:-personal}"
RUN_CODEX=1

usage() {
    sed -n '2,/^[^#]/p' "$0" | sed -e '$d' -e 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --plugins-dir) PLUGINS_DIR="$2"; shift 2 ;;
        --plugins-dir=*) PLUGINS_DIR="${1#*=}"; shift ;;
        --marketplace) MARKETPLACE="$2"; shift 2 ;;
        --marketplace=*) MARKETPLACE="${1#*=}"; shift ;;
        --marketplace-name) MARKETPLACE_NAME="$2"; shift 2 ;;
        --marketplace-name=*) MARKETPLACE_NAME="${1#*=}"; shift ;;
        --no-codex) RUN_CODEX=0; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "install.sh: unknown option $1" >&2; usage >&2; exit 2 ;;
    esac
done

SOURCE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DEST="$PLUGINS_DIR/$PLUGIN_NAME"

if [ "$SOURCE" != "$DEST" ]; then
    echo "Copying $PLUGIN_NAME to $DEST"
    mkdir -p "$PLUGINS_DIR"
    rm -rf "$DEST"
    cp -R "$SOURCE" "$DEST"
else
    echo "$PLUGIN_NAME is already installed at $DEST"
fi

mkdir -p "$(dirname -- "$MARKETPLACE")"

if command -v python3 >/dev/null 2>&1; then
    python3 - "$MARKETPLACE" "$MARKETPLACE_NAME" "$PLUGIN_NAME" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
marketplace_name, plugin_name = sys.argv[2], sys.argv[3]

if path.is_file():
    data = json.loads(path.read_text(encoding="utf-8"))
else:
    data = {"name": marketplace_name, "interface": {"displayName": "Personal"}}

data.setdefault("name", marketplace_name)
data.setdefault("interface", {"displayName": "Personal"})
entry = {
    "name": plugin_name,
    "source": {"source": "local", "path": f"./plugins/{plugin_name}"},
    "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
    "category": "Productivity",
}
plugins = [item for item in data.get("plugins", []) if item.get("name") != plugin_name]
plugins.append(entry)
data["plugins"] = plugins
path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
elif command -v jq >/dev/null 2>&1; then
    entry=$(jq -n --arg name "$PLUGIN_NAME" '{
        name: $name,
        source: {source: "local", path: ("./plugins/" + $name)},
        policy: {installation: "AVAILABLE", authentication: "ON_INSTALL"},
        category: "Productivity"
    }')
    if [ -f "$MARKETPLACE" ]; then
        tmp=$(mktemp)
        jq --argjson entry "$entry" '
            .plugins = (((.plugins // []) | map(select(.name != $entry.name))) + [$entry])
        ' "$MARKETPLACE" >"$tmp"
        mv "$tmp" "$MARKETPLACE"
    else
        jq -n --arg name "$MARKETPLACE_NAME" --argjson entry "$entry" '{
            name: $name,
            interface: {displayName: "Personal"},
            plugins: [$entry]
        }' >"$MARKETPLACE"
    fi
else
    echo "install.sh: need python3 or jq to update $MARKETPLACE" >&2
    exit 1
fi
echo "Updated marketplace: $MARKETPLACE"

if [ "$RUN_CODEX" -eq 1 ] && command -v codex >/dev/null 2>&1; then
    codex plugin add "$PLUGIN_NAME@$MARKETPLACE_NAME"
    echo "Installed. Start a new Codex thread to pick up the tools."
elif [ "$RUN_CODEX" -eq 1 ]; then
    echo "codex not found on PATH; install it and run:"
    echo "  codex plugin add $PLUGIN_NAME@$MARKETPLACE_NAME"
else
    echo "Skipped codex registration. Once codex is available run:"
    echo "  codex plugin add $PLUGIN_NAME@$MARKETPLACE_NAME"
fi
