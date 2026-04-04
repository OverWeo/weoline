#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

CARGO_TOML="$REPO_ROOT/Cargo.toml"
MAIN_PKG="$REPO_ROOT/npm/weoline/package.json"
PLATFORM_PKGS=(
  "$REPO_ROOT/npm/weoline-darwin-arm64/package.json"
  "$REPO_ROOT/npm/weoline-darwin-x64/package.json"
  "$REPO_ROOT/npm/weoline-linux-x64/package.json"
  "$REPO_ROOT/npm/weoline-win32-x64/package.json"
)

# Read current version from Cargo.toml (first match only = package version)
current_version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$CARGO_TOML" | head -1)

if [ -z "$current_version" ]; then
  echo "Error: could not read version from Cargo.toml" >&2
  exit 1
fi

if [ $# -ge 1 ]; then
  new_version="$1"
else
  # Auto-increment patch: X.Y.Z -> X.Y.(Z+1)
  IFS='.' read -r major minor patch <<< "$current_version"
  new_version="$major.$minor.$((patch + 1))"
fi

echo "Bumping version: $current_version -> $new_version"
echo ""

# 1. Cargo.toml
sed -i '' "s/^version = \"$current_version\"/version = \"$new_version\"/" "$CARGO_TOML"
echo "  patched: Cargo.toml"

# 2. Main npm package — version + optionalDependencies
sed -i '' "s/\"$current_version\"/\"$new_version\"/g" "$MAIN_PKG"
echo "  patched: npm/weoline/package.json"

# 3. Platform npm packages
for pkg in "${PLATFORM_PKGS[@]}"; do
  sed -i '' "s/\"$current_version\"/\"$new_version\"/g" "$pkg"
  echo "  patched: ${pkg#$REPO_ROOT/}"
done

echo ""
echo "Done. All 6 files set to $new_version."
