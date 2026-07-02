#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/desktop_metadata/verify}"
src_dir="packaging/linux"
desktop="$src_dir/io.oathyard.OATHYARD.desktop"
icon="removed_by_3d_only_visual_policy"
blocker="$src_dir/APPSTREAM_BLOCKED.md"

mkdir -p "$out"

desktop_validator="fallback"
desktop_valid=true
desktop_error=""
if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop_validator="desktop-file-validate"
  if ! desktop-file-validate "$desktop" >"$out/desktop-file-validate.log" 2>&1; then
    desktop_valid=false
    desktop_error="$(tr '\n' ' ' < "$out/desktop-file-validate.log")"
  fi
else
  for pattern in '^\[Desktop Entry\]$' '^Type=Application$' '^Name=OATHYARD$' '^Exec=oathyard$' '^Icon=io.oathyard.OATHYARD$' '^Terminal=false$' '^Categories=Game;$'; do
    if ! grep -qE "$pattern" "$desktop"; then
      desktop_valid=false
      desktop_error="fallback desktop validation missed pattern $pattern"
    fi
  done
fi

icon_validator="not_applicable_3d_only_visual_policy"
icon_valid=true
icon_error=""

exec_value="$(awk -F= '/^Exec=/ {print $2; exit}' "$desktop")"
icon_value="$(awk -F= '/^Icon=/ {print $2; exit}' "$desktop")"
name_value="$(awk -F= '/^Name=/ {print $2; exit}' "$desktop")"
appstream_generated=false
appstream_blocker="license_pending_unlicensed_project"

if [[ "$exec_value" != "oathyard" || "$icon_value" != "io.oathyard.OATHYARD" || "$name_value" != "OATHYARD" ]]; then
  desktop_valid=false
  desktop_error="${desktop_error} required desktop fields did not match OATHYARD package contract"
fi

if ! grep -q 'Status: BLOCKED_LICENSE_PENDING' "$blocker"; then
  appstream_blocker="missing_blocker_note"
fi

passed=false
if [[ "$desktop_valid" == "true" && "$icon_valid" == "true" && "$appstream_blocker" == "license_pending_unlicensed_project" ]]; then
  passed=true
fi

cat > "$out/desktop_metadata.json" <<JSON
{
  "schema": "oathyard.desktop_metadata.v1",
  "product": "OATHYARD",
  "desktop_entry": "$desktop",
  "desktop_validator": "$desktop_validator",
  "desktop_entry_valid": $desktop_valid,
  "desktop_exec": "$exec_value",
  "desktop_icon": "$icon_value",
  "icon_source": "$icon",
  "icon_validator": "$icon_validator",
  "icon_valid_xml": $icon_valid,
  "appstream_metadata_generated": $appstream_generated,
  "appstream_blocker": "$appstream_blocker",
  "public_demo_ready": false,
  "release_candidate_ready": false,
  "owner_final_acceptance": false,
  "legal_clearance": false,
  "trademark_clearance": false,
  "passed": $passed
}
JSON

cat > "$out/desktop_metadata_report.md" <<REPORT
# OATHYARD Desktop Metadata Report

Status: $(if [[ "$passed" == "true" ]]; then echo "PASSED_LOCAL_DESKTOP_ENTRY"; else echo "FAILED"; fi)

- Desktop entry: \`$desktop\`
- Desktop validator: \`$desktop_validator\`
- Desktop entry valid: \`$desktop_valid\`
- Desktop Exec: \`$exec_value\`
- Desktop Icon: \`$icon_value\`
- Icon source: \`$icon\`
- Icon validator: \`$icon_validator\`
- Icon valid XML: \`$icon_valid\`
- AppStream metadata generated: \`$appstream_generated\`
- AppStream blocker: \`$appstream_blocker\`
- Public demo ready: \`false\`
- Release candidate ready: \`false\`
- Owner final acceptance: \`false\`
- Legal clearance: \`false\`
- Trademark clearance: \`false\`
REPORT

if [[ "$passed" != "true" ]]; then
  {
    echo "desktop_error=$desktop_error"
    echo "icon_error=$icon_error"
  } >&2
  exit 1
fi

echo "desktop metadata passed"
