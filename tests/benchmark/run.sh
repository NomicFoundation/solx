#!/usr/bin/env bash
#
# Compile-time benchmark driver: times `<solx> --standard-json <fixture>` for
# each fixture in the fixtures directory across one or more solx binaries,
# using hyperfine. Raw hyperfine JSON lands in --out; report.py renders the
# markdown report from it.
#
# Usage:
#   run.sh --bin <name>=<path> [--bin <name>=<path> ...] \
#          [--fixtures <dir>] [--out <dir>] [--runs N] [--warmup N]
#
# The first --bin is the candidate; report.py treats the second as the
# baseline for the Δ column. Example:
#   run.sh --bin pr=./target/release/solx --bin main=./temp-solx-main/target/release/solx

set -euo pipefail

FIXTURES_DIR="$(dirname "$0")/fixtures"
OUT_DIR="benchmark-out"
RUNS=5
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
    --warmup) WARMUP="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

[ "${#BIN_NAMES[@]}" -ge 1 ] || { echo "at least one --bin required" >&2; exit 1; }

mkdir -p "${OUT_DIR}"

# Record binary identities alongside the measurements.
: > "${OUT_DIR}/versions.txt"
for i in "${!BIN_NAMES[@]}"; do
  echo "${BIN_NAMES[$i]}: $("${BIN_PATHS[$i]}" --version | head -1)" >> "${OUT_DIR}/versions.txt"
done
cat "${OUT_DIR}/versions.txt"

for fixture in "${FIXTURES_DIR}"/*.json; do
  name="$(basename "${fixture}" .json)"
  args=()
  for i in "${!BIN_NAMES[@]}"; do
    args+=(--command-name "${BIN_NAMES[$i]}" "${BIN_PATHS[$i]} --standard-json ${fixture} > /dev/null")
  done
  hyperfine \
    --warmup "${WARMUP}" \
    --runs "${RUNS}" \
    --export-json "${OUT_DIR}/${name}.json" \
    "${args[@]}"
done
