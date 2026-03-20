# floem-toolkit PR Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver `floem-toolkit` from the current partial V0 state to a reviewable PR sequence with explicit ownership boundaries, repeatable verification, and realistic parallelism.

**Architecture:** Build on top of Floem's native reactive pipeline and the approved `reactive contract-first` process. Reuse Floem primitives first. Only introduce toolkit-owned coordinators where there is real cross-widget ownership or policy, such as input IME routing, overlay focus return, and chart interaction ownership.

**Tech Stack:** Rust, Floem 0.2.0, floem-editor-core 0.2.0, workspace crates `floem-tokens`, `floem-theme`, `floem-ui`, `floem-charts`, `floem-showcase`

---

## Baseline

Current workspace already contains:

- token/theme crates with semantic tokens and recipe resolution
- V0-ish UI components in `floem-ui`
- a running `floem-showcase`
- chart model, scale, and benchmark basics in `floem-charts`
- the approved process docs:
  - `docs/plans/2026-03-10-floem-toolkit-design.md`
  - `docs/plans/2026-03-11-floem-reactive-contract-process.md`

Known unstable areas:

- `Input` focus and Korean IME ownership
- overlay ownership and focus return policy
- incomplete interaction token coverage
- showcase still mixes demo and diagnostics weakly
- charts do not yet expose interactive ownership through a stable contract

## Review Target Freeze

- `target kind`: `plan`
- `review target`: `docs/plans/2026-03-11-floem-toolkit-pr-implementation-plan.md`
- `change context`: `architecture`, `ui`, `performance`, `bugfix`, `refactor`
- `required layers`:
  - `intent/scope`
  - `correctness/invariants`
  - `workflow/operability`
  - `complexity/YAGNI`
  - `boundaries/contracts`
  - `state/accessibility`
  - `equivalence/regression-surface`
  - `reproduction/test-gap`
  - `capacity/contention`
- `non-goals`:
  - branch/commit history cleanup
  - publishing crates
  - CI or packaging automation
  - theme editor/import-export tooling

## PR Strategy

Use PR-sized slices with explicit dependency edges.

Rules:

- A PR must have one dominant purpose.
- If two PRs touch the same unstable ownership layer, they are sequential.
- If two PRs touch separate crates and only consume stable contracts, they may run in parallel.
- `floem-showcase/src/main.rs` has one owner per wave. Diagnostic harness structure is frozen at the start of each wave.
- `crates/floem-ui/src/components/mod.rs`, `crates/floem-ui/src/lib.rs`, chart benchmark files, and crate-root export stitching are main-agent integration files unless a PR explicitly says otherwise.
- Every later PR that touches `input.rs` or input-derived ownership behavior must rerun the PR1 input ownership matrix.
- Every later PR that touches `dialog.rs`, `popover.rs`, or `select.rs` must rerun the PR3 overlay matrix.
- Every later PR that touches chart interaction ownership must rerun the PR4 chart interaction matrix and benchmark suite.

### Canonical PR Gate Order

Every code PR ends with this gate order:

1. targeted tests for the changed contract
2. `cargo check --workspace`
3. `cargo test --workspace`
4. showcase launch and smoke verification if UI-visible
5. additional matrix walkthrough if the PR changes ownership or accessibility contracts
6. benchmark gate if the PR changes chart hot paths
7. PR-level review loop

Benchmark policy:

- `cargo bench` is additive for non-chart PRs
- `cargo bench` is required for `PR4`, `PR9A`, and `PR9B`
- PR4 captures the benchmark baseline
- after PR4, any tracked benchmark regression above 10% requires explanation in the PR notes
- any tracked regression above 20% blocks promotion to the next wave until fixed or explicitly re-baselined

## Shared Verification Oracles

### Input Ownership Matrix

PR1 defines the baseline oracle. It must be rerun by every later PR that edits `input.rs` or input ownership paths.

Required cases:

- three independent inputs keep distinct values under rapid focus changes
- only one visual focus owner is shown at a time
- Korean composition started in field A does not mutate field B when focus moves before commit
- Korean composition started in field A then mouse click to field B leaves the pending glyph with A or clears it, but never migrates into B
- Korean composition started in field A then `Tab` to B does not duplicate or migrate the pending glyph
- starting a new composition in B after leaving A works from the first composed syllable
- diagnostics expose exactly one `focused_id`, zero or one `composing_id`, and a route log for the last transitions

### Overlay Ownership Matrix

