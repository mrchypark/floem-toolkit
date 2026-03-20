#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
source "${SCRIPT_DIR}/run_matrix_lib.sh"

ARTIFACTS_ROOT="${IME_ARTIFACTS_ROOT:-${REPO_ROOT}/artifacts/ime}"
HS_TIMEOUT="${IME_HS_TIMEOUT:-45}"
WINDOW_WAIT_SEC="${IME_WINDOW_WAIT_SEC:-45}"
DEFAULT_WINDOW_W="${IME_WINDOW_W:-800}"
DEFAULT_WINDOW_H="${IME_WINDOW_H:-632}"
INPUT_SOURCE_HELPER="${SCRIPT_DIR}/input_source_helper.swift"
TIMESTAMP="$(date +"%Y%m%d-%H%M%S")"
LOG_ROOT="${ARTIFACTS_ROOT}/_logs/${TIMESTAMP}"
INPUT_SOURCE_STATE_FILE="${LOG_ROOT}/input_source_previous.txt"
AUTO_SWITCH_INPUT_SOURCE="${IME_AUTO_SWITCH_INPUT_SOURCE:-1}"

DRY_RUN=0
REUSE_RUNNING=0
GROUP="all"

managed_pid=""
restore_input_source=0

usage() {
  cat <<'EOF'
usage: bash scripts/ime/run_matrix.sh [--dry-run] [--reuse-running] [--group plain|normal|all]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      ;;
    --reuse-running)
      REUSE_RUNNING=1
      ;;
    --group)
      shift
      GROUP="${1:-}"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if [[ "${GROUP}" != "plain" && "${GROUP}" != "normal" && "${GROUP}" != "all" ]]; then
  echo "invalid group: ${GROUP}" >&2
  exit 2
fi

mkdir -p "${LOG_ROOT}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

for cmd in bash swift ps; do
  require_cmd "${cmd}"
done

if [[ "${DRY_RUN}" -eq 0 ]]; then
  require_cmd hs
fi

log() {
  printf '[ime-matrix] %s\n' "$*"
}

ensure_korean_input_source() {
  if [[ "${DRY_RUN}" -eq 1 || "${AUTO_SWITCH_INPUT_SOURCE}" -ne 1 ]]; then
    return 0
  fi

  if [[ ! -f "${INPUT_SOURCE_HELPER}" ]]; then
    echo "missing input source helper: ${INPUT_SOURCE_HELPER}" >&2
    exit 1
  fi

  log "switching input source to Korean 2-set"
  IME_INPUT_SOURCE_STATE_FILE="${INPUT_SOURCE_STATE_FILE}" \
    swift "${INPUT_SOURCE_HELPER}" set-korean >/dev/null
  restore_input_source=1
}

cleanup() {
  if [[ -n "${managed_pid}" ]] && kill -0 "${managed_pid}" >/dev/null 2>&1; then
    log "stopping managed showcase pid=${managed_pid}"
    kill "${managed_pid}" >/dev/null 2>&1 || true
    wait "${managed_pid}" 2>/dev/null || true
  fi
  if [[ "${restore_input_source}" -eq 1 ]]; then
    IME_INPUT_SOURCE_STATE_FILE="${INPUT_SOURCE_STATE_FILE}" \
      swift "${INPUT_SOURCE_HELPER}" restore >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT

window_snapshot_for_pid() {
  local pid="$1"
  swift /dev/stdin "${pid}" <<'SWIFT'
import CoreGraphics
import Foundation

let targetPid = pid_t(CommandLine.arguments[1]) ?? 0
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
let windows = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] ?? []

func intValue(_ any: Any?) -> Int? {
    if let value = any as? Int { return value }
    if let value = any as? NSNumber { return value.intValue }
    if let value = any as? Double { return Int(value.rounded()) }
    return nil
}

for info in windows {
    guard intValue(info[kCGWindowOwnerPID as String]) == Int(targetPid) else { continue }
    guard intValue(info[kCGWindowLayer as String]) == 0 else { continue }
    guard let bounds = info[kCGWindowBounds as String] as? [String: Any] else { continue }
    let x = intValue(bounds["X"]) ?? 0
    let y = intValue(bounds["Y"]) ?? 0
    let w = intValue(bounds["Width"]) ?? 0
    let h = intValue(bounds["Height"]) ?? 0
    guard w > 0, h > 0 else { continue }
    let id = intValue(info[kCGWindowNumber as String]) ?? 0
    print("\(id)|\(Int(targetPid))|\(x)|\(y)|\(w)|\(h)")
    exit(0)
}

exit(1)
SWIFT
}

