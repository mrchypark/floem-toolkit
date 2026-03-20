# floem Native Input Verification Harness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a repeatable native input verification harness so `Input` and future text-entry components can be debugged and validated without relying on ad hoc manual reproduction.

**Architecture:** Separate verification into two layers. A plain primitive lab verifies vendored Floem `text_input` behavior without toolkit abstraction. A showcase IME lab verifies the final user-facing `floem-ui::Input` behavior. Both are exercised by native automation, screenshots, and event logs.

**Tech Stack:** Rust, Floem 0.2.0, vendored `floem` and `floem-winit`, `floem-showcase`, Hammerspoon Lua, macOS `screencapture`, shell scripts

---

## Problem Statement

We currently have a reliable way to reproduce some IME issues, but we do not yet
have a reliable way to verify them repeatedly.

That creates four problems:

- we cannot cleanly separate `floem-ui` wrapper bugs from vendored Floem primitive bugs
- we cannot prove whether a fix holds across repeated runs
- visual regressions and input-state regressions get mixed together
- future work on `Textarea`, `NumberInput`, or `Combobox` would inherit the same uncertainty

This plan creates a verification harness first, then uses that harness as the
gate for all future text-entry work.

## Target Outcome

After this plan lands, we should be able to run one verification flow that:

- launches a plain text-input lab
- launches the toolkit showcase IME lab
- drives both with native automation
- captures screenshots at stable checkpoints
- records event logs with enough detail to diagnose failures
- produces a pass/fail summary for the known IME matrix

## Non-Goals

- CI integration in this phase
- OCR-based automated semantic comparison of screenshots
- cross-platform IME parity beyond macOS in this phase
- building a general-purpose desktop UI test framework

## Verification Layers

### Layer 1: Plain Primitive Lab

Purpose:

- prove whether vendored Floem `text_input` itself is correct
- isolate engine and platform behavior from toolkit wrapper behavior

Properties:

- no `floem-ui::Input`
- no theme recipe complexity beyond visibility and focus clarity
- minimum three fields
- explicit labels for each test field

### Layer 2: Showcase IME Lab

Purpose:

- verify the final public `floem-ui::Input` in the actual UI kit app
- verify visual states, focus ring, and integration with surrounding layout

Properties:

- keep the existing IME lab
- make the diagnostics truthful about committed value and local input state
- avoid toolkit-owned IME routing here

### Layer 3: Native Automation

Purpose:

- drive actual focus transitions and Korean IME composition
- repeat the same scenarios without relying on human timing

Properties:

- use Hammerspoon Lua scripts
- click by window-relative coordinates
- send key sequences using the active Korean 2-set input method
- store screenshots per scenario

### Layer 4: Event Logs

Purpose:

- make failures explainable
- distinguish `focus`, `preedit`, `commit`, `disabled`, and value mutations

Properties:

- plain text logs are sufficient
- logs should be scenario-tagged
- keep them low-volume and scenario-specific

## Acceptance Matrix

This matrix becomes the gate for any PR touching text-entry behavior.

### Case A: First Hangul Syllable

Steps:

- focus empty field
- input `d`, `k`, `s`

Pass:

- the field visibly shows a composed syllable path
- no raw jamo migration occurs
- focus highlight stays correct

### Case B: Click Handoff During Composition

Steps:

- focus source field
- input `d`, `k`, `v` to start `앞`
- click target field before composition is fully settled

Pass:

- unfinished syllable does not appear in target
- source is either left with the pending glyph or cleared, but never copied into target
- target can start a fresh composition afterward

### Case C: Tab Handoff During Composition

Steps:

- focus source field
- start composition
- hit `Tab`

Pass:

- pending glyph does not migrate into the next field
- target accepts a new composition from the first syllable

### Case D: Rapid Multi-Field Focus Changes

Steps:

- switch across three fields quickly
- type in each field

Pass:

- field values remain independent
- focus ring remains unique
- no stale composition or stray commit appears elsewhere

### Case E: Visual Focus Regression

Steps:

- click each field in sequence
- capture screenshots

Pass:

- only the active field is visually highlighted
- highlight remains visible in both light and dark themes

## Files And Structure

Planned additions:

