#!/usr/bin/env bash
set -euo pipefail

./tools/build.sh
./tools/build_assets.sh

safe_empty_generated_dir() {
  local dir="$1"
  case "$dir" in
    artifacts/package/oathyard-linux-x86_64) ;;
    *)
      echo "refusing to clean unexpected package directory: $dir" >&2
      exit 1
      ;;
  esac
  if [[ -d "$dir" ]]; then
    find "$dir" -mindepth 1 -depth -delete
  fi
}

package_root="artifacts/package/oathyard-linux-x86_64"
safe_empty_generated_dir "$package_root"
mkdir -p \
  "$package_root/bin" \
  "$package_root/assets" \
  "$package_root/assets_src" \
  "$package_root/content" \
  "$package_root/docs" \
  "$package_root/docs/packaging" \
  "$package_root/examples" \
  "$package_root/share/applications"

cp target/debug/oathyard "$package_root/bin/oathyard"
cp README.md AGENTS.md ACCEPTANCE_MAP.md LICENSE "$package_root/"
cp -R docs/design "$package_root/docs/"
cp -R docs/decisions "$package_root/docs/"
cp -R docs/acceptance "$package_root/docs/"
cp -R docs/asset_pipeline "$package_root/docs/"
cp -R docs/roadmap "$package_root/docs/"
cp -R assets/runtime "$package_root/assets/"
cp -R assets/textures "$package_root/assets/"
cp -R assets/presentation_runtime "$package_root/assets/"
cp -R assets/presentation_gltf "$package_root/assets/"
cp -R assets/model_candidates "$package_root/assets/"
cp -R assets/source/model_candidates "$package_root/assets_src/"
cp assets/runtime_manifest.json assets/asset_provenance_report.md assets/asset_validation_report.md assets/gltf_validation_report.md "$package_root/assets/"
cp assets/manifests/presentation_manifest.json assets/manifests/production_visual_manifest.json assets/manifests/production_candidate_visual_manifest.json "$package_root/assets/"
cp content/oathyard_content.manifest "$package_root/content/"
cp -R examples/duels "$package_root/examples/"
cp packaging/linux/io.oathyard.OATHYARD.desktop "$package_root/share/applications/"
cp packaging/linux/README.md "$package_root/docs/packaging/linux-desktop-metadata.md"
cp packaging/linux/APPSTREAM_BLOCKED.md "$package_root/docs/packaging/linux-appstream-blocked.md"

cat > "$package_root/package_manifest.txt" <<'MANIFEST'
product=OATHYARD
schema=oathyard.package.v1
public_demo_ready=false
release_candidate_ready=false
contents=bin/oathyard,assets/runtime,assets/textures,assets/presentation_runtime,assets/presentation_gltf,assets/model_candidates,assets/source/model_candidates,content,docs,examples,share/applications,README.md,AGENTS.md,ACCEPTANCE_MAP.md,LICENSE,package_checksums.sha256
MANIFEST

(
  cd "$package_root"
  find . -type f \
    ! -name package_checksums.sha256 \
    -print0 \
    | sort -z \
    | xargs -0 sha256sum > package_checksums.sha256
)

mkdir -p artifacts/package
tar --sort=name --mtime='UTC 2026-01-01' --owner=0 --group=0 --numeric-owner \
  -cf artifacts/package/oathyard-linux-x86_64.tar -C artifacts/package oathyard-linux-x86_64

(
  cd artifacts/package
  sha256sum oathyard-linux-x86_64.tar > oathyard-linux-x86_64.tar.sha256
)

echo "package=artifacts/package/oathyard-linux-x86_64.tar"
echo "package_sha256=artifacts/package/oathyard-linux-x86_64.tar.sha256"
echo "contents_sha256=$package_root/package_checksums.sha256"