PR3 defines the overlay oracle. It must be rerun by every later PR that edits overlay ownership paths.

Required invariants:

- `Dialog` is modal and traps focus within the content while open
- `Popover` and `Select` are non-modal and do not trap focus outside their popup
- `Escape` dismisses only the topmost eligible overlay
- outside click dismisses `Popover` and `Select` by default
- outside click does not dismiss `Dialog` by default in V0
- focus returns to the trigger when an overlay closes and the trigger is still mounted
- nested overlay ownership is stack-based and child dismiss does not implicitly collapse the parent unless configured
- `Select` trigger/content semantics are preserved: trigger state, listbox role, option navigation, and active selection ownership

### Chart Interaction Matrix

PR4 defines the chart oracle. It must be rerun by every later PR that edits chart interaction ownership.

Required invariants:

- at most one hovered datum owner per plot surface
- series hover precedence is explicit and stable across plot, legend, and tooltip interactions
- tooltip anchor is derived from the current ownership state and cannot remain stale after theme or viewport changes
- selection and viewport state reset cleanly when the referenced series or datum disappears
- diagnostics expose hovered datum, hovered series, selected item, and viewport state

### Chart Benchmark Suite

PR4 captures the baseline for these scenarios:

- `scale_mapping_4096`
- `nearest_point_lookup_4096`
- `nearest_point_lookup_multi_series_8x1024`
- `hover_projection_8x1024`
- `selection_projection_8x1024`

PR9A and PR9B extend but do not replace this suite.

## PR Graph

### PR1: Input Ownership And Minimal Diagnostics Scaffold

**Goal:** Stabilize `Input` ownership using the approved contract-first process and install the minimal showcase diagnostics scaffold needed by later PRs.

**Files:**
- Modify: `crates/floem-ui/src/components/input.rs`
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `crates/floem-ui/src/lib.rs`
- Optional Create: `crates/floem-ui/src/components/input_debug.rs`
- Test: `crates/floem-ui/src/components/input.rs`

**Scope:**
- add app-owned input coordinator state
- expose `focused`, `composing`, `active`, `value`, and route log in diagnostics
- install stable showcase sections for later ownership verification
- keep reuse-first discipline
- do not introduce a framework-wide generic debug runtime

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- full input ownership matrix walkthrough

**Parallelization:** blocking foundation, do not parallelize with PR2, PR3, or PR4

---

### PR2: Interaction Tokens And V0 Control Recipes

**Goal:** Complete the missing interaction contract for V0 controls without widening public API unnecessarily.

**Files:**
- Modify: `crates/floem-theme/src/resolved.rs`
- Modify: `crates/floem-theme/src/recipes.rs`
- Modify: `crates/floem-theme/src/lib.rs`
- Modify: `crates/floem-ui/src/components/button.rs`
- Modify: `crates/floem-ui/src/components/input.rs`
- Modify: `crates/floem-ui/src/components/checkbox.rs`
- Modify: `crates/floem-ui/src/components/tabs.rs`
- Modify: `crates/floem-ui/src/components/select.rs`

**Scope:**
- hover/active/focus/selected rules
- dark/light parity
- prefer helper expansion over broad recipe API churn
- preserve the PR1 input ownership behavior and diagnostics contract

**Verification:**
- `cargo test -p floem-theme`
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- rerun the PR1 input ownership matrix
- light/dark visual walkthrough on the frozen showcase pages

**Parallelization:** sequential after PR1, may run in parallel with PR4 if the main agent owns any shared showcase stitching

---

### PR3: Overlay Ownership For Dialog, Popover, And Select

**Goal:** Implement semantic overlay ownership, focus return, and dismiss policy without recreating Floem raw pointer ownership.

**Files:**
- Modify: `crates/floem-ui/src/components/dialog.rs`
- Modify: `crates/floem-ui/src/components/popover.rs`
- Modify: `crates/floem-ui/src/components/select.rs`
- Modify: `crates/floem-showcase/src/main.rs`
- Optional Create: `crates/floem-ui/src/components/overlay_coordinator.rs`

**Scope:**
- `open_id`
- `focus_return_target`
- `last_dismiss_reason`
- overlay diagnostics panel in the existing showcase scaffold
- modal versus non-modal semantics
- nested overlay ownership rules

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- full overlay ownership matrix walkthrough

**Parallelization:** sequential after PR1, and sequential after PR2 if both modify `select.rs`

---

### PR4: Charts Interaction Contract And Baseline Benchmarks

