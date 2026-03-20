#!/usr/bin/env bash

showcase_mode_from_ps_output() {
  local ps_output="${1:-}"
  if [[ "${ps_output}" == *'FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY=1'* ]]; then
    printf 'plain\n'
  elif [[ "${ps_output}" == *'FLOEM_SHOWCASE_DEBUG_IME_LAB_ONLY=1'* ]]; then
    printf 'ime\n'
  else
    printf 'regular\n'
  fi
}

showcase_mode_for_pid() {
  local pid="$1"
  local ps_output
  ps_output="$(ps eww -p "${pid}" 2>/dev/null || true)"
  if [[ -z "${ps_output}" ]]; then
    return 1
  fi
  showcase_mode_from_ps_output "${ps_output}"
}

pid_matches_requested_group() {
  local requested_group="$1"
  local actual_mode="$2"
  case "${requested_group}" in
    plain)
      [[ "${actual_mode}" == "plain" ]]
      ;;
    normal)
      [[ "${actual_mode}" == "ime" ]]
      ;;
    all)
      [[ "${actual_mode}" == "plain" || "${actual_mode}" == "ime" || "${actual_mode}" == "regular" ]]
      ;;
    *)
      return 1
      ;;
  esac
}
