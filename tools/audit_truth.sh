#!/usr/bin/env bash
set -euo pipefail

fail=0
mapfile -t rust_sources < <(find src -name '*.rs' -type f | sort)

check_forbidden() {
  local label="$1"
  local pattern="$2"
  if grep -InE "$pattern" "${rust_sources[@]}"; then
    echo "truth audit failed: $label" >&2
    fail=1
  fi
}

check_forbidden "gameplay float types" '\b(f32|f64)\b'
check_forbidden "wall-clock APIs" '(SystemTime|Instant|std::time)'
check_forbidden "hidden RNG APIs" '(rand::|thread_rng|StdRng|SmallRng|random\()'
check_forbidden "unordered truth collections" '\b(HashMap|HashSet)\b'
check_forbidden "shortcut body meters" '(hit_points|health_points|\bhp\b|armor_points)'
check_forbidden "arbitrary combat stats" '(dps|crit_chance|super_meter|bonus_damage|damage_bonus|speed_bonus|\+damage|\+speed)'

if ! grep -q 'pub const TRUTH_HZ: u32 = 120;' src/lib.rs; then
  echo "truth audit failed: TRUTH_HZ is not fixed at 120" >&2
  fail=1
fi

if ! grep -q 'PUBLIC_DEMO_READY: bool = false' src/lib.rs; then
  echo "truth audit failed: public demo readiness flag is not false" >&2
  fail=1
fi

if ! grep -q 'RELEASE_CANDIDATE_READY: bool = false' src/lib.rs; then
  echo "truth audit failed: release candidate readiness flag is not false" >&2
  fail=1
fi

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

echo "truth audit passed"
