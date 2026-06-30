#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/sim_reference_compare/latest}"
scenario="${2:-examples/duels/basic_oathyard.duel}"
mkdir -p "$out"
rm -rf \
  "$out/internal_truth" \
  "$out/internal_truth_after_reference" \
  "$out/internal_fixtures" \
  "$out/external_references"

./tools/run_duel.sh "$scenario" --out "$out/internal_truth" > "$out/internal_truth.log" 2>&1

python3 - "$out" "$scenario" <<'PY'
import hashlib
import importlib.util
import json
import shutil
import subprocess
import sys
from pathlib import Path

out = Path(sys.argv[1])
scenario = sys.argv[2]
python = sys.executable or "python3"

REFERENCE_STACKS = [
    {
        "id": "warp",
        "label": "NVIDIA Warp",
        "candidates": [{"kind": "module", "name": "warp"}],
        "intended_layer": "offline_research_authoring",
    },
    {
        "id": "newton",
        "label": "Newton Physics",
        "candidates": [{"kind": "module", "name": "newton"}],
        "intended_layer": "offline_research_authoring",
    },
    {
        "id": "mujoco_warp",
        "label": "MuJoCo Warp / MJWarp",
        "candidates": [{"kind": "module", "name": "mujoco_warp"}],
        "intended_layer": "offline_research_authoring",
    },
    {
        "id": "mujoco",
        "label": "MuJoCo",
        "candidates": [{"kind": "module", "name": "mujoco"}],
        "intended_layer": "offline_research_authoring",
    },
    {
        "id": "physx",
        "label": "NVIDIA PhysX",
        "candidates": [
            {"kind": "module", "name": "ovphysx"},
            {"kind": "command", "argv": ["physx", "--version"]},
        ],
        "intended_layer": "offline_research_authoring",
    },
    {
        "id": "chrono",
        "label": "Project Chrono / PyChrono",
        "candidates": [
            {"kind": "module", "name": "pychrono"},
            {"kind": "module", "name": "pychrono.core"},
            {"kind": "module", "name": "chrono"},
        ],
        "intended_layer": "offline_research_authoring",
    },
]


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path, payload):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def rel(path):
    return path.relative_to(out).as_posix()


def check(checks, failures, check_id, passed, detail):
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})
    if not passed:
        failures.append(f"{check_id}: {detail}")


def contains_float(value):
    if isinstance(value, float):
        return True
    if isinstance(value, dict):
        return any(contains_float(item) for item in value.values())
    if isinstance(value, list):
        return any(contains_float(item) for item in value)
    return False


def import_version(module):
    code = (
        "import importlib; "
        f"m=importlib.import_module({module!r}); "
        "print(getattr(m, '__version__', 'unknown'))"
    )
    return subprocess.run([python, "-c", code], capture_output=True, text=True, timeout=10)


def probe_module(module):
    try:
        spec = importlib.util.find_spec(module)
    except Exception as exc:  # noqa: BLE001
        return {"kind": "module", "name": module, "available": False, "detail": f"find_spec failed: {exc}"}
    if spec is None:
        return {"kind": "module", "name": module, "available": False, "detail": "module not found"}
    try:
        proc = import_version(module)
    except Exception as exc:  # noqa: BLE001
        return {"kind": "module", "name": module, "available": False, "detail": f"import probe failed: {exc}"}
    detail = (proc.stdout or proc.stderr).strip() or "available"
    return {"kind": "module", "name": module, "available": proc.returncode == 0, "detail": detail}


def probe_command(argv):
    exe = argv[0]
    if shutil.which(exe) is None:
        return {"kind": "command", "argv": argv, "available": False, "detail": f"command not found: {exe}"}
    try:
        proc = subprocess.run(argv, capture_output=True, text=True, timeout=10)
    except Exception as exc:  # noqa: BLE001
        return {"kind": "command", "argv": argv, "available": False, "detail": str(exc)}
    detail = (proc.stdout or proc.stderr).strip() or "available"
    return {"kind": "command", "argv": argv, "available": proc.returncode == 0, "detail": detail}


