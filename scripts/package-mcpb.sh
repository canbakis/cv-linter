#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 5 ]]; then
  echo "usage: $0 <version> <mac-arm64.tar.xz> <mac-x64.tar.xz> <windows-x64.zip> <output.mcpb>" >&2
  exit 2
fi

version=$1
mac_arm_archive=$2
mac_x64_archive=$3
windows_x64_archive=$4
output=$5
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work_dir=$(mktemp -d "${TMPDIR:-/tmp}/cv-linter-mcpb.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT

stage="$work_dir/stage"
mkdir -p \
  "$stage/server/bin/aarch64-apple-darwin" \
  "$stage/server/bin/x86_64-apple-darwin" \
  "$stage/server/bin/x86_64-pc-windows-msvc"

extract_binary() {
  local archive=$1
  local binary_name=$2
  local destination=$3
  local extract_dir
  extract_dir=$(mktemp -d "$work_dir/extract.XXXXXX")

  case "$archive" in
    *.tar.xz) tar -xJf "$archive" -C "$extract_dir" ;;
    *.zip) unzip -q "$archive" -d "$extract_dir" ;;
    *)
      echo "unsupported release archive: $archive" >&2
      return 2
      ;;
  esac

  local binary_path
  binary_path=$(find "$extract_dir" -type f -name "$binary_name" -print -quit)
  if [[ -z "$binary_path" ]]; then
    echo "$binary_name was not found in $archive" >&2
    return 1
  fi

  cp "$binary_path" "$destination"
}

extract_binary "$mac_arm_archive" cv-linter \
  "$stage/server/bin/aarch64-apple-darwin/cv-linter"
extract_binary "$mac_x64_archive" cv-linter \
  "$stage/server/bin/x86_64-apple-darwin/cv-linter"
extract_binary "$windows_x64_archive" cv-linter.exe \
  "$stage/server/bin/x86_64-pc-windows-msvc/cv-linter.exe"

cp "$repo_root/packaging/mcpb/manifest.json" "$stage/manifest.json"
cp "$repo_root/packaging/mcpb/server/index.js" "$stage/server/index.js"
cp "$repo_root/LICENSE" "$repo_root/README.md" "$repo_root/CHANGELOG.md" "$stage/"
cp "$repo_root/THIRD-PARTY-LICENSES.html" "$stage/"
cp -R "$repo_root/dictionaries" "$stage/dictionaries"
chmod 755 "$stage/server/index.js" "$stage/server/bin"/*/cv-linter

node - "$stage/manifest.json" "$version" <<'NODE'
const fs = require("node:fs");
const [manifestPath, version] = process.argv.slice(2);
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
manifest.version = version;
fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
NODE

mkdir -p "$(dirname "$output")"
npx --yes @anthropic-ai/mcpb@2.1.2 pack "$stage" "$output"
npx --yes @anthropic-ai/mcpb@2.1.2 info "$output"
