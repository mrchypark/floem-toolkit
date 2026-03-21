#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../../.." && pwd)"

TMP_DIR="$(mktemp -d)"
STUB_BIN="${TMP_DIR}/bin"
STUB_STATE="${TMP_DIR}/state"
ARTIFACTS_ROOT="${TMP_DIR}/artifacts"
mkdir -p "${STUB_BIN}" "${STUB_STATE}" "${ARTIFACTS_ROOT}"

cleanup() {
  if [[ -f "${STUB_STATE}/pids" ]]; then
    while IFS= read -r pid; do
      [[ -n "${pid}" ]] || continue
      kill "${pid}" >/dev/null 2>&1 || true
    done <"${STUB_STATE}/pids"
  fi
  rm -rf "${TMP_DIR}"
}

trap cleanup EXIT

cat >"${STUB_BIN}/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
exit 0
EOF

cat >"${STUB_BIN}/swift" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "/dev/stdin" ]]; then
  if [[ -n "${2:-}" ]] && kill -0 "${2}" >/dev/null 2>&1; then
    printf '1|%s|10|10|800|632\n' "${2}"
    exit 0
  fi
  exit 1
fi

exit 0
EOF

cat >"${STUB_BIN}/pgrep" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

state_file="${HARNESS_TEST_STATE_DIR}/pids"
[[ -f "${state_file}" ]] || exit 1

while IFS= read -r pid; do
  [[ -n "${pid}" ]] || continue
  if kill -0 "${pid}" >/dev/null 2>&1; then
    printf '%s\n' "${pid}"
  fi
done <"${state_file}"
EOF

cat >"${STUB_BIN}/hs" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

joined="$*"
params_file="$(printf '%s\n' "${joined}" | sed -n 's/.*IME_PARAMS_FILE=\[\[\(.*\.params\.lua\)\]\].*/\1/p')"
basename="$(basename "${params_file}")"

extract_value() {
  local key="$1"
  sed -n "s/.*${key} = \\[\\[\\(.*\\)\\]\\].*/\\1/p" "${params_file}"
}

write_plain_state_log() {
  local file="$1"
  cat >"${file}" <<'LOG'
stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("안") commit_pending=false
showcase::plain_input_lab_source focused=false value="앞"
showcase::plain_input_lab_target focused=true value=""
showcase::plain_input_lab_source ime_preedit text="안"
showcase::plain_input_lab_source focused=false value="앞안"
showcase::plain_input_lab_target focused=true value=""
LOG
}

if [[ "${basename}" == launch_*.params.lua ]]; then
  pid_file="$(extract_value pid_file)"
  app_log_file="$(extract_value app_log_file)"
  env FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY=1 sleep 300 &
  pid=$!
  printf '%s\n' "${pid}" >>"${HARNESS_TEST_STATE_DIR}/pids"
  printf '%s\n' "${pid}" >"${pid_file}"
  write_plain_state_log "${app_log_file}"
  printf '%s\n' "${pid}"
  exit 0
fi

if [[ "${basename}" == plain_input_lab.params.lua ]]; then
  state_log_file="$(extract_value state_log_file)"
  write_plain_state_log "${state_log_file}"
  exit 0
fi

exit 0
EOF

chmod +x "${STUB_BIN}/cargo" "${STUB_BIN}/swift" "${STUB_BIN}/pgrep" "${STUB_BIN}/hs"

PATH="${STUB_BIN}:${PATH}" \
HARNESS_TEST_STATE_DIR="${STUB_STATE}" \
IME_ARTIFACTS_ROOT="${ARTIFACTS_ROOT}" \
IME_AUTO_SWITCH_INPUT_SOURCE=0 \
bash "${REPO_ROOT}/scripts/ime/run_matrix.sh" --group plain

state_file="${STUB_STATE}/pids"
if [[ ! -f "${state_file}" ]]; then
  echo "expected harness test to record launched pids" >&2
  exit 1
fi

while IFS= read -r pid; do
  [[ -n "${pid}" ]] || continue
  if kill -0 "${pid}" >/dev/null 2>&1; then
    echo "expected harness cleanup to stop pid ${pid}" >&2
    exit 1
  fi
done <"${state_file}"
