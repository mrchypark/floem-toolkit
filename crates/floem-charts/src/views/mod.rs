//! Floem chart views live here.

use crate::interaction_state::ChartInteractionDebugSnapshot;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views;

fn format_series(series_id: Option<crate::model::SeriesId>) -> String {
    series_id
        .map(|series_id| series_id.get().to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_datum(datum_id: Option<crate::model::DatumId>) -> String {
    datum_id
        .map(|datum_id| format!("{}:{}", datum_id.series_id.get(), datum_id.point_index))
        .unwrap_or_else(|| "none".to_string())
}

fn format_viewport(viewport: Option<crate::interaction_state::ViewportState>) -> String {
    viewport
        .map(|viewport| {
            format!(
                "{:.1}..{:.1} / {:.1}..{:.1}",
                viewport.x_min, viewport.x_max, viewport.y_min, viewport.y_max
            )
        })
        .unwrap_or_else(|| "none".to_string())
}

fn format_point(point: Option<crate::model::ProjectedPoint>) -> String {
    point
        .map(|point| format!("{:.1}, {:.1}", point.x, point.y))
        .unwrap_or_else(|| "none".to_string())
}

pub fn interaction_diagnostics_view(
    snapshot: RwSignal<ChartInteractionDebugSnapshot>,
) -> impl IntoView {
    (
        views::label(move || {
            format!(
                "Hovered series: {}",
                format_series(snapshot.get().hovered_series)
            )
        }),
        views::label(move || {
            format!(
                "Hovered datum: {}",
                format_datum(snapshot.get().hovered_datum)
            )
        }),
        views::label(move || {
            format!(
                "Hover source: {}",
                snapshot
                    .get()
                    .hover_source
                    .map(|source| format!("{source:?}"))
                    .unwrap_or_else(|| "none".to_string())
            )
        }),
        views::label(move || {
            format!(
                "Selected item: {}",
                snapshot
                    .get()
                    .selected_item
                    .map(|item| format!("{item:?}"))
                    .unwrap_or_else(|| "none".to_string())
            )
        }),
        views::label(move || format!("Viewport: {}", format_viewport(snapshot.get().viewport))),
        views::label(move || {
            format!(
                "Tooltip anchor: {}",
                format_point(snapshot.get().tooltip_anchor)
            )
        }),
        views::label(move || {
            format!(
                "Selection anchor: {}",
                format_point(snapshot.get().selection_anchor)
            )
        }),
        views::label(move || {
            let snapshot = snapshot.get();
            let recent_events = if snapshot.recent_events.is_empty() {
                "none".to_string()
            } else {
                snapshot.recent_events.join(" | ")
            };
            format!("Recent events: {recent_events}")
        }),
    )
        .v_stack()
        .style(|s| s.row_gap(6.0).line_height(1.35))
}