def probe_reference_stack(stack):
    probes = []
    for candidate in stack["candidates"]:
        if candidate["kind"] == "module":
            probe = probe_module(candidate["name"])
        elif candidate["kind"] == "command":
            probe = probe_command(candidate["argv"])
        else:
            probe = {"kind": candidate["kind"], "available": False, "detail": "unsupported probe kind"}
        probes.append(probe)
        if probe["available"]:
            return {
                "id": stack["id"],
                "label": stack["label"],
                "module": probe.get("name") or "/".join(probe.get("argv", [])),
                "available": True,
                "version": probe["detail"],
                "intended_layer": stack["intended_layer"],
                "probes": probes,
            }
    detail = "; ".join(probe["detail"] for probe in probes) if probes else "no probes"
    first = stack["candidates"][0]
    module = first.get("name") or "/".join(first.get("argv", []))
    return {
        "id": stack["id"],
        "label": stack["label"],
        "module": module,
        "available": False,
        "version": detail,
        "intended_layer": stack["intended_layer"],
        "probes": probes,
    }


def truth_core(trace, replay, final_hash_text):
    return {
        "content_hash": replay.get("content_hash"),
        "end_condition_status": replay.get("end_condition_status"),
        "end_condition_winner": replay.get("end_condition_winner"),
        "final_state_hash": replay.get("final_state_hash"),
        "final_hash_text": final_hash_text,
        "initial_state_hash": replay.get("initial_state_hash"),
        "replay_schema": replay.get("schema"),
        "trace_schema": trace.get("schema"),
        "trace_turn_count": len(trace.get("turns", [])),
        "truth_hz": trace.get("truth_hz"),
        "turn_hashes": replay.get("turn_hashes", []),
    }


def collect_truth(path):
    trace = read_json(path / "trace.json")
    replay = read_json(path / "replay.json")
    final_hash_text = (path / "final_state_hash.txt").read_text(encoding="utf-8").strip()
    return {"trace": trace, "replay": replay, "core": truth_core(trace, replay, final_hash_text)}


def contact_records(trace):
    records = []
    for turn in trace.get("turns", []):
        for contact in turn.get("contacts", []):
            record = {
                "turn": contact.get("turn"),
                "frame": contact.get("frame"),
                "attacker": contact.get("attacker"),
                "defender": contact.get("defender"),
                "action": contact.get("action"),
                "direction": contact.get("direction"),
                "target": contact.get("target"),
                "weapon": contact.get("weapon"),
                "armor": contact.get("armor"),
                "energy_milli": contact.get("energy_milli"),
                "impulse_milli": contact.get("impulse_milli"),
                "material_result": contact.get("material_result"),
                "anatomy_result": contact.get("anatomy_result"),
                "capability_delta": contact.get("capability_delta", {}),
                "cause_chain": contact.get("cause_chain"),
            }
            records.append(record)
    return records


def contact_order_keys(contacts):
    return [
        [
            item.get("turn"),
            item.get("frame"),
            item.get("attacker"),
            item.get("defender"),
            item.get("action"),
            item.get("target"),
            item.get("direction"),
        ]
        for item in contacts
    ]


def make_fixture(path, kind, payload):
    write_json(path, payload)
    return {"kind": kind, "path": rel(path), "sha256": sha256_file(path), "truth_mutation_allowed": False}


checks = []
failures = []
before = collect_truth(out / "internal_truth")
trace = before["trace"]
replay = before["replay"]
core = before["core"]
contacts = contact_records(trace)
order_keys = contact_order_keys(contacts)
expected_order = sorted(order_keys)

