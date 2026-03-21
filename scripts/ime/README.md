# Native IME Verification Scripts

This directory contains the macOS-native verification harness for text-entry IME behavior.

## Prerequisites

- macOS
- `cargo`
- [Hammerspoon](https://www.hammerspoon.org/) + `hs` CLI
- Hammerspoon IPC enabled in `~/.hammerspoon/init.lua`:

```lua
require("hs.ipc")
```

- `swift` (for window bounds lookup fallback)
- Korean 2-set input source, or allow the harness to switch it automatically

## Scripts

- `run_plain_input_lab.lua`
  - Drives the plain primitive verification surface (`FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY=1` mode)
- `run_showcase_ime_lab.lua`
  - Drives the regular showcase IME lab (`floem-ui::Input`)
- `capture.sh`
  - Capture entrypoint that uses the Swift CoreGraphics helper
- `capture_helper.swift`
  - CoreGraphics capture helper used to avoid clearing IME preedit during shell-side screenshots
- `input_source_helper.swift`
  - Saves the current macOS input source, switches to Korean 2-set, and later restores the saved source
- `run_matrix.sh`
  - Orchestrates both labs, runs `case_a/case_b/case_c`, and captures shell-side artifacts

## Usage

Run the whole matrix:

```bash
bash scripts/ime/run_matrix.sh
```

Reuse an already running plain-lab-only window:

```bash
bash scripts/ime/run_matrix.sh --group plain --reuse-running
```

Reuse an already running IME-lab-only window:

```bash
bash scripts/ime/run_matrix.sh --group normal --reuse-running
```

When the reused window was launched by the harness, semantic log validation still runs. If you reuse a manually launched verification window, you must also launch it with `FLOEM_SHOWCASE_IME_STATE_LOG_FILE=...` and `FLOEM_IME_DEBUG=1`; otherwise the matrix exits because it cannot prove IME semantics from screenshots alone.

Reuse dedicated verification windows for just one group:

```bash
bash scripts/ime/run_matrix.sh --group plain --reuse-running
bash scripts/ime/run_matrix.sh --group normal --reuse-running
```

Dry run (no app launch, no typing, no capture):

```bash
bash scripts/ime/run_matrix.sh --dry-run
```

Run only one Lua scenario directly:

```bash
hs -A -t 45 /Users/cypark/.codex/worktrees/cbb8/floem-ui-kit/scripts/ime/run_plain_input_lab.lua -- \
  "$(pwd)" case_b skip_capture plain_only window:1234 160 120 800 632
```

When you invoke the showcase IME lab directly, pass live window bounds so the script targets the current showcase frame:

```bash
hs -A -t 45 /Users/cypark/.codex/worktrees/cbb8/floem-ui-kit/scripts/ime/run_showcase_ime_lab.lua -- \
  "$(pwd)" case_a ime_only window:1234 880 202 800 632
```

Input-source helper:

```bash
swift scripts/ime/input_source_helper.swift set-korean
swift scripts/ime/input_source_helper.swift restore
```

The helper stores the previous source ID under `artifacts/ime/_state/input_source_previous.txt` by default. Set `IME_INPUT_SOURCE_STATE_FILE` to override that path.

The matrix runner switches to Korean 2-set automatically by default and restores the previous source on exit. Set `IME_AUTO_SWITCH_INPUT_SOURCE=0` if you want to keep full manual control.

## Artifact Paths

Default root:

```text
artifacts/ime/
```

Generated outputs:

- `artifacts/ime/plain_input_lab/case_a_first_hangul.png`
- `artifacts/ime/plain_input_lab/case_b_click_handoff.png`
- `artifacts/ime/plain_input_lab/case_c_tab_handoff.png`
- `artifacts/ime/showcase_ime_lab/case_a_first_hangul.png`
- `artifacts/ime/showcase_ime_lab/case_b_click_handoff.png`
- `artifacts/ime/showcase_ime_lab/case_c_tab_handoff.png`
- `artifacts/ime/_logs/<timestamp>/plain_input_lab.log`
- `artifacts/ime/_logs/<timestamp>/showcase_ime_lab.log`

## Environment Flags

- `IME_ARTIFACTS_ROOT` (default: `<repo>/artifacts/ime`)
- `IME_HS_TIMEOUT` (default: `45`)
- `IME_WINDOW_WAIT_SEC` (default: `45`)
- `IME_AUTO_SWITCH_INPUT_SOURCE` (`1` by default)
- `IME_WINDOW_W` / `IME_WINDOW_H` (used by coordinate fallback in Lua helper)
- `IME_CASE` (`case_a`, `case_b`, `case_c`, `all`) remains supported, but direct Hammerspoon runs prefer `_cli.args`
- `IME_SKIP_INTERNAL_CAPTURE=1` remains supported, but direct Hammerspoon runs prefer `_cli.args`

Coordinate overrides for layout drift:

- `IME_PLAIN_RESET_X`, `IME_PLAIN_RESET_Y`
- `IME_PLAIN_FIRST_X`, `IME_PLAIN_FIRST_Y`
- `IME_PLAIN_SOURCE_X`, `IME_PLAIN_SOURCE_Y`
- `IME_PLAIN_TARGET_X`, `IME_PLAIN_TARGET_Y`
- `IME_SHOWCASE_RESET_X`, `IME_SHOWCASE_RESET_Y`
- `IME_SHOWCASE_FIRST_X`, `IME_SHOWCASE_FIRST_Y`
- `IME_SHOWCASE_SOURCE_X`, `IME_SHOWCASE_SOURCE_Y`
- `IME_SHOWCASE_TARGET_X`, `IME_SHOWCASE_TARGET_Y`

## Manual Checks Still Required

The scripts provide reproducible interaction and artifacts, but final IME judgement is still manual:

- confirm first Hangul syllable composes (no raw jamo split)
- confirm unfinished syllable does not migrate to the next field on click handoff
- confirm unfinished syllable does not migrate on tab handoff
- confirm focus highlight remains visually correct in both labs
- review artifacts for wrong viewport targeting and rerun with adjusted coordinates when needed

## Notes

- `--reuse-running` intentionally targets dedicated verification windows only: `FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY=1` for the plain primitive lab and `FLOEM_SHOWCASE_DEBUG_IME_LAB_ONLY=1` for the wrapper IME lab. The embedded labs inside the full showcase page remain useful for manual inspection, but they are too scroll- and layout-sensitive for stable native automation.
- Shell-side screenshots use the Swift CoreGraphics helper so we do not steal focus and clear IME preedit during `case_a`. If that helper fails, the matrix run fails instead of silently falling back to a focus-stealing capture path.
