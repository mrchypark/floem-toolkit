# floem-toolkit Track 2 And Track 3 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finish the `floem-toolkit` V0 surface area without letting `Input` and IME stabilization block the rest of the UI kit.

**Architecture:** Split execution into two lanes. `Track 2` stabilizes the host-sensitive text entry primitive in vendored Floem and keeps `floem-ui::Input` thin. `Track 3` finishes the non-input-heavy UI kit surface, showcase IA, and docs on top of already-stable token/theme/component contracts.

**Tech Stack:** Rust, Floem 0.2.0, vendored `floem` and `floem-winit`, workspace crates `floem-tokens`, `floem-theme`, `floem-ui`, `floem-charts`, `floem-showcase`

---

## Why This Split Exists

We have two very different kinds of work:

- host-sensitive primitives
  - `Input`
  - future `Textarea`
  - future `NumberInput`
  - future `Combobox`
  - IME, focus, preedit, selection, and blur/commit behavior
- fast UI kit surface
  - presentational and recipe-driven components
  - showcase structure
  - docs and examples
  - light/dark parity and visual polish

These should not share one delivery lane.

### Track 2

Owns the text entry primitive and only the text entry primitive.

Success means:

- plain Floem `text_input` behavior is acceptable under Korean IME
- vendored patches are minimal and explainable
- `floem-ui::Input` does not own IME routing policy
- the showcase IME lab is only a verification harness

### Track 3

Owns everything we can finish without redefining text entry ownership.

Success means:

- the showcase becomes a credible UI kit app
- simple/presentational components are complete and reviewable
- foundations and docs are strong enough that the project visibly moves forward
- no Track 3 PR depends on experimental `Input` abstractions

## Non-Goals For This Plan

- publishing crates
- CI automation
- theme editor and import/export
- chart interaction expansion beyond already-approved baseline work
- advanced input-like components before Track 2 is stable

## Global Rules

1. `Track 2` owns all text entry primitive behavior.
2. `Track 3` must not modify vendored Floem text entry code.
3. `crates/floem-showcase/src/main.rs` has one owner per wave.
4. `crates/floem-ui/src/lib.rs` and `crates/floem-ui/src/components/mod.rs` are main-agent integration files.
5. Any PR that touches `Input` must rerun the IME acceptance matrix.
6. Any PR that touches showcase structure must preserve the frozen diagnostics sections needed by Track 2.
7. `Textarea`, `NumberInput`, and `Combobox` are blocked until Track 2 reaches the declared exit gate.

## Verification Oracles

### Track 2 IME Matrix

Required cases:

- first Hangul syllable on an empty focused field composes normally
- rapid focus changes between three fields keep values independent
- composition started in field A does not mutate field B when focus moves before commit
- click handoff from field A to field B does not move the unfinished syllable into B
- `Tab` handoff from field A to field B does not move the unfinished syllable into B
- starting a new composition in B after leaving A works from the first syllable
- visual focus highlight remains unique and correct during all cases

Manual evidence:

- Hammerspoon or equivalent automation
- screenshots for first-syllable case
- screenshots for composition handoff before and after focus move

### Track 3 UI Kit Matrix

Required cases:

- light and dark theme visual walkthrough
- hover, active, disabled, and selected states match recipes
- showcase sections do not overlap or clip at default window size
- examples use crate-root imports
- simple components render correctly without depending on input internals

## PR Graph

### Track 2

#### PR-T2-1: Plain Floem Text Input Acceptance Harness

**Goal:** Prove what plain vendored Floem `text_input` does and does not handle, without toolkit-level IME routing.

**Files:**
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `crates/floem-ui/src/components/input.rs`
- Test: `crates/floem-ui/src/components/input.rs`

**Scope:**
- freeze a dedicated `IME lab` section in the showcase
- keep `floem-ui::Input` as thin as possible
- expose only local debug signals needed for verification
- remove any toolkit-owned IME router that is not strictly necessary