- Create: `crates/floem-showcase/src/plain_input_lab.rs`
- Modify: `crates/floem-showcase/src/main.rs`
- Create: `scripts/ime/README.md`
- Create: `scripts/ime/run_plain_input_lab.lua`
- Create: `scripts/ime/run_showcase_ime_lab.lua`
- Create: `scripts/ime/capture.sh`
- Create: `scripts/ime/run_matrix.sh`
- Optional Create: `scripts/ime/common.lua`
- Optional Create: `artifacts/ime/.gitkeep`

Potential log touch points:

- Modify: `vendor/floem/src/views/text_input.rs`
- Modify: `crates/floem-ui/src/components/input.rs`

Documentation:

- Modify: `README.md`
- Modify: `docs/plans/2026-03-11-floem-reactive-contract-process.md`

## Task 1: Add Plain Primitive Lab

**Files:**
- Create: `crates/floem-showcase/src/plain_input_lab.rs`
- Modify: `crates/floem-showcase/src/main.rs`

**Step 1: Add a new plain lab module**

- build a small view using plain `floem::views::text_input`
- include three labeled fields
- include a reset button
- keep the styling minimal and diagnostic-friendly

**Step 2: Add a route or section in the showcase**

- make the plain lab reachable without disturbing the existing showcase
- also add a dedicated single-surface mode for native automation
  - `FLOEM_SHOWCASE_DEBUG_PLAIN_INPUT_LAB_ONLY=1 cargo run -p floem-showcase`
- preserve the existing IME lab

**Step 3: Run the app**

Run:

```bash
cargo run -p floem-showcase
```

Expected:

- both the plain lab and the toolkit IME lab are visible and usable
- the dedicated plain-lab-only mode opens a stable automation surface with no unrelated sections above it

**Step 4: Commit**

```bash
git add crates/floem-showcase/src/plain_input_lab.rs crates/floem-showcase/src/main.rs
git commit -m "feat: add plain input verification lab"
```

## Task 2: Add Minimal Event Logging

**Files:**
- Modify: `vendor/floem/src/views/text_input.rs`
- Modify: `crates/floem-ui/src/components/input.rs`

**Step 1: Add scenario-friendly log output**

- log focus gained/lost
- log ime preedit start/clear
- log ime commit
- log value mutation points

**Step 2: Keep logs opt-in**

- gate logs behind an environment variable
- keep normal showcase output readable

**Step 3: Run targeted verification**

Run:

```bash
FLOEM_IME_DEBUG=1 cargo run -p floem-showcase
cargo --config 'patch.crates-io.floem-winit.path="/Users/cypark/.codex/worktrees/cbb8/floem-ui-kit/vendor/floem-winit"' test --manifest-path vendor/floem/Cargo.toml ime_debug_enabled_accepts -- --nocapture --test-threads=1
```

Expected:

- logs appear only when enabled
- logs are readable enough to match the matrix steps
- the vendored Floem logging aliases are covered by an explicit vendor test run, because `vendor/floem` is not part of the default workspace test set

**Step 4: Commit**

```bash
git add vendor/floem/src/views/text_input.rs crates/floem-ui/src/components/input.rs
git commit -m "chore: add opt-in ime verification logging"
```

## Task 3: Add Native Automation Scripts

**Files:**
- Create: `scripts/ime/common.lua`
- Create: `scripts/ime/run_plain_input_lab.lua`
- Create: `scripts/ime/run_showcase_ime_lab.lua`
- Create: `scripts/ime/capture.sh`

**Step 1: Write common helpers**

- locate the showcase window
- activate the app
- click by window-relative coordinates
- type key sequences

**Step 2: Add plain lab scenario script**

- first syllable
- click handoff
- tab handoff

**Step 3: Add showcase IME lab scenario script**

- same cases against `floem-ui::Input`

**Step 4: Add screenshot capture helper**

- store images under `artifacts/ime/<scenario>/`
- timestamp or overwrite predictably

**Step 5: Verify scripts manually**

Run:

```bash
hs -c "dofile('scripts/ime/run_plain_input_lab.lua')"
hs -c "dofile('scripts/ime/run_showcase_ime_lab.lua')"
```

Expected:

- scripts complete without manual timing hacks beyond bounded sleeps

**Step 6: Commit**

