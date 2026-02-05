#!/usr/bin/env bash
# Shared helper functions for CI workflows.

# Get a boolean value from a TOML config file.
# Usage: get_toml_bool <config_file> <section> <key>
# Returns the value of the key in the given section, or empty string if not found.
get_toml_bool() {
  local config_file="$1"
  local section="$2"
  local key="$3"
  awk -v section="$section" -v key="$key" '
    $0 ~ "^\\[" section "\\]" { in_section=1; next }
    /^\[/ { if (in_section) { exit } }
    in_section && $1 == key { print $3; exit }
  ' "${config_file}"
}

# Check if the solx-main compiler is required based on test config.
# Usage: check_main_required <config_file>
# Outputs: "true" or "false"
check_main_required() {
  local config_file="$1"
  local needs_main=false

  if [ -f "${config_file}" ] && grep -qE "^\[compilers\.solx-main\]" "${config_file}"; then
    local disabled
    disabled=$(get_toml_bool "${config_file}" "compilers.solx-main" "disabled")
    if [ -z "${disabled}" ] || [ "${disabled}" = "false" ]; then
      needs_main=true
    fi
  fi
  echo "${needs_main}"
}

# Get the LLVM submodule SHA for a given directory.
# Usage: get_llvm_sha <repo_dir>
get_llvm_sha() {
  git -C "$1" submodule status solx-llvm | awk '{print $1}' | tr -d ' +-'
}

# Compare LLVM submodule SHAs between PR and main.
# Usage: compare_llvm_shas <pr_dir> <main_dir>
# Outputs: Sets PR_LLVM_SHA and MAIN_LLVM_SHA, warns on mismatch.
compare_llvm_shas() {
  local pr_dir="$1"
  local main_dir="$2"

  PR_LLVM_SHA=$(get_llvm_sha "${pr_dir}")
  MAIN_LLVM_SHA=$(get_llvm_sha "${main_dir}")

  echo "pr=${PR_LLVM_SHA}"
  echo "main=${MAIN_LLVM_SHA}"

  if [ "${PR_LLVM_SHA}" != "${MAIN_LLVM_SHA}" ]; then
    echo "::warning::LLVM submodule mismatch (PR=${PR_LLVM_SHA}, main=${MAIN_LLVM_SHA})."
  fi
}
