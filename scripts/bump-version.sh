#!/bin/bash
# Usage: ./scripts/bump-version.sh <version>
# Example: ./scripts/bump-version.sh 1.2.0
#
# Updates version in package.json, tauri.conf.json, Cargo.toml,
# commits the change, and creates a git tag.

set -e

VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/bump-version.sh <version>"
  echo "Example: ./scripts/bump-version.sh 1.2.0"
  exit 1
fi

if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Error: Version must be in semver format (e.g., 1.2.0)"
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# package.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$ROOT_DIR/package.json"

# tauri.conf.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$ROOT_DIR/src-tauri/tauri.conf.json"

# Cargo.toml — only the package version line (line 3)
sed -i "0,/^version = \"[^\"]*\"/s//version = \"$VERSION\"/" "$ROOT_DIR/src-tauri/Cargo.toml"

echo "Updated version to $VERSION in:"
echo "  - package.json"
echo "  - src-tauri/tauri.conf.json"
echo "  - src-tauri/Cargo.toml"

# Commit and tag
git add "$ROOT_DIR/package.json" "$ROOT_DIR/src-tauri/tauri.conf.json" "$ROOT_DIR/src-tauri/Cargo.toml"
git commit -m "chore: bump version to $VERSION"
git tag "v$VERSION"

echo ""
echo "Committed and tagged v$VERSION"
echo "Run 'git push && git push --tags' to publish."
