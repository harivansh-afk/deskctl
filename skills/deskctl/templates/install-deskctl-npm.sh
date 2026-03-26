#!/usr/bin/env bash
set -euo pipefail

if command -v deskctl >/dev/null 2>&1; then
  echo "deskctl already installed: $(command -v deskctl)"
  exit 0
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required for the preferred deskctl install path"
  exit 1
fi

prefix="${DESKCTL_NPM_PREFIX:-$HOME/.local}"
bin_dir="$prefix/bin"

mkdir -p "$bin_dir"
npm install -g --prefix "$prefix" deskctl-cli

if ! command -v deskctl >/dev/null 2>&1; then
  echo "deskctl installed to $bin_dir"
  echo "add this to PATH if needed:"
  echo "export PATH=\"$bin_dir:\$PATH\""
fi

"$bin_dir/deskctl" --help >/dev/null 2>&1 || true
echo "deskctl bootstrap complete"
