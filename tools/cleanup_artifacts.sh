#!/usr/bin/env bash
# Cleanup old artifact directories, keeping only the latest per unit.
set -euo pipefail

ARTIFACTS_DIR="artifacts"
STALE_DIR="${ARTIFACTS_DIR}/_stale"
TIMESTAMP=$(date +%Y-%m-%d)
DRY_RUN=${1:-""}

PROTECTED=("current" "latest" "_stale" "secrets")

is_protected() {
    local name="$1"
    for p in "${PROTECTED[@]}"; do
        [[ "$name" == "$p" ]] && return 0
    done
    return 1
}

CANDIDATES=()
for d in "${ARTIFACTS_DIR}"/*/; do
    name=$(basename "$d")
    is_protected "$name" || CANDIDATES+=("$name")
done

echo "Found ${#CANDIDATES[@]} candidate directories in ${ARTIFACTS_DIR}/"

if [[ ${#CANDIDATES[@]} -eq 0 ]]; then
    echo "Nothing to clean up."
    exit 0
fi

# Build list of dirs with their groups and mtimes
declare -a DIRS_GROUP DIRS_NAME DIRS_MTIME
for name in "${CANDIDATES[@]}"; do
    # Extract group by removing timestamp suffix (e.g., _20260704T102530Z)
    group=$(echo "$name" | sed 's/_[0-9]\{8,\}T[0-9]\{6,\}Z.*//' | sed 's/_[0-9]\{8,\}[-_][0-9]\{6,\}.*//' | sed 's/_.fugu.*//')
    [[ -z "$group" ]] && group="$name"
    
    FULL_PATH="${ARTIFACTS_DIR}/${name}"
    MTIME=$(stat -c %Y "$FULL_PATH" 2>/dev/null || echo 0)
    DIRS_GROUP+=("$group")
    DIRS_NAME+=("$name")
    DIRS_MTIME+=("$MTIME")
done

# Find latest per group
declare -A LATEST_NAME LATEST_MTIME
for i in "${!DIRS_NAME[@]}"; do
    g="${DIRS_GROUP[$i]}"
    n="${DIRS_NAME[$i]}"
    m="${DIRS_MTIME[$i]}"
    if [[ -z "${LATEST_NAME[$g]:-}" ]] || [[ "$m" -gt "${LATEST_MTIME[$g]:-0}" ]]; then
        LATEST_NAME[$g]="$n"
        LATEST_MTIME[$g]="$m"
    fi
done

# Classify
TO_PRESERVE=()
TO_REMOVE=()
for i in "${!DIRS_NAME[@]}"; do
    g="${DIRS_GROUP[$i]}"
    n="${DIRS_NAME[$i]}"
    if [[ "${LATEST_NAME[$g]:-}" == "$n" ]]; then
        TO_PRESERVE+=("$n")
    else
        TO_REMOVE+=("$n")
    fi
done

echo "Preserving (latest per group): ${#TO_PRESERVE[@]} dirs"
echo "Removing (older duplicates):   ${#TO_REMOVE[@]} dirs"
echo ""

# Calculate savings
TOTAL_SAVE=0
for name in "${TO_REMOVE[@]}"; do
    SIZE=$(du -sm "${ARTIFACTS_DIR}/${name}" 2>/dev/null | awk '{print $1}')
    TOTAL_SAVE=$((TOTAL_SAVE + SIZE))
done
echo "Estimated space to free: ~${TOTAL_SAVE} MB"
echo ""

if [[ "$DRY_RUN" == "dry-run" ]]; then
    echo "DRY RUN -- would move to ${STALE_DIR}/${TIMESTAMP}:"
    for name in "${TO_REMOVE[@]}"; do echo "  ${name}"; done
    exit 0
fi

if [[ ${#TO_REMOVE[@]} -eq 0 ]]; then
    echo "Nothing to remove."
    exit 0
fi

mkdir -p "${STALE_DIR}/${TIMESTAMP}"
MOVED=0; FREED=0
for name in "${TO_REMOVE[@]}"; do
    SRC="${ARTIFACTS_DIR}/${name}"
    DST="${STALE_DIR}/${TIMESTAMP}/${name}"
    SIZE=$(du -sm "$SRC" 2>/dev/null | awk '{print $1}')
    if mv "$SRC" "$DST" 2>/dev/null; then
        MOVED=$((MOVED + 1)); FREED=$((FREED + SIZE))
        echo "  Moved: ${name} (${SIZE}MB)"
    else
        echo "  FAILED: ${name}"
    fi
done

echo ""
echo "Moved ${MOVED} directories to ${STALE_DIR}/${TIMESTAMP} -- freed ~${FREED} MB"
echo "To permanently delete: rm -rf ${STALE_DIR}/${TIMESTAMP}/"
