#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

package_version=$(cargo metadata --no-deps --format-version 1 \
  | jq -r '.packages[] | select(.name == "cv-linter") | .version')

if [[ -z "$package_version" || "$package_version" == "null" ]]; then
  echo "could not read the cv-linter package version" >&2
  exit 1
fi

for manifest in .claude-plugin/plugin.json packaging/mcpb/manifest.json; do
  manifest_version=$(jq -r '.version' "$manifest")
  if [[ "$manifest_version" != "$package_version" ]]; then
    echo "$manifest version $manifest_version does not match Cargo.toml $package_version" >&2
    exit 1
  fi
done

if ! grep -Fq "## [$package_version]" CHANGELOG.md; then
  echo "CHANGELOG.md has no release section for $package_version" >&2
  exit 1
fi

if [[ $# -gt 0 ]]; then
  expected_tag="v$package_version"
  if [[ "$1" != "$expected_tag" ]]; then
    echo "release tag $1 does not match package version $expected_tag" >&2
    exit 1
  fi
fi

printf 'release metadata agrees on %s\n' "$package_version"