check(checks, failures, "trace_schema", trace.get("schema") == "oathyard.trace.v1", str(trace.get("schema")))
check(checks, failures, "replay_schema", replay.get("schema") == "oathyard.replay.v1", str(replay.get("schema")))
check(checks, failures, "truth_hz_120", trace.get("truth_hz") == 120 and replay.get("truth_hz") == 120, f"trace={trace.get('truth_hz')} replay={replay.get('truth_hz')}")
check(checks, failures, "final_hash_matches_replay", core["final_hash_text"] == replay.get("final_state_hash"), f"txt={core['final_hash_text']} replay={replay.get('final_state_hash')}")
check(checks, failures, "trace_final_hash_matches_replay", trace.get("final_state_hash") == replay.get("final_state_hash"), f"trace={trace.get('final_state_hash')} replay={replay.get('final_state_hash')}")
check(checks, failures, "content_hash_matches", trace.get("content_hash") == replay.get("content_hash"), f"trace={trace.get('content_hash')} replay={replay.get('content_hash')}")
check(checks, failures, "turn_hash_count_matches_trace", len(replay.get("turn_hashes", [])) == len(trace.get("turns", [])), f"turn_hashes={len(replay.get('turn_hashes', []))} turns={len(trace.get('turns', []))}")
check(checks, failures, "contact_order_rule_declared", trace.get("contact_order_rule") == "frame_then_attacker_then_defender_then_action_then_target_then_direction", str(trace.get("contact_order_rule")))
check(checks, failures, "contact_order_deterministic", order_keys == expected_order, f"contacts={len(order_keys)}")
check(checks, failures, "truth_json_has_no_floats", not contains_float(trace) and not contains_float(replay), "trace/replay contain only integer/string/bool/list/object JSON values")
check(checks, failures, "public_demo_ready_false", trace.get("public_demo_ready") is False, str(trace.get("public_demo_ready")))
check(checks, failures, "release_candidate_ready_false", trace.get("release_candidate_ready") is False, str(trace.get("release_candidate_ready")))

availability = [probe_reference_stack(stack) for stack in REFERENCE_STACKS]

fixture_dir = out / "internal_fixtures"
fixture_dir.mkdir(parents=True, exist_ok=True)
total_energy = sum(int(item.get("energy_milli") or 0) for item in contacts)
total_impulse = sum(int(item.get("impulse_milli") or 0) for item in contacts)
fixtures = []
fixtures.append(
    make_fixture(
        fixture_dir / "truth_core_fixture.json",
        "truth_core",
        {
            "schema": "oathyard.internal_truth_core_fixture.v1",
            "scenario": scenario,
            "source": "internal_deterministic_truth",
            "authoritative_truth_layer": "internal_deterministic_oathyard_only",
            "truth_mutation_allowed": False,
            "core": core,
        },
    )
)
fixtures.append(
    make_fixture(
        fixture_dir / "contact_packet_fixture.json",
        "contact_packets",
        {
            "schema": "oathyard.contact_packet_reference_fixture.v1",
            "scenario": scenario,
            "source": "internal_truth_trace_after_hash",
            "contact_order_rule": trace.get("contact_order_rule"),
            "truth_mutation_allowed": False,
            "contacts": contacts,
        },
    )
)
fixtures.append(
    make_fixture(
        fixture_dir / "reduced_observables_fixture.json",
        "reduced_observables",
        {
            "schema": "oathyard.reduced_solver_observables_fixture.v1",
            "scenario": scenario,
            "source": "internal_truth_trace_after_hash",
            "truth_mutation_allowed": False,
            "observables": {
                "contact_count": len(contacts),
                "turn_count": len(trace.get("turns", [])),
                "total_energy_milli": total_energy,
                "total_impulse_milli": total_impulse,
                "end_condition_status": replay.get("end_condition_status"),
                "end_condition_winner": replay.get("end_condition_winner"),
            },
        },
    )
)
fixtures.append(
    make_fixture(
        fixture_dir / "truth_boundary_contract.json",
        "truth_boundary_contract",
        {
            "schema": "oathyard.offline_reference_truth_boundary.v1",
            "scenario": scenario,
            "layer": "offline_research_authoring",
            "authoritative_truth_layer": "internal_deterministic_oathyard_only",
            "truth_mutation_allowed": False,
            "external_solver_state_ingested_by_truth": False,
            "may_read_internal_fixtures": True,
            "may_write_internal_truth": False,
            "may_write_replay_json": False,
            "may_write_trace_json": False,
            "may_decide_contacts": False,
            "may_decide_injuries": False,
            "may_decide_capabilities": False,
            "may_decide_end_condition": False,
            "may_write_hashes": False,
        },
    )
)
fixture_manifest_path = fixture_dir / "reference_fixture_manifest.json"
fixture_manifest = {
    "schema": "oathyard.sim_reference_fixture_manifest.v1",
    "scenario": scenario,
    "fixture_count": len(fixtures),
    "fixtures": fixtures,
    "source_truth_fingerprint": core,
    "truth_mutation_allowed": False,
}
write_json(fixture_manifest_path, fixture_manifest)
fixture_manifest_hash = sha256_file(fixture_manifest_path)

