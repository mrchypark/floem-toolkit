#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../../.." && pwd)"

source "${REPO_ROOT}/scripts/ime/run_matrix_lib.sh"

assert_eq() {
  local expected="$1"
  local actual="$2"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "expected '${expected}', got '${actual}'" >&2
    exit 1
  fi
}

assert_true() {
  if ! "$@"; then
    echo "expected command to succeed: $*" >&2
    exit 1
  fi
}

assert_false() {
  if "$@"; then
    echo "expected command to fail: $*" >&2
    exit 1
  fi
}

assert_eq plain "$(showcase_mode_from_ps_output '... FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY=1 ...')"
assert_eq ime "$(showcase_mode_from_ps_output '... FLOEM_SHOWCASE_DEBUG_IME_LAB_ONLY=1 ...')"
assert_eq regular "$(showcase_mode_from_ps_output '... no special flags ...')"

assert_true pid_matches_requested_group plain plain
assert_true pid_matches_requested_group normal ime
assert_true pid_matches_requested_group all regular
assert_false pid_matches_requested_group normal plain
assert_false pid_matches_requested_group plain ime