parse_snapshot_into() {
  local snapshot="$1"
  local prefix="$2"
  local id pid x y w h
  IFS='|' read -r id pid x y w h <<<"${snapshot}"
  printf -v "${prefix}_WINDOW_ID" '%s' "${id}"
  printf -v "${prefix}_OWNER_PID" '%s' "${pid}"
  printf -v "${prefix}_X" '%s' "${x}"
  printf -v "${prefix}_Y" '%s' "${y}"
  printf -v "${prefix}_W" '%s' "${w}"
  printf -v "${prefix}_H" '%s' "${h}"
}

pid_matches_requested_group_for_pid() {
  local pid="$1"
  local requested_group="$2"
  local actual_mode
  actual_mode="$(showcase_mode_for_pid "${pid}" 2>/dev/null || true)"
  [[ -n "${actual_mode}" ]] || return 1
  pid_matches_requested_group "${requested_group}" "${actual_mode}"
}

bring_pid_to_front() {
  local pid="$1"
  swift /dev/stdin "${pid}" <<'SWIFT' >/dev/null
import AppKit
import Foundation

let pid = pid_t(CommandLine.arguments[1]) ?? 0
guard let app = NSRunningApplication(processIdentifier: pid) else {
    fputs("no running app for pid \(pid)\n", stderr)
    exit(1)
}

if !app.activate(options: [.activateAllWindows]) {
    fputs("failed to activate pid \(pid)\n", stderr)
    exit(1)
}
SWIFT
}

ensure_korean_input_source

cleanup_stale_harness_showcases() {
  local requested_group="$1"
  local pid actual_mode
  while IFS= read -r pid; do
    [[ -n "${pid}" ]] || continue
    actual_mode="$(showcase_mode_for_pid "${pid}" 2>/dev/null || true)"
    [[ -n "${actual_mode}" ]] || continue
    if pid_matches_requested_group "${requested_group}" "${actual_mode}"; then
      printf '[ime-matrix] stopping stale harness showcase pid=%s mode=%s\n' "${pid}" "${actual_mode}" >&2
      kill "${pid}" >/dev/null 2>&1 || true
      wait "${pid}" 2>/dev/null || true
    fi
  done < <(pgrep -x floem-showcase || true)
}

frontmost_showcase_pid() {
  swift /dev/stdin <<'SWIFT'
import AppKit
import Foundation

guard let app = NSWorkspace.shared.frontmostApplication else {
    exit(1)
}

let bundleID = app.bundleIdentifier ?? ""
let localizedName = app.localizedName ?? ""
if bundleID == "floem-showcase" || localizedName == "floem-showcase" {
    print(app.processIdentifier)
    exit(0)
}

exit(1)
SWIFT
}

wait_for_new_pid() {
  local existing_csv="$1"
  local deadline=$((SECONDS + WINDOW_WAIT_SEC))
  while (( SECONDS < deadline )); do
    while IFS= read -r pid; do
      [[ -z "${pid}" ]] && continue
      if [[ ",${existing_csv}," != *",${pid},"* ]]; then
        printf '%s\n' "${pid}"
        return 0
      fi
    done < <(pgrep -x floem-showcase || true)
    sleep 0.2
  done
  return 1
}

wait_for_window_snapshot() {
  local pid="$1"
  local deadline=$((SECONDS + WINDOW_WAIT_SEC))
  while (( SECONDS < deadline )); do
    if snapshot="$(window_snapshot_for_pid "${pid}" 2>/dev/null)"; then
      printf '%s\n' "${snapshot}"
      return 0
    fi
    sleep 0.2
  done
  return 1
}