contracts = []
comparisons = [
    {
        "reference": "Internal deterministic OATHYARD fixtures",
        "status": "fixture_harness_executed",
        "fixture_manifest": rel(fixture_manifest_path),
        "fixture_manifest_sha256": fixture_manifest_hash,
        "truth_overwrite_allowed": False,
        "external_solver_state_ingested_by_truth": False,
    }
]

for ref in availability:
    ref_dir = out / "external_references" / ref["id"]
    contract_path = ref_dir / "reference_contract.json"
    status = "available_contract_generated_solver_not_executed" if ref["available"] else "unavailable_contract_generated"
    contract = {
        "schema": "oathyard.offline_solver_reference_contract.v1",
        "reference_id": ref["id"],
        "label": ref["label"],
        "layer": ref["intended_layer"],
        "available": ref["available"],
        "availability": ref,
        "status": status,
        "scenario": scenario,
        "source_truth_fingerprint": core,
        "input_fixture_manifest": rel(fixture_manifest_path),
        "input_fixture_manifest_sha256": fixture_manifest_hash,
        "execution": {
            "availability_probe_performed": True,
            "external_solver_execution_performed": False,
            "external_solver_execution_opt_in_env": "OATHYARD_SIM_REFERENCE_EXECUTE_EXTERNAL",
            "reason": "This harness records offline-reference contracts and fixtures. External solver output is never imported into OATHYARD truth.",
        },
        "output_contract": {
            "directory": rel(ref_dir),
            "path_must_stay_under_reference_directory": True,
            "allowed_non_authoritative_files": [
                "reference_contract.json",
                "reference_observables.json",
                "reference_solver_output.json",
                "reference_solver_output.sha256",
            ],
            "external_state_hash_record_required_before_review": True,
        },
        "truth_boundary": {
            "may_read_internal_fixtures": True,
            "may_write_internal_truth": False,
            "may_write_replay_json": False,
            "may_write_trace_json": False,
            "may_decide_contacts": False,
            "may_decide_injuries": False,
            "may_decide_capabilities": False,
            "may_decide_end_condition": False,
            "may_write_hashes": False,
            "truth_overwrite_allowed": False,
            "external_solver_state_ingested_by_truth": False,
        },
    }
    write_json(contract_path, contract)
    contracts.append({
        "reference_id": ref["id"],
        "label": ref["label"],
        "available": ref["available"],
        "status": status,
        "contract_path": rel(contract_path),
        "contract_sha256": sha256_file(contract_path),
    })
    if ref["available"]:
        comparisons.append({
            "reference": ref["label"],
            "status": status,
            "contract_path": rel(contract_path),
            "truth_overwrite_allowed": False,
            "external_solver_state_ingested_by_truth": False,
            "required_before_solver_execution": [
                "scene/source hash",
                "unit mapping",
                "solver version",
                "seed/options",
                "tolerance policy",
                "non-authoritative output path",
                "output hash",
            ],
        })

check(checks, failures, "all_required_reference_contracts_written", len(contracts) == len(REFERENCE_STACKS), f"contracts={len(contracts)} required={len(REFERENCE_STACKS)}")
check(checks, failures, "internal_fixture_manifest_written", fixture_manifest_path.is_file(), rel(fixture_manifest_path))
check(checks, failures, "external_contracts_are_non_authoritative", all(not read_json(out / item["contract_path"])["truth_boundary"]["truth_overwrite_allowed"] for item in contracts), "truth_overwrite_allowed false for every contract")
check(checks, failures, "external_contracts_under_out_dir", all((out / item["contract_path"]).resolve().is_relative_to(out.resolve()) for item in contracts), "all reference contracts stay under artifact dir")

after_dir = out / "internal_truth_after_reference"
after_log = out / "internal_truth_after_reference.log"
with after_log.open("w", encoding="utf-8") as log:
    proc = subprocess.run(["./tools/run_duel.sh", scenario, "--out", str(after_dir)], stdout=log, stderr=subprocess.STDOUT, text=True)
check(checks, failures, "truth_after_reference_run_succeeded", proc.returncode == 0, f"rc={proc.returncode} log={rel(after_log)}")