**Goal:** Add app-owned chart interaction state, diagnostics, and the benchmark baseline without rewriting the existing chart core.

**Files:**
- Modify: `crates/floem-charts/src/interaction_state.rs`
- Modify: `crates/floem-charts/src/model.rs`
- Modify: `crates/floem-charts/src/views/mod.rs`
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `crates/floem-charts/benches/line_chart.rs`

**Scope:**
- hovered series
- hovered datum
- selected item
- viewport state
- chart diagnostics views
- baseline benchmarks for interaction-heavy paths

**Verification:**
- `cargo test -p floem-charts`
- `cargo bench -p floem-charts --bench line_chart -- --noplot`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- full chart interaction matrix walkthrough
- baseline benchmark capture in PR notes

**Parallelization:** may run in parallel with PR2 after PR1 if the main agent owns shared showcase stitching and benchmark integration

---

### PR5: Showcase Navigation, Docs, And Harness Polish

**Goal:** Reorganize showcase around the already-installed diagnostics scaffold without redefining the ownership harness introduced in PR1, PR3, and PR4.

**Files:**
- Modify: `crates/floem-showcase/src/main.rs`
- Modify: `crates/floem-showcase/README.md`
- Modify: `README.md`

**Scope:**
- `normal` and `diagnostics` navigation
- page organization and spacing polish
- copyable docs sections
- preserve all previously proven diagnostics surfaces

**Verification:**
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- showcase smoke path through both normal and diagnostics navigation

**Parallelization:** sequential after PR3 and PR4, because it owns showcase structure rather than append-only diagnostics sections

---

### PR6: V1 Simple Components

**Goal:** Add V1 components that do not require new ownership policy.

**Target components:**
- `Textarea`
- `Badge`
- `Separator`
- `Switch`
- `Progress`
- `Typography`
- `Avatar`

**Ownership model:**
- subagents may own individual component files only
- the main agent owns `components/mod.rs`, `floem-ui/src/lib.rs`, and showcase integration

**Verification:**
- per-component tests where relevant
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`

**Parallelization:** after PR2 and PR5, with component-file subagents and main-agent integration

---

### PR7: V1 Interactive Components

**Goal:** Add V1 components that require ownership or overlay policy, using the patterns proven in PR1 and PR3.

**Target components:**
- `Tooltip`
- `DropdownMenu`
- `Toast`
- `Accordion`
- `IconButton`

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- keyboard and accessibility checks in showcase
- rerun overlay matrix for any overlay-derived components

**Parallelization:** sequential after PR3 and PR5

---

### PR8A: NumberInput

**Goal:** Prove advanced input behavior without mixing overlay and pointer ownership in the same PR.

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- rerun input ownership matrix

**Parallelization:** after PR2 and PR5, may run in parallel with PR8C and PR8D if showcase integration stays with the main agent

---

### PR8B: Combobox

**Goal:** Add the highest-risk V2 control after input and overlay contracts are stable.

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- rerun input ownership matrix
- rerun overlay ownership matrix

**Parallelization:** sequential after PR8A and PR3

---

### PR8C: Slider

**Goal:** Add pointer-driven control behavior without entangling it with input or overlay contracts.

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`

**Parallelization:** after PR2 and PR5, may run in parallel with PR8A and PR8D if showcase integration stays with the main agent

---

### PR8D: ButtonGroup And Breadcrumbs

**Goal:** Add simple V2 compositional controls that should not expand ownership policy.

**Verification:**
- `cargo test -p floem-ui`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`

**Parallelization:** after PR2 and PR5, may run in parallel with PR8A and PR8C if showcase integration stays with the main agent

---

### PR9A: Chart Marks Expansion

**Goal:** Extend chart marks without widening interaction ownership in the same PR.

**Target features:**
- area chart
- stacked bar
- grouped bar

**Verification:**
- `cargo test -p floem-charts`
- `cargo bench -p floem-charts --bench line_chart -- --noplot`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- compare results against PR4 baseline

**Parallelization:** after PR4, internal task parallelism only unless file ownership is proven disjoint

---

### PR9B: Chart Interaction Expansion

**Goal:** Finish the chart interaction roadmap on top of the PR4 ownership contract.

**Target features:**
- legend ownership
- zoom/pan
- brush
- crosshair

**Verification:**
- `cargo test -p floem-charts`
- `cargo bench -p floem-charts --bench line_chart -- --noplot`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- rerun chart interaction matrix
- compare results against PR4 baseline

**Parallelization:** sequential after PR9A

---

### PR10: Docs, Examples, And Residual Hardening

**Goal:** Close the loop on agent-facing docs, examples, and residual polish.

**Files:**
- `README.md`
- crate READMEs
- showcase docs sections
- plan and process cross-links as needed

**Verification:**
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p floem-showcase`
- example and showcase smoke checks