launch_showcase() {
  local mode="$1"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "[dry-run] launch showcase mode=${mode}"
    printf '0|0|0|0|%s|%s\n' "${DEFAULT_WINDOW_W}" "${DEFAULT_WINDOW_H}"
    return 0
  fi

  cargo build -p floem-showcase >/dev/null

  cleanup_stale_harness_showcases "${mode}"

  local existing_csv
  existing_csv="$(pgrep -x floem-showcase | paste -sd, - || true)"
  local pid_file="${LOG_ROOT}/launch_${mode}.pid"
  local app_log_file="${LOG_ROOT}/launch_${mode}.state.log"
  local params_file="${LOG_ROOT}/launch_${mode}.params.lua"
  cat >"${params_file}" <<EOF
return {
  repo_root = [[${REPO_ROOT}]],
  mode = [[${mode}]],
  pid_file = [[${pid_file}]],
  app_log_file = [[${app_log_file}]],
}
EOF
  hs -A -q -c "IME_PARAMS_FILE=[[${params_file}]]; dofile([[${REPO_ROOT}/scripts/ime/launch_showcase.lua]])" >/dev/null

  local pid
  if [[ -s "${pid_file}" ]]; then
    pid="$(cat "${pid_file}")"
  else
    pid="$(wait_for_new_pid "${existing_csv}")"
  fi
  [[ -n "${pid:-}" ]] || {
    echo "failed to detect launched floem-showcase pid for mode=${mode}" >&2
    exit 1
  }
  managed_pid="${pid}"
  local snapshot
  snapshot="$(wait_for_window_snapshot "${pid}")" || {
    echo "failed to detect showcase window for pid=${pid}" >&2
    exit 1
  }
  printf '%s\n' "${snapshot}"
}

select_reuse_snapshot() {
  local requested_group="$1"
  local snapshot=""
  local preferred_pid=""
  local actual_mode=""
  local best_snapshot=""
  local best_rank="-1"
  local best_area="-1"
  if [[ "${DRY_RUN}" -eq 0 ]]; then
    preferred_pid="$(frontmost_showcase_pid || true)"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    snapshot="0|0|0|0|${DEFAULT_WINDOW_W}|${DEFAULT_WINDOW_H}"
    printf '%s\n' "${snapshot}"
    return 0
  fi

  if [[ -n "${preferred_pid}" ]] \
    && actual_mode="$(showcase_mode_for_pid "${preferred_pid}" 2>/dev/null || true)" \
    && pid_matches_requested_group "${requested_group}" "${actual_mode}" \
    && snapshot="$(window_snapshot_for_pid "${preferred_pid}" 2>/dev/null)"; then
    printf '%s\n' "${snapshot}"
    return 0
  fi

  while IFS= read -r pid; do
    [[ -n "${pid}" ]] || continue
    actual_mode="$(showcase_mode_for_pid "${pid}" 2>/dev/null || true)"
    [[ -n "${actual_mode}" ]] || continue
    if ! pid_matches_requested_group "${requested_group}" "${actual_mode}"; then
      continue
    fi
    if snapshot="$(window_snapshot_for_pid "${pid}" 2>/dev/null)"; then
      local window_id owner_pid x y w h
      IFS='|' read -r window_id owner_pid x y w h <<<"${snapshot}"
      local rank=0
      if [[ "${x}" =~ ^-?[0-9]+$ && "${y}" =~ ^-?[0-9]+$ ]] && (( x >= 0 && y >= 0 )); then
        rank=1
      fi
      local area=$(( w * h ))
      if (( rank > best_rank || (rank == best_rank && area > best_area) )); then
        best_snapshot="${snapshot}"
        best_rank="${rank}"
        best_area="${area}"
      fi
    fi
  done < <(pgrep -x floem-showcase || true)

  if [[ -n "${best_snapshot}" ]]; then
    printf '%s\n' "${best_snapshot}"
    return 0
  fi

  if [[ "${requested_group}" == "normal" ]]; then
    echo "no running IME-lab-only floem-showcase window found for --reuse-running" >&2
  elif [[ "${requested_group}" == "plain" ]]; then
    echo "no running plain-lab-only floem-showcase window found for --reuse-running" >&2
  else
    echo "no running floem-showcase window found for --reuse-running" >&2
  fi
  exit 1
}

hs_invoke() {
  local log_file="$1"
  local script_path="$2"
  shift 2
  local repo_root="$1"
  local case_name="$2"
  local surface_mode="$3"
  local window_id="$4"
  local window_x="$5"
  local window_y="$6"
  local window_w="$7"
  local window_h="$8"
  local state_log_file="$9"
  local params_file="${log_file%.log}.params.lua"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "[dry-run] script=${script_path} params=${repo_root} ${case_name} ${surface_mode} ${window_id} ${window_x} ${window_y} ${window_w} ${window_h} ${state_log_file}"
    return 0
  fi
  : >"${log_file}"
  cat >"${params_file}" <<EOF
return {
  repo_root = [[${repo_root}]],
  case = [[${case_name}]],
  surface_mode = [[${surface_mode}]],
  scenario_log_file = [[${log_file}]],
  window_id = [[${window_id}]],
  window_x = tonumber([[${window_x}]]),
  window_y = tonumber([[${window_y}]]),
  window_w = tonumber([[${window_w}]]),
  window_h = tonumber([[${window_h}]]),
  state_log_file = [[${state_log_file}]],
}
EOF
  local hs_status=0
  if IME_SCENARIO_LOG_FILE="${log_file}" \
    hs -A -q -t "${HS_TIMEOUT}" -c "IME_PARAMS_FILE=[[${params_file}]]; dofile([[${script_path}]])" >>"${log_file}" 2>&1; then
    hs_status=0
  else
    hs_status=$?
  fi

  if (( hs_status != 0 )); then
    return "${hs_status}"
  fi
}

