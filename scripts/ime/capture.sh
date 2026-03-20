#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 6 ]]; then
  echo "usage: $0 <output_file> <window_id> <x> <y> <w> <h>" >&2
  exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
helper="${script_dir}/capture_helper.swift"

output_file="$1"
window_id="$2"
x="$3"
y="$4"
w="$5"
h="$6"

mkdir -p "$(dirname "$output_file")"
if [[ ! -f "${helper}" ]]; then
  echo "missing capture helper: ${helper}" >&2
  exit 1
fi

swift -Xfrontend -disable-availability-checking "${helper}" \
  "$output_file" "$window_id" "$x" "$y" "$w" "$h"