**Step 1: Write the failing tests**

- add focused/highlight regression tests for `Input`
- add local change-tracking tests for multiple fields if missing

**Step 2: Run the targeted tests**

Run:

```bash
cargo test -p floem-ui input -- --nocapture
```

Expected:

- current `Input` contract tests pass or reveal remaining wrapper bugs

**Step 3: Freeze the showcase verification harness**

- keep `IME lab`
- keep multi-input isolation area
- make the debug labels honest about committed value versus preedit state

**Step 4: Run the manual matrix**

Run:

```bash
cargo run -p floem-showcase
```

Then verify:

- first-syllable case
- click handoff case
- tab handoff case

**Step 5: Commit**

```bash
git add crates/floem-showcase/src/main.rs crates/floem-ui/src/components/input.rs
git commit -m "test: freeze text input acceptance harness"
```

**Parallelization:** blocking start for Track 2, but Track 3 may begin once showcase section ownership is frozen

---

#### PR-T2-2: Vendor Text Input Stabilization

**Goal:** Move IME, preedit, blur, and commit behavior into vendored Floem where it belongs.

**Files:**
- Modify: `vendor/floem/src/views/text_input.rs`
- Modify: `vendor/floem/src/app_state.rs`
- Modify: `vendor/floem-winit/src/platform_impl/macos/view.rs`
- Test: `vendor/floem/src/views/text_input.rs`

**Scope:**
- keep patches minimal
- prefer local widget state over toolkit-owned global routing
- focus ordering must be engine-correct
- platform-specific marked-text cleanup must live in `floem-winit`, not in `floem-ui`

**Step 1: Write or extend failing tests**

- `display_text_with_preedit`
- local commit acceptance
- preedit clear conditions
- focus ordering assumptions where testable

**Step 2: Run the vendored targeted tests**

Run:

```bash
cargo test --manifest-path vendor/floem/Cargo.toml text_input -- --nocapture
```

Expected:

- tests fail on the exact primitive behavior being fixed

**Step 3: Implement minimal vendor patch**

- patch `focus_changed` ordering only if still necessary
- patch `text_input` preedit/commit behavior only where proven by the matrix
- patch macOS marked-text shutdown only in the platform layer

**Step 4: Re-run primitive and workspace verification**

Run:

```bash
cargo check --workspace
cargo test -p floem-ui --lib
cargo test --manifest-path vendor/floem/Cargo.toml text_input -- --nocapture
```

Expected:

- all targeted tests pass

**Step 5: Re-run manual IME matrix**

Run:

```bash
cargo run -p floem-showcase
```

Then collect:

- first-syllable screenshot
- handoff-before screenshot
- handoff-after screenshot

**Step 6: Commit**

```bash
git add vendor/floem/src/views/text_input.rs vendor/floem/src/app_state.rs vendor/floem-winit/src/platform_impl/macos/view.rs
git commit -m "fix: stabilize vendored floem text input ime behavior"
```

**Parallelization:** main-agent only

---

#### PR-T2-3: Thin Wrapper Recovery And Exit Gate

**Goal:** Restore `floem-ui::Input` to a thin wrapper and declare the exit gate for future input-like components.

**Files:**
- Modify: `crates/floem-ui/src/components/input.rs`
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `docs/plans/2026-03-11-floem-reactive-contract-process.md`

**Scope:**
- remove leftover workaround logic that belongs in vendor Floem
- document the stable boundary clearly
- declare which future components may now reuse the primitive

**Step 1: Write the failing thin-wrapper expectation tests**

- wrapper keeps focus styling and buffer sync
- wrapper does not introduce extra IME ownership state

**Step 2: Run the targeted tests**

Run:

```bash
cargo test -p floem-ui input -- --nocapture
```

**Step 3: Simplify the wrapper**

- retain style, placeholder, disabled, invalid, and debug surface
- avoid toolkit-owned IME routing