require_log_pattern() {
  local file="$1"
  local pattern="$2"
  local description="$3"
  if [[ ! -f "${file}" ]]; then
    echo "missing log file for validation: ${file}" >&2
    exit 1
  fi
  if ! grep -Fq "${pattern}" "${file}"; then
    echo "missing expected log pattern (${description}): ${pattern}" >&2
    exit 1
  fi
}

last_match_line() {
  local file="$1"
  local pattern="$2"
  awk -v pattern="${pattern}" 'index($0, pattern) { line = NR } END { if (line) { print line } }' "${file}"
}

require_log_pattern_order() {
  local file="$1"
  local first_pattern="$2"
  local second_pattern="$3"
  local description="$4"
  local first_line second_line
  first_line="$(last_match_line "${file}" "${first_pattern}")"
  second_line="$(last_match_line "${file}" "${second_pattern}")"
  if [[ -z "${first_line}" || -z "${second_line}" ]]; then
    echo "missing expected ordered log patterns (${description})" >&2
    echo "  first:  ${first_pattern}" >&2
    echo "  second: ${second_pattern}" >&2
    exit 1
  fi
  if (( first_line >= second_line )); then
    echo "unexpected log order (${description}): ${first_pattern} !< ${second_pattern}" >&2
    exit 1
  fi
}

state_log_for_pid() {
  local pid="$1"
  local fallback_path="$2"
  local ps_output state_log

  if [[ -f "${fallback_path}" ]]; then
    printf '%s\n' "${fallback_path}"
    return 0
  fi

  ps_output="$(ps eww -p "${pid}" 2>/dev/null || true)"
  if [[ -z "${ps_output}" ]]; then
    return 1
  fi

  state_log="$(printf '%s\n' "${ps_output}" | sed -n 's/.*FLOEM_SHOWCASE_IME_STATE_LOG_FILE=\([^ ]*\).*/\1/p' | tail -n1)"
  if [[ -n "${state_log}" && -f "${state_log}" ]]; then
    printf '%s\n' "${state_log}"
    return 0
  fi

  return 1
}

validate_plain_group() {
  local state_log=""
  state_log="$(state_log_for_pid "${PLAIN_OWNER_PID}" "${LOG_ROOT}/launch_plain.state.log" || true)"
  if [[ -z "${state_log}" ]]; then
    echo "missing plain state log for pid=${PLAIN_OWNER_PID}; semantic validation requires a harness-owned or explicitly instrumented verification window" >&2
    exit 1
  fi
  require_log_pattern \
    "${state_log}" \
    'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("안") commit_pending=false' \
    "plain case_a first Hangul preedit"
  require_log_pattern_order \
    "${state_log}" \
    'showcase::plain_input_lab_source focused=false value="앞"' \
    'showcase::plain_input_lab_target focused=true value=""' \
    "plain composition handoff keeps target empty"
  require_log_pattern \
    "${state_log}" \
    'showcase::plain_input_lab_source ime_preedit text="안"' \
    "plain case_d re-entry starts a new Hangul preedit"
  require_log_pattern \
    "${state_log}" \
    'showcase::plain_input_lab_source focused=false value="앞안"' \
    "plain re-entry recommit stays on source"
  require_log_pattern \
    "${state_log}" \
    'showcase::plain_input_lab_target focused=true value=""' \
    "plain re-entry handoff keeps target empty after recommit"
}

