# floem-toolkit Design

This file mirrors the approved architecture for the initial `floem-toolkit`
workspace. It records the design contract that the implementation follows.

## Crates

- `floem-tokens`
- `floem-theme`
- `floem-ui`
- `floem-charts`
- `floem-showcase`

## Core Principles

- Semantic tokens are the design source of truth.
- `ThemeDefinition` is the only persisted theme model.
- Recipes are derived, not stored.
- Stateful UI is controlled-first.
- Charts split pure geometry/state logic from Floem rendering.
- Showcase is both demo app and public API verification harness.

## Initial Deliverables

- Working Cargo workspace
- `floem-tokens` semantic token contract
- `floem-theme` recipe resolution and patching
- V0 UI components and V0 charts
- Showcase pages for foundations, UI, charts, and docs