**Parallelization:** final consolidation PR, sequential

## Parallel Waves

### Wave A

- PR1 only

Reason:
- `Input` contract and the minimal showcase diagnostics scaffold are the first unstable ownership layer and block all later ownership work.

### Wave B

- PR2
- PR4

Parallel condition:
- showcase structure stays frozen from PR1
- the main agent owns any `floem-showcase/src/main.rs` stitching
- subagents stay inside their crate-local files plus append-only showcase sections assigned by the main agent

### Wave C

- PR3 only
- then PR5

Reason:
- overlay ownership and showcase structure both heavily depend on the same verification surface

### Wave D

- PR6 with internal component-file parallelism only

Suggested subagent split:
- agent A: `Textarea`, `Badge`, `Separator`
- agent B: `Switch`, `Progress`
- agent C: `Typography`, `Avatar`
- main agent: `components/mod.rs`, crate-root exports, showcase

### Wave E

- PR7 sequential

### Wave F

- PR8A
- PR8C
- PR8D

Parallel condition:
- the main agent owns showcase integration
- `input.rs`, overlay files, and crate-root exports are not edited by more than one worker at a time

Then:

- PR8B sequential

### Wave G

- PR9A sequential
- PR9B sequential

### Wave H

- PR10 final consolidation

## PR-Level Review Loop

Each PR must follow this review freeze before implementation review starts.

- `target kind`: `code`
- `review target`: exact PR diff or exact file set
- `change context`: choose from `architecture`, `ui`, `bugfix`, `performance`, `refactor`
- `required layers`:
  - `intent/scope`
  - `correctness/invariants`
  - `workflow/operability`
  - `complexity/YAGNI`
  - `boundaries/contracts` when ownership or architecture changes
  - `state/accessibility` for UI
  - `equivalence/regression-surface` for refactors
  - `reproduction/test-gap` for bugfixes
  - `capacity/contention` for charts and performance-sensitive work

Every PR must run:

1. layered review
2. revision
3. fix confirmation review
4. full re-review

Do not promote a PR to the next wave while material findings remain open.

## Subagent Operating Rules

- Assign one subagent per PR or per disjoint component cluster.
- Do not let two agents edit the same unstable file set in parallel.
- `floem-showcase/src/main.rs` has a high conflict rate and has one owner per wave.
- `crates/floem-ui/src/components/input.rs`, `select.rs`, `dialog.rs`, and `popover.rs` are high-risk ownership files and should not be edited in parallel unless the plan says the main agent owns integration.
- `crates/floem-charts/*` is a good parallel island only after PR4 establishes the ownership contract and benchmark baseline.
- Review findings always come back to the main agent before integration.
- The main agent owns:
  - showcase structure and navigation
  - crate-root export stitching
  - benchmark baseline capture and comparison
  - wave-level integration and rollback decisions

## First Execution Order

Start here:

1. PR1 `Input Ownership And Minimal Diagnostics Scaffold`
2. review-loop for PR1
3. PR2 and PR4 in parallel if PR1 lands cleanly
4. review-loop for each PR independently
5. PR3
6. PR5
7. continue by wave

## Success Criteria

- `Input` ownership bugs are diagnosable through the showcase diagnostics scaffold and pass the PR1 matrix
- V0 interactions are complete and stable in light/dark and later PRs touching input rerun the matrix
- overlays have explicit semantic ownership, focus return behavior, and pass the overlay matrix
- charts expose interactive ownership through app-owned state and preserve the PR4 benchmark baseline within the declared threshold
- showcase works as both demo and diagnostics harness, and the diagnostics navigation has a repeatable smoke path
- each PR has completed review-loop evidence and the canonical PR gate order

## Residual Risks

- macOS Korean IME remains partly manual to verify even after the PR1 diagnostics scaffold exists
- showcase merge conflicts are likely if wave ownership is not enforced
- `Combobox` remains the highest-risk V2 control and may still require one extra split if hidden overlay/input coupling appears
- chart performance thresholds may need one re-baseline after PR4 if the current benchmark suite is expanded materially
