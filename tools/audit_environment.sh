#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/environment/verify}"

python3 - "$out" <<'PY'
import glob
import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)

REQUIRED_COMMANDS = [
    "bash",
    "cargo",
    "rustc",
    "python3",
    "tar",
    "sha256sum",
    "find",
    "grep",
    "awk",
    "sed",
    "cmp",
    "sort",
    "xargs",
    "tee",
    "cp",
    "mkdir",
]

OPTIONAL_COMMANDS = [
    "cc",
    "gcc",
    "clang",
    "cmake",
    "make",
    "ninja",
    "pkg-config",
    "desktop-file-validate",
    "xmllint",
    "vulkaninfo",
    "glxinfo",
    "ffmpeg",
    "blender",
    "gltf-validator",
    "gltfpack",
    "toktx",
    "sox",
    "aplay",
    "pw-play",
    "node",
    "npm",
    "zig",
]

PKG_CONFIG_LIBRARIES = [
    "x11",
    "wayland-client",
    "egl",
    "gl",
    "vulkan",
    "sdl2",
    "glfw3",
    "alsa",
    "libpulse",
    "openal",
]

VERSION_ARGS = {
    "cargo": ["--version"],
    "rustc": ["--version"],
    "python3": ["--version"],
    "pkg-config": ["--version"],
    "vulkaninfo": ["--version"],
    "glxinfo": ["-B"],
    "gltfpack": ["-v"],
    "pw-play": ["--version"],
}


def run_version(name: str) -> dict:
    path = shutil.which(name)
    result = {
        "name": name,
        "present": path is not None,
        "path": path or "",
        "version": "",
        "version_rc": None,
    }
    if path is None:
        return result
    args = VERSION_ARGS.get(name, ["--version"])
    try:
        completed = subprocess.run(
            [path, *args],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=3,
            check=False,
        )
        result["version_rc"] = completed.returncode
        for line in completed.stdout.splitlines():
            line = line.strip()
            if line:
                result["version"] = line
                break
    except Exception as error:  # noqa: BLE001 - diagnostic artifact records the failure.
        result["version_rc"] = -1
        result["version"] = f"version_probe_failed: {error}"
    return result


