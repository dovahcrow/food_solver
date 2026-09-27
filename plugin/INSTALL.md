# Installing Food Solver

The plugin is a self-contained Rust binary: the food database and the Clarabel
solver are compiled in, so it needs no Python and no internet access.

## What ships

- `bin/food-mcp` — the MCP server Codex launches.
- `bin/food` — the same solver as a command-line tool.
- `.mcp.json`, `.codex-plugin/plugin.json`, `skills/food-solver/`.
- `install.sh` / `uninstall.sh` — copy the plugin and edit the marketplace
  (only used when installing from a release tarball).

Binaries are **platform-specific**: build them on the machine that will run
them (or on one with the same OS and CPU architecture).

## Release tarball (static musl Linux)

`just release-plugin` builds the plugin against the musl target for a fully
static Linux binary and packs `dist/food-solver-musl.tar.gz`:

```sh
just release-plugin
just release-plugin --skip-build              # reuse target/<triple>/release
just release-plugin --target aarch64-unknown-linux-musl
```

The tarball unpacks to a single `food-solver/` directory that carries
`install.sh` and `uninstall.sh`.

## Install from this checkout

```sh
just make-plugin --install
```

That builds the release binaries, writes `~/plugins/food-solver`, registers the
personal marketplace, and runs `codex plugin add food-solver@personal`.
Start a new Codex thread afterwards to pick up the tools.

## Install on another machine

Package a tarball on the build host (`just release-plugin` for a static musl
build, or the `make-plugin` command below), copy it over, then run the bundled
installer on the target host.

On the build host:

```sh
just release-plugin
# or, without the musl target:
just make-plugin --skip-marketplace --dest dist --archive dist/food-solver.tar.gz
```

On the target host (same OS and architecture):

```sh
tar -xzf food-solver-musl.tar.gz
./food-solver/install.sh
```

`install.sh` copies the plugin to `~/plugins/food-solver`, merges it into
`~/.agents/plugins/marketplace.json`, and runs
`codex plugin add food-solver@personal`. It needs `python3` or `jq` to edit the
marketplace, and `codex` on `PATH` to register the plugin. Use `--no-codex` to
skip registration, `--plugins-dir` / `--marketplace` to retarget the copy, and
`./food-solver/uninstall.sh` to reverse it.

If you would rather wire things up by hand, the same steps are:

```sh
mkdir -p ~/plugins
tar -xzf food-solver.tar.gz -C ~/plugins

mkdir -p ~/.agents/plugins
cat > ~/.agents/plugins/marketplace.json <<'JSON'
{
  "name": "personal",
  "interface": { "displayName": "Personal" },
  "plugins": [
    {
      "name": "food-solver",
      "source": { "source": "local", "path": "./plugins/food-solver" },
      "policy": { "installation": "AVAILABLE", "authentication": "ON_INSTALL" },
      "category": "Productivity"
    }
  ]
}
JSON

codex plugin add food-solver@personal
```

`~/.agents/plugins/marketplace.json` is discovered implicitly, so no
`codex plugin marketplace add` is needed for that path. If you keep the
marketplace elsewhere, register it first with
`codex plugin marketplace add <path>` and install from that name instead.

## Building on another platform

The workspace has no cross-compilation setup; build natively instead.

```sh
cargo build --release --bin food --bin food-mcp
```

For a fully static Linux binary, build on Linux for the musl target (uses the
system `musl-gcc`): `just release-plugin` does this for the host architecture,
or pass `--target x86_64-unknown-linux-musl` / `aarch64-unknown-linux-musl`
to `scripts/make_plugin.py` to pick one.

## Verifying an install

```sh
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
  | ./bin/food-mcp | head -c 200
```

A JSON `initialize` result means the binary runs on this host. Then:

```sh
codex mcp list | grep food_solver
```

Finally, open a new Codex thread and check that the `food_solver` tools appear.
