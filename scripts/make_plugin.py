#!/usr/bin/env python3
"""Package the food-solver Codex plugin from the Rust workspace.

The plugin is self-contained: it ships the release `food-mcp` binary (with the
food database and the Clarabel solver compiled in), the `food` CLI next to it,
the MCP configuration, and the skill. Nothing points back at this checkout, so
the plugin keeps working if the repository moves or disappears.

Build the binaries on the machine that will run them: the plugin ships native
executables, so a macOS build will not run on Linux or vice versa. Rust makes
this cheap, so package once per platform instead of cross-compiling.

Usage:
    python scripts/make_plugin.py                    # build + update marketplace
    python scripts/make_plugin.py --install          # also run `codex plugin add`
    python scripts/make_plugin.py --skip-build       # reuse target/release
    python scripts/make_plugin.py --archive out.tar  # also write a tarball
    python scripts/make_plugin.py --musl --archive dist/food-solver-musl.tar.gz
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
TEMPLATE_ROOT = REPO_ROOT / "plugin"
PLUGIN_NAME = "food-solver"
VERSION = "0.1.0"
BINARIES = ("food-mcp", "food")


def musl_triple() -> str:
    """The musl target matching the machine we are building on."""
    machine = platform.machine().lower()
    if machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-musl"
    if machine in {"aarch64", "arm64"}:
        return "aarch64-unknown-linux-musl"
    raise SystemExit(
        f"no musl target known for {machine!r}; pass --target explicitly "
        "(for example --target x86_64-unknown-linux-musl)"
    )


def target_dir(target: str | None) -> Path:
    """Release directory cargo used.

    A bare `cargo build` writes to `target/release`; passing `--target`
    (directly or through `CARGO_BUILD_TARGET`) nests it under the triple.
    """
    triple = target or os.environ.get("CARGO_BUILD_TARGET")
    if triple:
        return REPO_ROOT / "target" / triple / "release"
    return REPO_ROOT / "target" / "release"


def platform_label(target: str | None = None) -> str:
    """Human-readable platform of the packaged binaries, e.g. linux-x86_64."""
    if target:
        if "musl" in target:
            return f"linux-{platform.machine()}-musl"
        return target
    return f"{sys.platform}-{platform.machine()}"


def build_release(target: str | None = None) -> None:
    triple = target or os.environ.get("CARGO_BUILD_TARGET")
    label = platform_label(triple)
    print(f"Building Rust release binaries for {label} (this takes a minute)")
    command = ["cargo", "build", "--release", "--bin", "food-mcp", "--bin", "food"]
    if triple:
        command += ["--target", triple]
    completed = subprocess.run(command, cwd=REPO_ROOT, check=False)
    if completed.returncode != 0:
        raise SystemExit("cargo build failed")


def copy_binaries(destination: Path, target: str | None = None) -> None:
    binaries_dir = destination / "bin"
    binaries_dir.mkdir(parents=True, exist_ok=True)
    built = target_dir(target)
    for name in BINARIES:
        source = built / name
        if not source.is_file():
            raise SystemExit(
                f"missing built binary {source}; "
                "run `cargo build --release` or drop --skip-build"
            )
        dest = binaries_dir / name
        shutil.copyfile(source, dest)
        dest.chmod(0o755)


def copy_template(destination: Path) -> None:
    for source in sorted(TEMPLATE_ROOT.rglob("*")):
        if source.is_dir():
            continue
        target = destination / source.relative_to(TEMPLATE_ROOT)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        # copyfile drops the mode, so reinstate the exec bit for shell scripts.
        if source.suffix == ".sh":
            target.chmod(0o755)


# Everything a previous packaging run may have written. The Python-era
# scaffold is listed too, so upgrading from it leaves no stale runtime behind.
GENERATED = (
    "bin",
    "skills",
    "foods",
    "foodsolver",
    "runtime",
    "scripts",
    "pyproject.toml",
    ".mcp.json",
    ".codex-plugin/plugin.json",
)


def remove_generated_files(destination: Path) -> None:
    for relative in GENERATED:
        target = destination / relative
        if target.is_dir():
            shutil.rmtree(target, ignore_errors=True)
        else:
            target.unlink(missing_ok=True)


def write_manifest(destination: Path, target: str | None = None) -> None:
    manifest = {
        "name": PLUGIN_NAME,
        "version": VERSION,
        "description": (
            "Plan homemade dog food batches and check them against FEDIAF "
            "adult maintenance requirements."
        ),
        "author": {"name": "Food Solver"},
        "keywords": ["dog", "nutrition", f"platform:{platform_label(target)}"],
        "skills": "./skills/",
        "mcpServers": "./.mcp.json",
        "interface": {
            "displayName": "Food Solver",
            "shortDescription": "Solve dog food recipes and their nutrient report.",
            "longDescription": (
                "Food Solver turns ingredient amounts and a day count into an "
                "optimised recipe plus a per-day nutrient report for adult "
                "dogs, based on FEDIAF 2025 Table III-3b. It is a self-contained "
                "Rust binary with the food database and the Clarabel solver "
                "compiled in."
            ),
            "developerName": "Food Solver",
            "category": "Productivity",
            "capabilities": ["Interactive"],
            "defaultPrompt": [
                "Solve a 7-day batch for these ingredients.",
                "Why is this recipe low in zinc?",
                "List the foods you can use.",
            ],
        },
    }
    path = destination / ".codex-plugin" / "plugin.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def write_archive(destination: Path, archive: Path) -> None:
    """Tar the plugin folder so it can be copied to another machine."""
    if archive.name.endswith(".tar.gz"):
        stem = archive.name[: -len(".tar.gz")]
    else:
        stem = archive.stem or PLUGIN_NAME
    archive.parent.mkdir(parents=True, exist_ok=True)
    produced = archive.parent / f"{stem}.tar.gz"
    with tarfile.open(produced, "w:gz") as bundle:
        bundle.add(destination, arcname=destination.name)
    print(f"Wrote archive: {produced} ({produced.stat().st_size / 1_000_000:.1f} MB)")


def update_marketplace(marketplace: Path, marketplace_name: str) -> None:
    if marketplace.is_file():
        payload = json.loads(marketplace.read_text(encoding="utf-8"))
    else:
        payload = {
            "name": marketplace_name,
            "interface": {"displayName": "Personal"},
            "plugins": [],
        }
    entry = {
        "name": PLUGIN_NAME,
        "source": {"source": "local", "path": f"./plugins/{PLUGIN_NAME}"},
        "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
        "category": "Productivity",
    }
    plugins = [
        item for item in payload.get("plugins", []) if item.get("name") != PLUGIN_NAME
    ]
    plugins.append(entry)
    payload["plugins"] = plugins
    marketplace.parent.mkdir(parents=True, exist_ok=True)
    marketplace.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dest",
        type=Path,
        default=Path.home() / "plugins",
        help="parent directory of the plugin folder (default: ~/plugins)",
    )
    parser.add_argument(
        "--marketplace",
        type=Path,
        default=Path.home() / ".agents" / "plugins" / "marketplace.json",
        help="personal marketplace JSON to add the plugin to",
    )
    parser.add_argument(
        "--marketplace-name",
        default="personal",
        help="marketplace name used only when the file does not exist yet",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="reuse the existing target/release binaries",
    )
    parser.add_argument(
        "--target",
        default=None,
        metavar="TRIPLE",
        help="cargo target triple to build for, e.g. x86_64-unknown-linux-musl",
    )
    parser.add_argument(
        "--musl",
        action="store_true",
        help="shorthand for --target <host>-unknown-linux-musl (static build)",
    )
    parser.add_argument(
        "--skip-marketplace",
        action="store_true",
        help="only render the plugin folder",
    )
    parser.add_argument(
        "--archive",
        type=Path,
        default=None,
        metavar="PATH",
        help="also write a .tar.gz of the plugin for copying to another host",
    )
    parser.add_argument(
        "--install",
        action="store_true",
        help=f"run `codex plugin add {PLUGIN_NAME}@<marketplace>` afterwards",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if not (REPO_ROOT / "Cargo.toml").is_file():
        print("run this from the food_solver checkout", file=sys.stderr)
        return 1
    if not TEMPLATE_ROOT.is_dir():
        print(f"missing plugin template at {TEMPLATE_ROOT}", file=sys.stderr)
        return 1

    target = args.target
    if target is None and args.musl:
        target = musl_triple()

    if not args.skip_build:
        build_release(target)

    destination = args.dest.expanduser().resolve() / PLUGIN_NAME
    remove_generated_files(destination)
    destination.mkdir(parents=True, exist_ok=True)
    copy_template(destination)
    copy_binaries(destination, target)
    write_manifest(destination, target)
    size = sum(path.stat().st_size for path in destination.rglob("*") if path.is_file())
    print(
        f"Packaged plugin: {destination} "
        f"({size / 1_000_000:.1f} MB, {platform_label(target)})"
    )

    if args.archive is not None:
        write_archive(destination, args.archive.expanduser().resolve())

    if args.skip_marketplace:
        return 0

    marketplace = args.marketplace.expanduser().resolve()
    update_marketplace(marketplace, args.marketplace_name)
    print(f"Updated marketplace: {marketplace}")

    if args.install:
        completed = subprocess.run(
            ["codex", "plugin", "add", f"{PLUGIN_NAME}@{args.marketplace_name}"],
            check=False,
        )
        if completed.returncode != 0:
            print(
                f"Install failed; rerun `codex plugin add "
                f"{PLUGIN_NAME}@{args.marketplace_name}` manually.",
                file=sys.stderr,
            )
            return completed.returncode
        print("Installed. Start a new Codex thread to pick up the tools.")
    else:
        print(f"Install with: codex plugin add {PLUGIN_NAME}@{args.marketplace_name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
