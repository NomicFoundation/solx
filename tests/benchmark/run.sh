#!/usr/bin/env bash
#
# Compile-time benchmark driver: times `<slang> --standard-json` over every
# fixture of a scenario in sequence, one hyperfine measurement per scenario,
# across one or more slang binaries. Raw hyperfine JSON lands in --out;
# report.py renders the markdown report from it.
#
# Usage:
#   run.sh --bin <name>=<path> [--bin <name>=<path> ...] \
#          [--fixtures <dir>] [--out <dir>] [--runs N] [--large-runs N] [--warmup N]
#
# The first --bin is the candidate; report.py treats the second as the
# baseline for the Δ column. Example:
#   run.sh --bin pr=./target/release/slang --bin main=./temp-solx-main/target/release/slang

set -euo pipefail

FIXTURES_DIR="$(dirname "$0")/fixtures"
OUT_DIR="benchmark-out"
RUNS=5
LARGE_RUNS=3
WARMUP=2
BIN_NAMES=()
BIN_PATHS=()

while [ $# -gt 0 ]; do
  case "$1" in
    --bin)
      BIN_NAMES+=("${2%%=*}")
      BIN_PATHS+=("${2#*=}")
      shift 2 ;;
    --fixtures) FIXTURES_DIR="$2"; shift 2 ;;
    --out) OUT_DIR="$2"; shift 2 ;;
    --runs) RUNS="$2"; shift 2 ;;
    --large-runs) LARGE_RUNS="$2"; shift 2 ;;
    --warmup) WARMUP="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

[ "${#BIN_NAMES[@]}" -ge 1 ] || { echo "at least one --bin required" >&2; exit 1; }

mkdir -p "${OUT_DIR}"

# Record binary identities and the runner alongside the measurements.
: > "${OUT_DIR}/versions.txt"
for i in "${!BIN_NAMES[@]}"; do
  echo "${BIN_NAMES[$i]}: $("${BIN_PATHS[$i]}" --version | head -1)" >> "${OUT_DIR}/versions.txt"
done
echo "runner: ${RUNNER_NAME:-$(hostname)}" >> "${OUT_DIR}/versions.txt"
if [ -n "${PR_HEAD_SHA:-}" ]; then
  echo "commits: pr ${PR_HEAD_SHA:0:8}${PR_BASE_REF:+ merged into ${PR_BASE_REF}}, main ${MAIN_SHA:0:8}" >> "${OUT_DIR}/versions.txt"
fi
cat "${OUT_DIR}/versions.txt"

# The standard-JSON protocol reports compilation failure inside the JSON
# (exit code stays 0), so an untimed validation run per binary must gate the
# benchmark: hyperfine would otherwise happily time failing compiles.
# Usage: validate <binary> <fixture> [NAME=VALUE ...]
validate() {
  env "${@:3}" "$1" --standard-json "$2" 2>/dev/null | python3 -c '
import json, sys
out = json.load(sys.stdin)
errors = [e for e in out.get("errors") or [] if e.get("severity") == "error"]
for error in errors:
    print(error.get("formattedMessage", error), file=sys.stderr)
assert not errors, f"{len(errors)} compilation error(s)"
assert out.get("contracts"), "no contracts in output"
'
}

# The env vars a scenario compiles with, as NAME=VALUE lines from the
# manifest: standard JSON has no field for them.
fixture_env() {
  python3 -c '
import json, sys
scenario = json.load(open(sys.argv[1]))["scenarios"][sys.argv[2]]
for name, value in scenario.get("compilerEnv", {}).items():
    print(f"{name}={value}")
' "${FIXTURES_DIR}/manifest.json" "$1"
}

# Scenarios over 2 MB in total (swap-vm, graph-horizon, OpenZeppelin, Solady)
# take 5-9 s per compile on the CI runner, so they get --large-runs instead of
# --runs.
LARGE_SCENARIO_BYTES=$((2 * 1024 * 1024))

# Fixtures mirror the hardhat-published corpus layout: <scenario>/<job>.json,
# one file per Hardhat compilation job (manifest.json at the root carries
# provenance and is not a fixture).
for scenario_dir in "${FIXTURES_DIR}"/*/; do
  scenario="$(basename "${scenario_dir}")"
  fixtures=("${scenario_dir}"*.json)
  mapfile -t envs < <(fixture_env "${scenario}")
  runs="${RUNS}"
  [ "$(cat "${fixtures[@]}" | wc -c)" -le "${LARGE_SCENARIO_BYTES}" ] || runs="${LARGE_RUNS}"
  args=()
  for i in "${!BIN_NAMES[@]}"; do
    for fixture in "${fixtures[@]}"; do
      echo "validating ${scenario}/$(basename "${fixture}") with ${BIN_NAMES[$i]}"
      validate "${BIN_PATHS[$i]}" "${fixture}" "${envs[@]}"
    done
    args+=(--command-name "${BIN_NAMES[$i]}" "for fixture in ${fixtures[*]}; do ${BIN_PATHS[$i]} --standard-json \$fixture > /dev/null; done")
  done
  env "${envs[@]}" hyperfine \
    --warmup "${WARMUP}" \
    --runs "${runs}" \
    --export-json "${OUT_DIR}/${scenario}.json" \
    "${args[@]}"
done
