# floem-toolkit Reactive Contract Process

This document records the recommended development process for `floem-toolkit`
on top of `floem 0.2.0`.

The goal is not to force a foreign architecture onto Floem. The goal is to
organize Floem's existing reactive primitives into a toolkit-friendly process
that works for complex components such as `Input`, overlays, and charts.

## Decision

`floem-toolkit` will use a `reactive contract-first` process built on top of
Floem's existing `signals/effects -> ViewId::update_state -> View::update`
pipeline.

This is closer to a `signal coordinator` model than a classic MVVM framework.

## What Floem Already Provides

Floem already has the core primitives needed for a contract-first workflow.

- App-owned reactive state via `RwSignal` and related signal types.
- Reactive effects via `create_effect` and `create_updater`.
- A documented custom-view update path via `ViewId::update_state(...)` and
  `View::update(...)`.
- Dynamic view projection via `dyn_view` and `dyn_stack`.
- Inbound/outbound value bridging via `value_container`.
- Async and external event bridging via `ext_event`.
- Engine-level tree inspection via `inspector`.

Relevant local upstream references:

- `floem/src/lib.rs`
- `floem/src/view.rs`
- `floem/src/views/value_container.rs`
- `floem/src/views/dyn_view.rs`
- `floem/src/views/dyn_stack.rs`
- `floem/src/ext_event.rs`
- `floem/src/inspector.rs`

## What Floem Does Not Provide

Floem does not give `floem-toolkit` a complete app-owned behavioral contract
for complex widgets.

In particular, Floem does not define:

- a toolkit-level component coordinator abstraction
- a source of truth for cross-widget ownership
- a finished policy for focus and IME ownership in multi-input forms
- a public diagnostic harness for verification

Important constraints from the local Floem source:

- `ViewState` is engine-owned internal state, not app-owned component state.
- IME events are not focus-only events.
- editor IME handling depends on caller-provided `is_active`.
- `text_input` is simple and focus-oriented, but does not provide IME preedit
  handling.
- `text_editor` handles IME, but its default helper assumes a broad
  always-active policy that is not enough for multi-field form input.

Relevant local upstream references:

- `floem/src/view_state.rs`
- `floem/src/context.rs`
- `floem/src/event.rs`
- `floem/src/views/editor/view.rs`
- `floem/src/views/text_input.rs`
- `floem/src/views/text_editor.rs`

## Process Summary

Every non-trivial component must be developed in this order:

1. Define the behavioral contract as app-owned state.
2. Define the coordinator that owns cross-widget policy.
3. Define the widget-local transient state separately.
4. Reuse Floem bridge primitives first, then add a narrow component-specific
   adapter only when needed.
5. Render the public component as a projection of coordinator state.
6. Add a showcase diagnostics panel for the same contract.
7. Verify with unit tests, showcase diagnostics, and native interaction checks.

This process is mandatory for:

- `Input`
- `Dialog`
- `Popover`
- `Select`
- `Tabs`
- chart interaction layers

This process is optional for:

- simple presentational components with no cross-widget state
- pure recipe-driven layout or styling components

## Reuse First

`floem-toolkit` should not create a new abstraction when Floem already provides
an adequate primitive.

Default preference order:

1. direct signals and direct view closures
2. `value_container` and `create_value_container_signals`
3. `dyn_view` and `dyn_stack`
4. `ext_event`
5. component-specific coordinator or adapter

Only introduce a coordinator when there is real policy or ownership spanning
more than one widget or more than one engine concern.

## Contract Layers

Each complex component should be modeled in four layers.

### 1. Public Component Contract

This is the user-facing API contract.

Examples:

- current value
- open/closed state
- selected tab
- hovered series
- disabled state

This contract must be representable as plain Rust values and signals.

### 2. Coordinator State

This is toolkit-owned state for cross-widget policy.

Examples:

- active input field
- focused field
- composing field
- focus return target for overlays
- chart hover owner
- chart brush owner

This state should not live in Floem `ViewState`.

### 3. Widget-Local Transient State

This is view-implementation state that does not define the public contract.

Examples:

- caret blink
- local drag offset in a custom low-level widget
- viewport scroll delta in a custom low-level widget
- pointer anchor during drag in a custom low-level widget

This layer may live inside custom widgets or low-level Floem views.

### 4. Debug State

This is a diagnostics-friendly projection of the contract and coordinator.

Examples:

- current owner ids
- last transition
- current value snapshots
- commit count
- active route log

This state is primarily for `floem-showcase` and tests.

## Recommended Coordinator Types

The coordinator should be small and explicit.

### Input

```rust
struct InputFieldState {
    id: InputFieldId,
    value: String,
    is_focused: bool,
    is_composing: bool,
    last_commit_rev: u64,
}

struct InputCoordinatorState {
    focused_id: Option<InputFieldId>,
    composing_id: Option<InputFieldId>,
    active_id: Option<InputFieldId>,
    last_route_event: Option<InputRouteEvent>,
}
```

### Overlay