**Step 4: Update docs**

- add the final primitive boundary to the reactive contract process doc
- list `Textarea` as first allowed reuse target
- keep `NumberInput` and `Combobox` behind a new review gate if they need extra policy

**Step 5: Full gate**

Run:

```bash
cargo check --workspace
cargo test --workspace
cargo run -p floem-showcase
```

**Step 6: Commit**

```bash
git add crates/floem-ui/src/components/input.rs crates/floem-showcase/src/main.rs docs/plans/2026-03-11-floem-reactive-contract-process.md
git commit -m "refactor: keep floem-ui input as a thin wrapper"
```

**Exit Gate:** only after PR-T2-3 can we start `Textarea`. `NumberInput` and `Combobox` still require a separate design check.

### Track 3

#### PR-T3-1: Showcase IA Freeze And Foundations Polish

**Goal:** Make the showcase feel like a real UI kit app without touching text entry primitives.

**Files:**
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `README.md`

**Scope:**
- improve section order
- improve spacing and readability
- keep diagnostics discoverable but not dominant
- clarify V0 surface area and current maturity

**Step 1: Write a checklist for the visual review**

- no overlap
- no clipped labels
- sections feel grouped
- foundations and UI sections are easy to scan

**Step 2: Make the minimal layout changes**

- spacing
- subsection ordering
- heading copy
- documentation copy

**Step 3: Run the showcase**

Run:

```bash
cargo run -p floem-showcase
```

Then verify:

- default window layout
- light/dark switch
- IME lab remains reachable

**Step 4: Commit**

```bash
git add crates/floem-showcase/src/main.rs README.md
git commit -m "feat: polish showcase information architecture"
```

**Parallelization:** may run in parallel with PR-T2-2 after the `IME lab` section is frozen

---

#### PR-T3-2: Simple Component Wave A

**Goal:** Finish small non-input-heavy components that increase library surface fast.

**Files:**
- Create or Modify: `crates/floem-ui/src/components/badge.rs`
- Create or Modify: `crates/floem-ui/src/components/separator.rs`
- Create or Modify: `crates/floem-ui/src/components/typography.rs`
- Modify: `crates/floem-ui/src/components/mod.rs`
- Modify: `crates/floem-ui/src/lib.rs`
- Modify: `crates/floem-showcase/src/main.rs`

**Step 1: Write failing compile or rendering tests**

- component constructors compile
- exported types are reachable from crate root

**Step 2: Implement minimal versions**

- `Badge`
- `Separator`
- `Typography`

**Step 3: Showcase each component**

- basic
- variants if needed
- light/dark pass

**Step 4: Verification**

Run:

```bash
cargo test -p floem-ui
cargo check --workspace
cargo run -p floem-showcase
```

**Step 5: Commit**

```bash
git add crates/floem-ui/src/components/badge.rs crates/floem-ui/src/components/separator.rs crates/floem-ui/src/components/typography.rs crates/floem-ui/src/components/mod.rs crates/floem-ui/src/lib.rs crates/floem-showcase/src/main.rs
git commit -m "feat: add badge separator and typography"
```

**Parallelization:** safe in parallel with PR-T2-2; showcase integration owned by main agent

---

#### PR-T3-3: Simple Component Wave B

**Goal:** Add more visible but still non-host-sensitive components.

**Files:**
- Create or Modify: `crates/floem-ui/src/components/avatar.rs`
- Create or Modify: `crates/floem-ui/src/components/progress.rs`
- Create or Modify: `crates/floem-ui/src/components/breadcrumbs.rs`
- Create or Modify: `crates/floem-ui/src/components/button_group.rs`
- Modify: `crates/floem-ui/src/components/mod.rs`
- Modify: `crates/floem-ui/src/lib.rs`
- Modify: `crates/floem-showcase/src/main.rs`

**Step 1: Write failing compile or rendering tests**