```bash
git add scripts/ime/common.lua scripts/ime/run_plain_input_lab.lua scripts/ime/run_showcase_ime_lab.lua scripts/ime/capture.sh
git commit -m "test: add native ime automation scripts"
```

## Task 4: Add Matrix Runner

**Files:**
- Create: `scripts/ime/run_matrix.sh`
- Create: `scripts/ime/README.md`
- Optional Create: `artifacts/ime/.gitkeep`

**Step 1: Write the runner**

- launch the showcase
- run the plain lab scenarios
- run the toolkit lab scenarios
- collect screenshots
- print a short summary of artifacts

**Step 2: Document usage**

- prerequisites
- required input method
- artifact paths
- known manual checks that still remain

**Step 3: Dry-run the matrix**

Run:

```bash
bash scripts/ime/run_matrix.sh
```

Expected:

- artifacts are created
- summary prints the expected paths

**Step 4: Commit**

```bash
git add scripts/ime/run_matrix.sh scripts/ime/README.md artifacts/ime/.gitkeep
git commit -m "test: add native ime verification matrix runner"
```

## Task 5: Tighten Showcase Diagnostics

**Files:**
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `crates/floem-ui/src/components/input.rs`

**Step 1: Make diagnostics honest**

- distinguish committed value from in-progress local state where possible
- keep the UI readable
- avoid presenting misleading empty-state labels during active composition

**Step 2: Run the showcase**

Run:

```bash
cargo run -p floem-showcase
```

Expected:

- diagnostics help validate input behavior instead of confusing it

**Step 3: Commit**

```bash
git add crates/floem-showcase/src/main.rs crates/floem-ui/src/components/input.rs
git commit -m "feat: improve ime lab diagnostics"
```

## Task 6: Wire The Harness Into The Development Process

**Files:**
- Modify: `README.md`
- Modify: `docs/plans/2026-03-11-floem-reactive-contract-process.md`
- Modify: `docs/plans/2026-03-16-floem-toolkit-track-2-3-plan.md`

**Step 1: Update docs**

- explain when to use the plain lab versus the showcase lab
- document the acceptance matrix
- state that future input-like components are blocked on this harness

**Step 2: Add PR gate guidance**

- any text-entry PR reruns the matrix
- screenshot artifacts are expected for IME bugfix PRs

**Step 3: Verify docs are coherent**

Run:

```bash
rg -n "IME|plain lab|showcase IME lab|matrix" README.md docs/plans
```

Expected:

- the new verification flow is described consistently

**Step 4: Commit**

```bash
git add README.md docs/plans/2026-03-11-floem-reactive-contract-process.md docs/plans/2026-03-16-floem-toolkit-track-2-3-plan.md
git commit -m "docs: add native input verification workflow"
```

## PR Slices

### PR-H1: Plain Lab And Logging

Includes:

- Task 1
- Task 2

Purpose:

- make failures diagnosable before changing automation

### PR-H2: Native Automation And Matrix Runner

Includes:

- Task 3
- Task 4

Purpose:

- make the same IME cases replayable

### PR-H3: Diagnostics And Process Integration

Includes:

- Task 5
- Task 6

Purpose:

- make the harness part of the normal development workflow

## Parallelization Rules

- `PR-H1` is sequential and main-agent owned
- `PR-H2` can be partially parallelized
  - worker A: Lua automation
  - worker B: shell runner and README
  - main agent: integration and final paths
- `PR-H3` is sequential because it touches shared docs and showcase framing

## Final Verification Gate

Run:

```bash
cargo check --workspace
cargo test --workspace
bash scripts/ime/run_matrix.sh
```

Then confirm:

- first-syllable screenshots exist
- handoff-before and handoff-after screenshots exist
- plain lab and showcase lab both produce artifacts
- logs are usable for failure diagnosis

## Finish Criteria

- we can distinguish plain primitive failures from toolkit wrapper failures
- IME regressions are no longer diagnosed from memory alone
- every future input-like PR has a stable verification path
- the harness is lightweight enough that we will actually keep using it

## Recommended Immediate Next Move

Start with `PR-H1`.

Reason:

- it improves observability first
- it does not require us to guess the final automation structure too early
- it gives Track 2 a stable base before more vendor text-input work
