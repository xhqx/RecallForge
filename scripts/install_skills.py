#!/usr/bin/env python3
"""Install the three skills without replacing an existing skill directory."""
import argparse
import os
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--destination", type=Path, default=Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex"))) / "skills")
args = parser.parse_args()
sources = sorted((root / "skills").glob("recallforge-*"))
for source in sources:
    destination = args.destination / source.name
    if destination.exists() or destination.is_symlink():
        raise SystemExit(f"Refusing to replace existing skill: {destination}")
args.destination.mkdir(parents=True, exist_ok=True)
for source in sources:
    shutil.copytree(source, args.destination / source.name)
    print(f"Installed {source.name}")
