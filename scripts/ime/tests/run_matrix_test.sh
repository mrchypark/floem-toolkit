#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../../.." && pwd)"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "expected output to contain: ${needle}" >&2
    echo "actual output:" >&2
    printf '%s\n' "${haystack}" >&2
    exit 1
  fi
}

output="$(cd "${REPO_ROOT}" && bash scripts/ime/run_matrix.sh --dry-run --reuse-running 2>&1)"
assert_contains "${output}" "[dry-run] reuse running showcase mode=plain"
assert_contains "${output}" "[dry-run] reuse running showcase mode=normal"
assert_contains "${output}" "script=${REPO_ROOT}/scripts/ime/run_plain_input_lab.lua params=${REPO_ROOT} all embedded "
assert_contains "${output}" "script=${REPO_ROOT}/scripts/ime/run_showcase_ime_lab.lua params=${REPO_ROOT} all embedded "

plain_count="$(printf '%s\n' "${output}" | grep -c 'run_plain_input_lab.lua')"
showcase_count="$(printf '%s\n' "${output}" | grep -c 'run_showcase_ime_lab.lua')"

if [[ "${plain_count}" -ne 1 ]]; then
  echo "expected one plain_input_lab invocation, got ${plain_count}" >&2
  exit 1
fi

if [[ "${showcase_count}" -ne 1 ]]; then
  echo "expected one showcase_ime_lab invocation, got ${showcase_count}" >&2
  exit 1
fi
