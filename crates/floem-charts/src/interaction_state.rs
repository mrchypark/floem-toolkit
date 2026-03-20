use crate::model::{ChartBounds, DatumId, Domain2D, LineChartModel, ProjectedPoint, SeriesId};

const MAX_RECENT_EVENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverSource {
    Plot,
    Legend,
    Tooltip,
}

impl HoverSource {
    fn label(self) -> &'static str {
        match self {
            Self::Plot => "plot",
            Self::Legend => "legend",
            Self::Tooltip => "tooltip",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverTarget {
    pub series_id: SeriesId,
    pub datum_id: Option<DatumId>,
}

impl HoverTarget {
    pub fn series(series_id: SeriesId) -> Self {
        Self {
            series_id,
            datum_id: None,
        }
    }

    pub fn datum(datum_id: DatumId) -> Self {
        Self {
            series_id: datum_id.series_id,
            datum_id: Some(datum_id),
        }
    }

    fn describe(self) -> String {
        match self.datum_id {
            Some(datum_id) => format!(
                "series:{} datum:{}",
                self.series_id.get(),
                datum_id.point_index
            ),
            None => format!("series:{}", self.series_id.get()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverState {
    pub source: HoverSource,
    pub target: HoverTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedChartItem {
    Series(SeriesId),
    Datum(DatumId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionState {
    pub item: Option<SelectedChartItem>,
}

impl SelectionState {
    pub fn single(series_index: usize) -> Self {
        Self {
            item: Some(SelectedChartItem::Series(SeriesId::new(series_index))),
        }
    }

    pub fn contains(&self, series_index: usize) -> bool {
        match self.item {
            Some(SelectedChartItem::Series(series_id)) => series_id.get() == series_index,
            Some(SelectedChartItem::Datum(datum_id)) => datum_id.series_id.get() == series_index,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportState {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomState {
    pub scale_x: f64,
    pub scale_y: f64,
}

impl ViewportState {
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn from_domain(domain: Domain2D) -> Self {
        Self::new(domain.x_min, domain.x_max, domain.y_min, domain.y_max)
    }

    pub fn as_domain(&self) -> Domain2D {
        Domain2D {
            x_min: self.x_min,
            x_max: self.x_max,
            y_min: self.y_min,
            y_max: self.y_max,
        }
    }

    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }

    pub fn intersects_domain(&self, domain: Domain2D) -> bool {
        self.x_min <= domain.x_max
            && self.x_max >= domain.x_min
            && self.y_min <= domain.y_max
            && self.y_max >= domain.y_min
    }

    pub fn zoomed(&self, zoom: ZoomState) -> Self {
        let center_x = (self.x_min + self.x_max) / 2.0;
        let center_y = (self.y_min + self.y_max) / 2.0;
        let next_width = self.width() / zoom.scale_x.max(f64::EPSILON);
        let next_height = self.height() / zoom.scale_y.max(f64::EPSILON);
        Self {
            x_min: center_x - (next_width / 2.0),
            x_max: center_x + (next_width / 2.0),
            y_min: center_y - (next_height / 2.0),
            y_max: center_y + (next_height / 2.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartInteractionState {
    plot_hover: Option<HoverTarget>,
    legend_hover: Option<HoverTarget>,
    tooltip_hover: Option<HoverTarget>,
    selection: SelectionState,
    viewport: Option<ViewportState>,
    recent_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartInteractionDebugSnapshot {
    pub hovered_series: Option<SeriesId>,
    pub hovered_datum: Option<DatumId>,
    pub hover_source: Option<HoverSource>,
    pub selected_item: Option<SelectedChartItem>,
    pub viewport: Option<ViewportState>,
    pub tooltip_anchor: Option<ProjectedPoint>,
    pub selection_anchor: Option<ProjectedPoint>,
    pub recent_events: Vec<String>,
}

impl ChartInteractionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hovered(&self) -> Option<HoverState> {
        self.tooltip_hover
            .map(|target| HoverState {
                source: HoverSource::Tooltip,
                target,
            })
            .or_else(|| {
                self.legend_hover.map(|target| HoverState {
                    source: HoverSource::Legend,
                    target,
                })
            })
            .or_else(|| {
                self.plot_hover.map(|target| HoverState {
                    source: HoverSource::Plot,
                    target,
                })
            })
    }

    pub fn hovered_series(&self) -> Option<SeriesId> {
        self.hovered().map(|state| state.target.series_id)
    }

    pub fn hovered_datum(&self) -> Option<DatumId> {
        self.hovered().and_then(|state| state.target.datum_id)
    }

    pub fn selected_item(&self) -> Option<SelectedChartItem> {
        self.selection.item
    }

    pub fn set_hover(&mut self, source: HoverSource, target: Option<HoverTarget>) {
        let slot = match source {
            HoverSource::Plot => &mut self.plot_hover,
            HoverSource::Legend => &mut self.legend_hover,
            HoverSource::Tooltip => &mut self.tooltip_hover,
        };
        if *slot == target {
            return;
        }

        *slot = target;
        match target {
            Some(target) => {
                self.push_event(format!("hover:{}:{}", source.label(), target.describe()))
            }
            None => self.push_event(format!("hover:{}:clear", source.label())),
        }
    }

    pub fn set_selection(&mut self, item: Option<SelectedChartItem>) {
        if self.selection.item == item {
            return;
        }

        self.selection.item = item;
        match item {
            Some(SelectedChartItem::Series(series_id)) => {
                self.push_event(format!("selection:series:{}", series_id.get()))
            }
            Some(SelectedChartItem::Datum(datum_id)) => self.push_event(format!(
                "selection:datum:{}:{}",
                datum_id.series_id.get(),
                datum_id.point_index
            )),
            None => self.push_event("selection:clear".to_string()),
        }
    }

    pub fn set_viewport(&mut self, viewport: Option<ViewportState>) {
        if self.viewport == viewport {
            return;
        }

        self.viewport = viewport;
        match viewport {
            Some(viewport) => self.push_event(format!(
                "viewport:{:.2},{:.2},{:.2},{:.2}",
                viewport.x_min, viewport.x_max, viewport.y_min, viewport.y_max
            )),
            None => self.push_event("viewport:reset".to_string()),
        }
    }

    pub fn effective_viewport(&self, model: &LineChartModel) -> Option<ViewportState> {
        self.viewport
            .or_else(|| model.domain().map(ViewportState::from_domain))
    }

    pub fn hover_projection(
        &self,
        model: &LineChartModel,
        bounds: ChartBounds,
    ) -> Option<ProjectedPoint> {
        let viewport = self.effective_viewport(model)?;
        let hovered = self.hovered()?;
        match hovered.target.datum_id {
            Some(datum_id) => model.project_datum(bounds, viewport.as_domain(), datum_id),
            None => model.series_anchor(bounds, viewport.as_domain(), hovered.target.series_id),
        }
    }

    pub fn selection_projection(
        &self,
        model: &LineChartModel,
        bounds: ChartBounds,
    ) -> Option<ProjectedPoint> {
        let viewport = self.effective_viewport(model)?;
        match self.selection.item? {
            SelectedChartItem::Series(series_id) => {
                model.series_anchor(bounds, viewport.as_domain(), series_id)
            }
            SelectedChartItem::Datum(datum_id) => {
                model.project_datum(bounds, viewport.as_domain(), datum_id)
            }
        }
    }

    pub fn debug_snapshot(
        &self,
        model: &LineChartModel,
        bounds: ChartBounds,
    ) -> ChartInteractionDebugSnapshot {
        ChartInteractionDebugSnapshot {
            hovered_series: self.hovered_series(),
            hovered_datum: self.hovered_datum(),
            hover_source: self.hovered().map(|state| state.source),
            selected_item: self.selection.item,
            viewport: self.effective_viewport(model),
            tooltip_anchor: self.hover_projection(model, bounds),
            selection_anchor: self.selection_projection(model, bounds),
            recent_events: self.recent_events.clone(),
        }
    }

    pub fn sync_with_model(&mut self, model: &LineChartModel) {
        if !Self::hover_target_is_valid(model, self.plot_hover) {
            self.plot_hover = None;
            self.push_event("sync:hover:plot:cleared".to_string());
        }
        if !Self::hover_target_is_valid(model, self.legend_hover) {
            self.legend_hover = None;
            self.push_event("sync:hover:legend:cleared".to_string());
        }
        if !Self::hover_target_is_valid(model, self.tooltip_hover) {
            self.tooltip_hover = None;
            self.push_event("sync:hover:tooltip:cleared".to_string());
        }

        let selection_invalid = self
            .selection
            .item
            .is_some_and(|item| !Self::selection_is_valid(model, item));
        if selection_invalid {
            self.selection.item = None;
            self.viewport = None;
            self.push_event("sync:selection:cleared".to_string());
            self.push_event("sync:viewport:reset".to_string());
        }

        let viewport_invalid = match (self.viewport, model.domain()) {
            (Some(_), None) => true,
            (Some(viewport), Some(domain)) => !viewport.intersects_domain(domain),
            (None, _) => false,
        };
        if viewport_invalid {
            self.viewport = None;
            self.push_event("sync:viewport:reset".to_string());
        }
    }

    fn hover_target_is_valid(model: &LineChartModel, target: Option<HoverTarget>) -> bool {
        match target {
            Some(target) => match target.datum_id {
                Some(datum_id) => model.contains_datum(datum_id),
                None => model.contains_series(target.series_id),
            },
            None => true,
        }
    }

    fn selection_is_valid(model: &LineChartModel, item: SelectedChartItem) -> bool {
        match item {
            SelectedChartItem::Series(series_id) => model.contains_series(series_id),
            SelectedChartItem::Datum(datum_id) => model.contains_datum(datum_id),
        }
    }

    fn push_event(&mut self, event: String) {
        self.recent_events.push(event);
        if self.recent_events.len() > MAX_RECENT_EVENTS {
            let extra = self.recent_events.len() - MAX_RECENT_EVENTS;
            self.recent_events.drain(0..extra);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChartBounds, DatumId, LineChartModel, LineDatum, LineSeries, SeriesId};

    #[test]
    fn viewport_zoom_preserves_center() {
        let viewport = ViewportState::new(0.0, 100.0, 0.0, 50.0);
        let zoomed = viewport.zoomed(ZoomState {
            scale_x: 2.0,
            scale_y: 5.0,
        });
        assert_eq!(zoomed.width(), 50.0);
        assert_eq!(zoomed.height(), 10.0);
        assert_eq!((zoomed.x_min + zoomed.x_max) / 2.0, 50.0);
        assert_eq!((zoomed.y_min + zoomed.y_max) / 2.0, 25.0);
    }

    #[test]
    fn selection_contains_selected_series() {
        let selection = SelectionState::single(3);
        assert!(selection.contains(3));
        assert!(!selection.contains(1));
    }

    #[test]
    fn hover_precedence_is_explicit_across_sources() {
        let mut state = ChartInteractionState::new();
        let plot_datum = DatumId::new(SeriesId::new(1), 2);

        state.set_hover(HoverSource::Plot, Some(HoverTarget::datum(plot_datum)));
        assert_eq!(
            state.hovered(),
            Some(HoverState {
                source: HoverSource::Plot,
                target: HoverTarget::datum(plot_datum),
            })
        );

        state.set_hover(
            HoverSource::Legend,
            Some(HoverTarget::series(SeriesId::new(4))),
        );
        assert_eq!(
            state.hovered(),
            Some(HoverState {
                source: HoverSource::Legend,
                target: HoverTarget::series(SeriesId::new(4)),
            })
        );

        state.set_hover(
            HoverSource::Tooltip,
            Some(HoverTarget::series(SeriesId::new(6))),
        );
        assert_eq!(
            state.hovered(),
            Some(HoverState {
                source: HoverSource::Tooltip,
                target: HoverTarget::series(SeriesId::new(6)),
            })
        );

        state.set_hover(HoverSource::Tooltip, None);
        assert_eq!(
            state.hovered(),
            Some(HoverState {
                source: HoverSource::Legend,
                target: HoverTarget::series(SeriesId::new(4)),
            })
        );
    }

    #[test]
    fn debug_snapshot_reprojects_tooltip_anchor_after_viewport_changes() {
        let model = LineChartModel::new(vec![LineSeries::new(
            SeriesId::new(0),
            vec![
                LineDatum { x: 0.0, y: 0.0 },
                LineDatum { x: 2.5, y: 7.5 },
                LineDatum { x: 10.0, y: 10.0 },
            ],
        )]);
        let datum_id = DatumId::new(SeriesId::new(0), 1);
        let bounds = ChartBounds {
            width: 100.0,
            height: 100.0,
        };
        let mut state = ChartInteractionState::new();

        state.set_hover(HoverSource::Plot, Some(HoverTarget::datum(datum_id)));
        let initial = state.debug_snapshot(&model, bounds);
        assert_eq!(
            initial.tooltip_anchor,
            Some(crate::model::ProjectedPoint { x: 25.0, y: 25.0 })
        );

        state.set_viewport(Some(ViewportState::new(0.0, 5.0, 5.0, 10.0)));
        let zoomed = state.debug_snapshot(&model, bounds);
        assert_eq!(
            zoomed.tooltip_anchor,
            Some(crate::model::ProjectedPoint { x: 50.0, y: 50.0 })
        );
    }

    #[test]
    fn hover_projection_falls_back_to_series_anchor() {
        let model = LineChartModel::new(vec![LineSeries::new(
            SeriesId::new(2),
            vec![LineDatum { x: 0.0, y: 0.0 }, LineDatum { x: 10.0, y: 10.0 }],
        )]);
        let bounds = ChartBounds {
            width: 100.0,
            height: 100.0,
        };
        let mut state = ChartInteractionState::new();

        state.set_hover(
            HoverSource::Legend,
            Some(HoverTarget::series(SeriesId::new(2))),
        );

        assert_eq!(
            state.hover_projection(&model, bounds),
            Some(crate::model::ProjectedPoint { x: 50.0, y: 50.0 })
        );
    }

    #[test]
    fn sync_with_model_clears_missing_refs_and_resets_viewport() {
        let bounds = ChartBounds {
            width: 120.0,
            height: 80.0,
        };
        let original = LineChartModel::new(vec![
            LineSeries::new(
                SeriesId::new(0),
                vec![LineDatum { x: 0.0, y: 0.0 }, LineDatum { x: 1.0, y: 2.0 }],
            ),
            LineSeries::new(
                SeriesId::new(1),
                vec![LineDatum { x: 2.0, y: 1.0 }, LineDatum { x: 3.0, y: 3.0 }],
            ),
        ]);
        let updated = LineChartModel::new(vec![LineSeries::new(
            SeriesId::new(0),
            vec![LineDatum { x: 0.0, y: 0.0 }, LineDatum { x: 1.0, y: 2.0 }],
        )]);
        let mut state = ChartInteractionState::new();
        let missing_datum = DatumId::new(SeriesId::new(1), 1);

        state.set_hover(HoverSource::Plot, Some(HoverTarget::datum(missing_datum)));
        state.set_selection(Some(SelectedChartItem::Datum(missing_datum)));
        state.set_viewport(Some(ViewportState::new(10.0, 20.0, 10.0, 20.0)));
        assert_eq!(
            state.debug_snapshot(&original, bounds).selected_item,
            Some(SelectedChartItem::Datum(missing_datum))
        );

        state.sync_with_model(&updated);

        let snapshot = state.debug_snapshot(&updated, bounds);
        assert_eq!(snapshot.hovered_series, None);
        assert_eq!(snapshot.hovered_datum, None);
        assert_eq!(snapshot.selected_item, None);
        assert_eq!(
            snapshot.viewport,
            Some(ViewportState::from_domain(
                updated.domain().expect("domain")
            ))
        );
    }
}