**Step 2: Implement minimal components**

**Step 3: Showcase each component**

- default
- variant/state if applicable

**Step 4: Verification**

Run:

```bash
cargo test -p floem-ui
cargo check --workspace
cargo run -p floem-showcase
```

**Step 5: Commit**

```bash
git add crates/floem-ui/src/components/avatar.rs crates/floem-ui/src/components/progress.rs crates/floem-ui/src/components/breadcrumbs.rs crates/floem-ui/src/components/button_group.rs crates/floem-ui/src/components/mod.rs crates/floem-ui/src/lib.rs crates/floem-showcase/src/main.rs
git commit -m "feat: add simple navigation and status components"
```

**Parallelization:** safe in parallel with PR-T2-2; showcase integration owned by main agent

---

#### PR-T3-4: Docs, Examples, And Review Polish

**Goal:** Make the finished Track 3 surface easy to review and hard to misunderstand.

**Files:**
- Modify: `README.md`
- Modify: `crates/floem-showcase/src/main.rs`
- Optional Create: per-crate `README.md`
- Optional Create: example snippets under `crates/floem-showcase` or `docs/`

**Scope:**
- document which components are production-ready
- document which components are blocked on Track 2
- keep examples aligned with crate-root imports

**Step 1: Write the docs checklist**

- foundations
- current component matrix
- blocked advanced components
- verification notes

**Step 2: Update docs and examples**

**Step 3: Final Track 3 verification**

Run:

```bash
cargo check --workspace
cargo test --workspace
cargo run -p floem-showcase
```

**Step 4: Commit**

```bash
git add README.md crates/floem-showcase/src/main.rs docs/
git commit -m "docs: polish floem-toolkit showcase and usage guidance"
```

**Parallelization:** sequential final Track 3 consolidation

## Parallel Execution Rules

### Wave 1

- Main agent: PR-T2-1
- No parallel Track 2 work yet

### Wave 2

- Main agent: PR-T2-2
- Worker A: PR-T3-1 draft edits, without final `main.rs` integration
- Worker B: PR-T3-2 component files only
- Worker C: PR-T3-3 component files only

Main agent integrates:

- `crates/floem-showcase/src/main.rs`
- `crates/floem-ui/src/lib.rs`
- `crates/floem-ui/src/components/mod.rs`

### Wave 3

- Main agent: PR-T2-3
- Track 3 integration continues
- No `Textarea`, `NumberInput`, or `Combobox` work until PR-T2-3 exit gate passes

### Wave 4

- Main agent: PR-T3-4 final polish
- optional follow-up design for `Textarea`

## Review Loop

Each PR must run the same freeze:

1. define exact file set
2. run targeted tests first
3. run workspace check
4. run workspace tests
5. launch showcase if UI-visible
6. do matrix walkthrough if it changes ownership or visual contracts
7. request subagent review on the exact diff
8. address only material findings

Do not promote a PR while:

- IME acceptance evidence is stale
- showcase visual overlap regresses
- light/dark visual walkthrough has not been rerun for changed components

## Finish Criteria

### Track 2 Complete

- IME first-syllable case is verified
- IME handoff case is verified
- multi-input independence is verified
- `floem-ui::Input` is a thin wrapper
- `Textarea` is unblocked

### Track 3 Complete

- showcase IA is polished
- simple/presentational component waves are complete
- docs and examples reflect the current public surface
- blocked advanced input-like components are clearly called out

### Combined Outcome

- the project visibly progresses as a UI kit
- text entry is treated as a primitive stabilization effort, not a recurring toolkit workaround
- future advanced input-like components have a safer foundation

## Recommended Immediate Next Move

Start with:

1. PR-T2-1
2. in parallel planning, queue PR-T3-1, PR-T3-2, and PR-T3-3 file ownership
3. once PR-T2-1 freezes the showcase harness, start PR-T2-2 and Track 3 worker slices together
