# floem-toolkit

`floem-toolkit` is a `Floem`-native design system workspace inspired by `shadcn/ui`
for visual language and `gpui-toolkit` for crate organization and builder-oriented APIs.

## Workspace

- `floem-tokens`: raw scales, semantic roles, derivation helpers
- `floem-theme`: serializable `ThemeDefinition`, patching, resolved recipes, theme providers
- `floem-ui`: primitives, base layers, and UI components
- `floem-charts`: pure chart core plus Floem rendering adapters
- `floem-showcase`: showcase app and documentation host

## Status

This repository is intentionally built from scratch around `floem = 0.2.0`.
The initial implementation focuses on a stable workspace, theme contract,
v0 components, v0 charts, and a showcase that exercises the public APIs.

Current implemented UI surface:

- `Button`
- `Input`
- `Label`
- `Card`
- `Checkbox`
- `Tabs`
- `Dialog`
- `Popover`
- `Select`

Current chart foundation:

- `LinearScale`
- nearest-point lookup
- multi-series nearest-point lookup benchmark

## Run

```bash
cargo run -p floem-showcase
```

## Benchmarks

```bash
cargo bench -p floem-charts --bench line_chart -- --noplot
```

Latest local benchmark snapshot on 2026-03-10:

- `scale_mapping_4096`: about `4.69µs`
- `nearest_point_lookup_4096`: about `29.6µs`
- `nearest_point_lookup_multi_series_8x1024`: about `11.4µs`