after = None
if proc.returncode == 0:
    after = collect_truth(after_dir)
    check(checks, failures, "truth_core_stable_after_reference_harness", after["core"] == core, f"before={core} after={after['core']}")
    check(checks, failures, "trace_json_stable_after_reference_harness", sha256_file(out / "internal_truth/trace.json") == sha256_file(after_dir / "trace.json"), "trace sha256 before/after")
    check(checks, failures, "replay_json_stable_after_reference_harness", sha256_file(out / "internal_truth/replay.json") == sha256_file(after_dir / "replay.json"), "replay sha256 before/after")
    check(checks, failures, "final_hash_stable_after_reference_harness", sha256_file(out / "internal_truth/final_state_hash.txt") == sha256_file(after_dir / "final_state_hash.txt"), "final_state_hash.txt sha256 before/after")

passed = not failures
manifest = {
    "schema": "oathyard.sim_reference_compare.v2",
    "tool": "tools/sim_reference_compare.sh",
    "passed": passed,
    "scenario": scenario,
    "authoritative_truth_layer": "internal_deterministic_oathyard_only",
    "truth_overwrite_allowed": False,
    "external_solver_state_ingested_by_truth": False,
    "no_external_solver_truth_promotion": True,
    "comparison_performed": True,
    "external_solver_comparison_performed": False,
    "internal_truth": core,
    "internal_truth_after_reference": after["core"] if after else None,
    "truth_stability_after_reference_harness": bool(after and after["core"] == core),
    "internal_fixture_harness": {
        "fixture_manifest": rel(fixture_manifest_path),
        "fixture_manifest_sha256": fixture_manifest_hash,
        "fixture_count": len(fixtures),
        "fixtures": fixtures,
    },
    "required_reference_stacks": [stack["label"] for stack in REFERENCE_STACKS],
    "external_reference_availability": availability,
    "external_reference_harness": {
        "reference_stack_count": len(REFERENCE_STACKS),
        "available_reference_stack_count": sum(1 for ref in availability if ref["available"]),
        "contracts": contracts,
    },
    "comparisons": comparisons,
    "failed_check_count": len(failures),
    "checks": checks,
}
manifest_path = out / "sim_reference_compare_manifest.json"
write_json(manifest_path, manifest)
(out / "failed_sim_reference_checks.txt").write_text("none\n" if passed else "\n".join(failures) + "\n", encoding="utf-8")

report = [
    "# OATHYARD Simulation Reference Compare",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Scenario: `{scenario}`",
    f"- Internal final hash: `{core['final_state_hash']}`",
    f"- Internal fixture count: `{len(fixtures)}`",
    f"- External reference contracts: `{len(contracts)}`",
    f"- Available external reference stacks: `{sum(1 for ref in availability if ref['available'])}`",
    "- Authoritative truth layer: `internal_deterministic_oathyard_only`",
    "- Truth overwrite allowed: `false`",
    "- External solver state ingested by truth: `false`",
    f"- Truth stable after reference harness: `{str(bool(after and after['core'] == core)).lower()}`",
    "",
    "## Internal fixtures",
    f"- Manifest: `{rel(fixture_manifest_path)}`",
    f"- Manifest SHA-256: `{fixture_manifest_hash}`",
]
for fixture in fixtures:
    report.append(f"- `{fixture['kind']}` `{fixture['path']}` sha256 `{fixture['sha256']}`")

report.extend(["", "## External reference availability"])
for ref in availability:
    report.append(f"- `{ref['label']}` probe `{ref['module']}` available: `{str(ref['available']).lower()}` version/detail: `{ref['version']}`")

report.extend(["", "## Reference contracts"])
for contract in contracts:
    report.append(f"- `{contract['label']}` `{contract['status']}` contract `{contract['contract_path']}` sha256 `{contract['contract_sha256']}`")

report.extend([
    "",
    "## Scope",
    "",
    "This gate builds offline internal reference fixtures and one-way contracts for Warp, Newton, MuJoCo Warp/MJWarp, MuJoCo, PhysX, and Chrono. External stacks may compare against these fixtures outside authoritative truth, but their solver state is not read by OATHYARD truth and cannot overwrite replay, trace, contact, injury, capability, end-condition, or hash data.",
])
if failures:
    report.extend(["", "## Failures"] + [f"- {failure}" for failure in failures])
(out / "sim_reference_compare_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")

if not passed:
    raise SystemExit(1)
PY

echo "sim reference compare: $out"