def pkg_config_probe(library: str) -> dict:
    pkg_config = shutil.which("pkg-config")
    result = {
        "name": library,
        "available": False,
        "version": "",
    }
    if pkg_config is None:
        return result
    exists = subprocess.run(
        [pkg_config, "--exists", library],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    result["available"] = exists.returncode == 0
    if result["available"]:
        version = subprocess.run(
            [pkg_config, "--modversion", library],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=3,
            check=False,
        )
        result["version"] = version.stdout.strip().splitlines()[0] if version.stdout.strip() else ""
    return result


def read_os_release() -> dict:
    path = Path("/etc/os-release")
    wanted = {"ID", "NAME", "PRETTY_NAME", "VERSION_ID"}
    data = {}
    if not path.is_file():
        return data
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in wanted:
            data[key.lower()] = value.strip().strip('"')
    return data


required_tools = [run_version(name) for name in REQUIRED_COMMANDS]
optional_tools = [run_version(name) for name in OPTIONAL_COMMANDS]
pkg_config_libraries = [pkg_config_probe(name) for name in PKG_CONFIG_LIBRARIES]

required_missing = [tool["name"] for tool in required_tools if not tool["present"]]
required_version_probe_failures = [
    tool["name"]
    for tool in required_tools
    if tool["present"] and tool["version_rc"] not in (0, None) and not tool["version"]
]

pkg_lookup = {entry["name"]: entry for entry in pkg_config_libraries}
optional_lookup = {entry["name"]: entry for entry in optional_tools}

native_graphics = {
    "display_set": bool(os.environ.get("DISPLAY")),
    "wayland_display_set": bool(os.environ.get("WAYLAND_DISPLAY")),
    "x11_pkg_config_available": pkg_lookup["x11"]["available"],
    "wayland_client_pkg_config_available": pkg_lookup["wayland-client"]["available"],
    "egl_pkg_config_available": pkg_lookup["egl"]["available"],
    "opengl_pkg_config_available": pkg_lookup["gl"]["available"],
    "vulkan_pkg_config_available": pkg_lookup["vulkan"]["available"],
    "vulkaninfo_present": optional_lookup["vulkaninfo"]["present"],
    "glxinfo_present": optional_lookup["glxinfo"]["present"],
    "sdl2_pkg_config_available": pkg_lookup["sdl2"]["available"],
    "glfw3_pkg_config_available": pkg_lookup["glfw3"]["available"],
}

runtime_surfaces = {
    "os_release": read_os_release(),
    "uname_system": platform.system(),
    "uname_release": platform.release(),
    "uname_machine": platform.machine(),
    "shell": os.environ.get("SHELL", ""),
    "display_set": native_graphics["display_set"],
    "wayland_display_set": native_graphics["wayland_display_set"],
    "xdg_session_type": os.environ.get("XDG_SESSION_TYPE", ""),
    "xdg_runtime_dir_set": bool(os.environ.get("XDG_RUNTIME_DIR")),
    "linux_joystick_devices": sorted(glob.glob("/dev/input/js*")),
}

unavailable_optional = sorted(
    tool["name"] for tool in optional_tools if not tool["present"]
)
unavailable_pkg_config = sorted(
    entry["name"] for entry in pkg_config_libraries if not entry["available"]
)

passed = not required_missing and not required_version_probe_failures

report = {
    "schema": "oathyard.environment_audit.v1",
    "product": "OATHYARD",
    "purpose": "local_build_runtime_environment_definition",
    "required_tools": required_tools,
    "required_missing": required_missing,
    "required_version_probe_failures": required_version_probe_failures,
    "required_tools_present": not required_missing,
    "optional_tools": optional_tools,
    "optional_unavailable": unavailable_optional,
    "pkg_config_libraries": pkg_config_libraries,
    "pkg_config_unavailable": unavailable_pkg_config,
    "native_graphics": native_graphics,
    "runtime_surfaces": runtime_surfaces,
    "clean_vm_or_container_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "owner_final_acceptance": False,
    "legal_clearance": False,
    "trademark_clearance": False,
    "store_readiness": False,
    "passed": passed,
}

(out / "environment_audit.json").write_text(
    json.dumps(report, indent=2) + "\n",
    encoding="utf-8",
)

lines = [
    "# OATHYARD Environment Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    "",
    "## Required Local Gate Tools",
    "",
]
for tool in required_tools:
    state = "present" if tool["present"] else "missing"
    version = tool["version"] or "version unavailable"
    lines.append(f"- `{tool['name']}`: `{state}` - {version}")

lines.extend(
    [
        "",
        "## Optional Tool Surface",
        "",
    ]
)
for tool in optional_tools:
    state = "present" if tool["present"] else "missing"
    version = tool["version"] or "version unavailable"
    lines.append(f"- `{tool['name']}`: `{state}` - {version}")

lines.extend(
    [
        "",
        "## pkg-config Libraries",
        "",
    ]
)
for library in pkg_config_libraries:
    state = "available" if library["available"] else "unavailable"
    version = library["version"] or "version unavailable"
    lines.append(f"- `{library['name']}`: `{state}` - {version}")

lines.extend(
    [
        "",
        "## Native Runtime Surface",
        "",
        f"- DISPLAY set: `{str(runtime_surfaces['display_set']).lower()}`",
        f"- WAYLAND_DISPLAY set: `{str(runtime_surfaces['wayland_display_set']).lower()}`",
        f"- XDG session type: `{runtime_surfaces['xdg_session_type'] or 'unknown'}`",
        f"- Linux joystick devices: `{len(runtime_surfaces['linux_joystick_devices'])}`",
        f"- Clean VM/container claimed: `{str(report['clean_vm_or_container_claimed']).lower()}`",
        "",
        "## Readiness Flags",
        "",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Owner final acceptance: `false`",
        "- Legal clearance: `false`",
        "- Trademark clearance: `false`",
        "- Store readiness: `false`",
    ]
)

if required_missing:
    lines.extend(["", "## Missing Required Tools", ""])
    for name in required_missing:
        lines.append(f"- `{name}`")

(out / "environment_audit_report.md").write_text(
    "\n".join(lines) + "\n",
    encoding="utf-8",
)

if not passed:
    print("environment audit failed: required local gate tool missing or unprobeable", file=sys.stderr)
    sys.exit(1)
PY
