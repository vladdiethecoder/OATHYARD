#!/usr/bin/env bash
set -euo pipefail

out_dir="${1:-artifacts/package_repro/latest}"

safe_empty_generated_dir() {
  local dir="$1"
  case "$dir" in
    artifacts/package_repro/*) ;;
    *)
      echo "refusing to clean unexpected package reproducibility directory: $dir" >&2
      exit 1
      ;;
  esac
  if [[ -d "$dir" ]]; then
    find "$dir" -mindepth 1 -depth -delete
  fi
}

safe_empty_generated_dir "$out_dir"
mkdir -p "$out_dir"

./tools/package.sh
cp artifacts/package/oathyard-linux-x86_64.tar "$out_dir/oathyard-linux-x86_64.first.tar"
cp artifacts/package/oathyard-linux-x86_64.tar.sha256 "$out_dir/oathyard-linux-x86_64.first.tar.sha256"
cp artifacts/package/oathyard-linux-x86_64/package_checksums.sha256 "$out_dir/package_checksums.first.sha256"

./tools/package.sh
cp artifacts/package/oathyard-linux-x86_64.tar "$out_dir/oathyard-linux-x86_64.second.tar"
cp artifacts/package/oathyard-linux-x86_64.tar.sha256 "$out_dir/oathyard-linux-x86_64.second.tar.sha256"
cp artifacts/package/oathyard-linux-x86_64/package_checksums.sha256 "$out_dir/package_checksums.second.sha256"

cmp "$out_dir/oathyard-linux-x86_64.first.tar" "$out_dir/oathyard-linux-x86_64.second.tar"
cmp "$out_dir/package_checksums.first.sha256" "$out_dir/package_checksums.second.sha256"

{
  echo "# OATHYARD Package Reproducibility Report"
  echo
  echo "Status: PASSED"
  echo
  echo "First package:"
  sha256sum "$out_dir/oathyard-linux-x86_64.first.tar"
  echo
  echo "Second package:"
  sha256sum "$out_dir/oathyard-linux-x86_64.second.tar"
  echo
  echo "Package content checksum manifest: identical"
  echo "Byte comparison: identical"
  echo
  echo "Public demo ready: false"
  echo "Release candidate ready: false"
} > "$out_dir/package_repro_report.md"

echo "package reproducibility passed"
echo "report=$out_dir/package_repro_report.md"