```rust
struct OverlayCoordinatorState {
    open_id: Option<OverlayId>,
    focus_return_target: Option<OverlayTriggerId>,
    last_dismiss_reason: Option<OverlayDismissReason>,
}
```

### Charts

```rust
struct ChartInteractionState {
    hovered_series: Option<SeriesId>,
    hovered_datum: Option<DatumId>,
    selected_series: Option<SeriesId>,
    viewport: ChartViewport,
    brush: Option<BrushState>,
}
```

## Diagnostic Harness

`floem-showcase` is not only a demo app. It is the primary diagnostics harness.

Each complex component page should support:

- normal mode
- diagnostics mode

Diagnostics mode should show:

- current public value
- current coordinator state
- current local ownership state
- recent transition log
- any derived active owner

Toolkit diagnostics should expose component contract state.
They should not try to replace Floem Inspector for engine tree, layout, style,
or raw event-routing truth.

For input-like components, diagnostics must expose at least:

- field id
- focused
- composing
- active
- current value
- last commit revision

For overlays:

- open owner
- focus return target
- last dismiss reason

For charts:

- hovered series
- hovered datum
- selected item
- current viewport

## Verification Rules

Every fix for a complex component must have three verification layers.

### 1. State Unit Tests

These test only the coordinator and contract logic.

Examples:

- focus switching
- composing owner retention
- overlay focus return
- chart selection transitions

### 2. Component Integration Checks

These verify that the low-level widget publishes the correct transitions into
the coordinator.

Examples:

- external value update sync
- local edit propagation
- open/close callback sequencing
- hover and selection callbacks

### 3. Showcase Diagnostics Verification

This verifies that native input and event ordering match the contract.

Examples:

- Korean IME composition in multi-input forms
- pointer-driven popover dismiss
- tab switching with keyboard focus
- chart hover and zoom ownership

## When To Use Direct Signal Closures

Use direct signal closures for:

- labels
- simple recipe projection
- read-only presentational views
- straightforward visibility toggles

Examples:

- `label(move || state.get().title.clone())`
- reactive theme text or small badges

## When To Use `update_state`

Use `ViewId::update_state(...)` for:

- custom widgets
- bridged low-level input state
- widgets that need explicit internal mutation
- complex state derived outside the widget

Examples:

- editor-backed input wrappers
- chart canvas or plot views
- custom scrolling or pointer views

If a custom `View` must react to signals, the preferred Floem pattern is:

- read signals in an effect
- push state into the view with `update_state`
- apply it in `View::update`

## When To Use `value_container`

Use a `value_container`-style split when a component needs:

- inbound producer state
- outbound local edits
- `on_update` style observation

This is a good fit for controlled components and diagnostics.

Reuse Floem's `value_container` and `create_value_container_signals` directly
when they are sufficient.

For `floem-toolkit`, this pattern should inform:

- controlled form controls
- wrapper components that expose change events
- diagnostics hooks that observe widget-local changes

Do not introduce a toolkit-wide generic replacement for `value_container`.

## When Not To Build A Coordinator

Do not create a component coordinator if the problem can be solved with:

- a single `RwSignal` and direct reactive projection
- a plain controlled component using `value_container`
- local widget state that does not cross widget boundaries
- a standard Floem dynamic projection via `dyn_view` or `dyn_stack`
- an engine concern already owned by Floem, such as raw pointer and drag
  tracking

Typical cases that do not need a coordinator:

- labels, badges, and read-only presentational views
- simple controlled text or value wrappers with no ownership policy
- visibility toggles with no focus-return or overlay policy
- lists or panels that only need dynamic rendering, not ownership logic

## Anti-Patterns

The following are explicitly out of bounds.

- Treating `ViewState` as app-owned component state.
- Treating `AppState` as component-level source of truth.
- Assuming `focus == IME owner`.
- Assuming `text_input` is a complete IME-safe single-line field.
- Using `text_editor(... |_| true ...)` in multi-field form input without a
  coordinator policy.
- Rebuilding whole view trees on every signal change instead of projecting
  through reactive updates.
- Relying on screenshots alone when state can be surfaced directly.
- Mixing public contract state and widget-local transient state into one ad hoc
  mutable object.

## floem-toolkit Adoption Plan

Adopt this process incrementally.

### Phase 1

- Add a coordinator and diagnostics layer for `Input`.
- Add showcase diagnostics mode for input fields.
- Use the same pattern to close IME and focus ownership bugs.

### Phase 2

- Apply the same process to `Dialog`, `Popover`, and `Select`.
- Standardize overlay dismissal and focus return diagnostics.

### Phase 3

- Apply the same process to chart interaction.
- Expose hover, selection, viewport, and brush ownership in showcase.

## Practical Rule

If a bug involves any of the following:

- focus
- IME
- overlay ownership
- keyboard routing
- pointer capture
- chart interaction ownership

do not patch visuals first.

First define or inspect:

- public contract
- coordinator state
- local transient state
- debug state

Then fix the policy, then verify the projection.
