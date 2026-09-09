#!/usr/bin/env bash
# Build/install the local binary. No remote installer, model download, or config replacement.
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
install_dir="${RECALLFORGE_BIN_DIR:-$HOME/.local/bin}"
cd "$project_dir"
cargo build --locked --release
mkdir -p "$install_dir"
install -m 755 target/release/recallforge "$install_dir/recallforge"
"$install_dir/recallforge" --version
printf 'Installed at %s/recallforge\n' "$install_dir"