validate_normal_group() {
  local state_log=""
  state_log="$(state_log_for_pid "${NORMAL_OWNER_PID}" "${LOG_ROOT}/launch_normal.state.log" || true)"
  if [[ -z "${state_log}" ]]; then
    echo "missing normal state log for pid=${NORMAL_OWNER_PID}; semantic validation requires a harness-owned or explicitly instrumented verification window" >&2
    exit 1
  fi
  require_log_pattern \
    "${state_log}" \
    'stage=ime_preedit:accept focused=true cursor=0 buffer="" preedit=Some("안") commit_pending=false' \
    "normal case_a first Hangul preedit"
  require_log_pattern_order \
    "${state_log}" \
    'ime-source focused=false value="앞"' \
    'ime-target focused=true value=""' \
    "normal composition handoff keeps target empty"
  require_log_pattern \
    "${state_log}" \
    'ime-source ime_preedit text="안"' \
    "normal case_d re-entry starts a new Hangul preedit"
  require_log_pattern \
    "${state_log}" \
    'ime-source focused=false value="앞안"' \
    "normal re-entry recommit stays on source"
  require_log_pattern \
    "${state_log}" \
    'ime-target focused=true value=""' \
    "normal re-entry handoff keeps target empty after recommit"
}

run_plain_group() {
  local snapshot="$1"
  parse_snapshot_into "${snapshot}" PLAIN
  local surface_mode="embedded"

  if [[ "${DRY_RUN}" -eq 0 ]] && pid_matches_requested_group_for_pid "${PLAIN_OWNER_PID}" plain; then
    surface_mode="plain_only"
  fi

  if [[ "${REUSE_RUNNING}" -eq 1 ]]; then
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      log "[dry-run] reuse running showcase mode=plain"
    else
      log "reuse running showcase mode=plain pid=${PLAIN_OWNER_PID} window=${PLAIN_WINDOW_ID}"
    fi
  fi

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    bring_pid_to_front "${PLAIN_OWNER_PID}"
    sleep 0.2
  fi

  hs_invoke \
    "${LOG_ROOT}/plain_input_lab.log" \
    "${REPO_ROOT}/scripts/ime/run_plain_input_lab.lua" \
    "${REPO_ROOT}" \
    "all" \
    "${surface_mode}" \
    "${PLAIN_WINDOW_ID}" \
    "${PLAIN_X}" \
    "${PLAIN_Y}" \
    "${PLAIN_W}" \
    "${PLAIN_H}" \
    "${LOG_ROOT}/launch_plain.state.log"

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    validate_plain_group
  fi
}

run_normal_group() {
  local snapshot="$1"
  parse_snapshot_into "${snapshot}" NORMAL
  local surface_mode="embedded"

  if [[ "${DRY_RUN}" -eq 0 ]] && pid_matches_requested_group_for_pid "${NORMAL_OWNER_PID}" normal; then
    surface_mode="ime_only"
  fi

  if [[ "${REUSE_RUNNING}" -eq 1 ]]; then
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      log "[dry-run] reuse running showcase mode=normal"
    else
      log "reuse running showcase mode=normal pid=${NORMAL_OWNER_PID} window=${NORMAL_WINDOW_ID}"
    fi
  fi

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    bring_pid_to_front "${NORMAL_OWNER_PID}"
    sleep 0.2
  fi

  hs_invoke \
    "${LOG_ROOT}/showcase_ime_lab.log" \
    "${REPO_ROOT}/scripts/ime/run_showcase_ime_lab.lua" \
    "${REPO_ROOT}" \
    "all" \
    "${surface_mode}" \
    "${NORMAL_WINDOW_ID}" \
    "${NORMAL_X}" \
    "${NORMAL_Y}" \
    "${NORMAL_W}" \
    "${NORMAL_H}" \
    "${LOG_ROOT}/launch_normal.state.log"

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    validate_normal_group
  fi
}

case "${GROUP}" in
  plain)
    if [[ "${REUSE_RUNNING}" -eq 1 ]]; then
      plain_snapshot="$(select_reuse_snapshot plain)"
    else
      plain_snapshot="$(launch_showcase plain)"
    fi
    run_plain_group "${plain_snapshot}"
    ;;
  normal)
    if [[ "${REUSE_RUNNING}" -eq 1 ]]; then
      normal_snapshot="$(select_reuse_snapshot normal)"
    else
      normal_snapshot="$(launch_showcase normal)"
    fi
    run_normal_group "${normal_snapshot}"
    ;;
  all)
    if [[ "${REUSE_RUNNING}" -eq 1 ]]; then
      plain_snapshot="$(select_reuse_snapshot plain)"
      run_plain_group "${plain_snapshot}"
      normal_snapshot="$(select_reuse_snapshot normal)"
      run_normal_group "${normal_snapshot}"
    else
      plain_snapshot="$(launch_showcase plain)"
      run_plain_group "${plain_snapshot}"
      cleanup
      managed_pid=""
      normal_snapshot="$(launch_showcase normal)"
      run_normal_group "${normal_snapshot}"
    fi
    ;;
esac
