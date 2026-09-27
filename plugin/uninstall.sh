#!/bin/sh
# Remove a Food Solver plugin installed with install.sh.
#
# Deletes the plugin directory and drops its entry from the personal
# marketplace. It does not touch anything else in the marketplace file.
#
# Usage: ./uninstall.sh [options]
#   --plugins-dir DIR      plugin location (default: ~/plugins)
#   --marketplace FILE     marketplace JSON to update
#                          (default: ~/.agents/plugins/marketplace.json)
#   -h, --help             show this help
#
# Environment fallbacks: FOOD_SOLVER_PLUGINS_DIR, FOOD_SOLVER_MARKETPLACE.
set -eu

PLUGIN_NAME="food-solver"
PLUGINS_DIR="${FOOD_SOLVER_PLUGINS_DIR:-$HOME/plugins}"
MARKETPLACE="${FOOD_SOLVER_MARKETPLACE:-$HOME/.agents/plugins/marketplace.json}"

usage() {
    sed -n '2,/^[^#]/p' "$0" | sed -e '$d' -e 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --plugins-dir) PLUGINS_DIR="$2"; shift 2 ;;
        --plugins-dir=*) PLUGINS_DIR="${1#*=}"; shift ;;
        --marketplace) MARKETPLACE="$2"; shift 2 ;;
        --marketplace=*) MARKETPLACE="${1#*=}"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "uninstall.sh: unknown option $1" >&2; usage >&2; exit 2 ;;
    esac
done

DEST="$PLUGINS_DIR/$PLUGIN_NAME"
if [ -d "$DEST" ]; then
    echo "Removing $DEST"
    rm -rf "$DEST"
else
    echo "No plugin directory at $DEST"
fi

if [ -f "$MARKETPLACE" ]; then
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$MARKETPLACE" "$PLUGIN_NAME" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
plugin_name = sys.argv[2]
data = json.loads(path.read_text(encoding="utf-8"))
data["plugins"] = [
    item for item in data.get("plugins", []) if item.get("name") != plugin_name
]
path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
PY
    elif command -v jq >/dev/null 2>&1; then
        tmp=$(mktemp)
        jq --arg name "$PLUGIN_NAME" \
            '.plugins = ((.plugins // []) | map(select(.name != $name)))' \
            "$MARKETPLACE" >"$tmp"
        mv "$tmp" "$MARKETPLACE"
    else
        echo "uninstall.sh: need python3 or jq to update $MARKETPLACE" >&2
        exit 1
    fi
    echo "Removed $PLUGIN_NAME from $MARKETPLACE"
fi
