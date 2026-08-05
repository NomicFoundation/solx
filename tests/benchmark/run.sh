#!/usr/bin/env bash
#
# Compile-time benchmark driver: times `<solx> --standard-json <fixture>` for
# each selected fixture across one or more solx binaries, using hyperfine. Raw
# hyperfine JSON lands in --out; report.py renders the markdown report from it.
#
# Usage:
#   run.sh --bin <name>=<path> [--bin <name>=<path> ...] --fixtures <dir> \
#          [--variant <name>]... [--out <dir>] [--runs N] [--warmup N]
#
# --fixtures points at a corpus laid out as <scenario>/<variant>.json. Each
# --variant selects the <scenario>/<variant>.json of every scenario; the
# default is the EVMLA (legacy) pipeline. The compiler pipeline is encoded in
# the standard JSON itself, so switching variants needs no solx flag.
#
# The first --bin is the candidate; report.py treats the second as the
# baseline for the vs-baseline column. Example:
#   run.sh --bin pr=./target/release/solx --bin main=./temp-solx-main/target/release/solx \
#          --fixtures corpus/solx-corpus --variant solx-via-ir-dwarf

set -euo pipefail
shopt -s nullglob  # a variant that matches no scenario expands to nothing

FIXTURES_DIR=""
OUT_DIR="benchmark-out"
RUNS=5
WARMUP=2
BIN_NAMES=()
BIN_PATHS=()
VARIANTS=()

while [ $# -gt 0 ]; do
  case "$1" in
    --bin)
      BIN_NAMES+=("${2%%=*}")
      BIN_PATHS+=("${2#*=}")
      shift 2 ;;
    --fixtures) FIXTURES_DIR="$2"; shift 2 ;;
    --variant) VARIANTS+=("$2"); shift 2 ;;
    --out) OUT_DIR="$2"; shift 2 ;;
    --runs) RUNS="$2"; shift 2 ;;
    --warmup) WARMUP="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

[ "${#BIN_NAMES[@]}" -ge 1 ] || { echo "at least one --bin required" >&2; exit 1; }
[ -n "${FIXTURES_DIR}" ] || { echo "--fixtures <dir> required" >&2; exit 1; }
[ "${#VARIANTS[@]}" -ge 1 ] || VARIANTS=(solx-legacy-dwarf)

# Resolve the requested variants to concrete fixtures up front so a typo or an
# empty corpus fails here rather than silently benchmarking nothing.
FIXTURES=()
for variant in "${VARIANTS[@]}"; do
  FIXTURES+=("${FIXTURES_DIR}"/*/"${variant}.json")
done
[ "${#FIXTURES[@]}" -ge 1 ] || {
  echo "no fixtures found under ${FIXTURES_DIR} for variant(s): ${VARIANTS[*]}" >&2
  exit 1
}

mkdir -p "${OUT_DIR}"

# Record binary identities alongside the measurements.
: > "${OUT_DIR}/versions.txt"
for i in "${!BIN_NAMES[@]}"; do
  echo "${BIN_NAMES[$i]}: $("${BIN_PATHS[$i]}" --version | head -1)" >> "${OUT_DIR}/versions.txt"
done
cat "${OUT_DIR}/versions.txt"

# The standard-JSON protocol reports compilation failure inside the JSON
# (exit code stays 0), so an untimed validation run per binary must gate the
# benchmark: hyperfine would otherwise happily time failing compiles.
validate() {
  "$1" --standard-json "$2" 2>/dev/null | python3 -c '
import json, sys
out = json.load(sys.stdin)
errors = [e for e in out.get("errors") or [] if e.get("severity") == "error"]
for error in errors:
    print(error.get("formattedMessage", error), file=sys.stderr)
assert not errors, f"{len(errors)} compilation error(s)"
assert out.get("contracts"), "no contracts in output"
'
}

for fixture in "${FIXTURES[@]}"; do
  name="$(basename "$(dirname "${fixture}")")--$(basename "${fixture}" .json)"
  args=()
  for i in "${!BIN_NAMES[@]}"; do
    echo "validating ${name} with ${BIN_NAMES[$i]}"
    validate "${BIN_PATHS[$i]}" "${fixture}"
    args+=(--command-name "${BIN_NAMES[$i]}" "${BIN_PATHS[$i]} --standard-json ${fixture} > /dev/null")
  done
  hyperfine \
    --warmup "${WARMUP}" \
    --runs "${RUNS}" \
    --export-json "${OUT_DIR}/${name}.json" \
    "${args[@]}"
done
