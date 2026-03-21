use crate::scale::LinearScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeriesId(usize);

impl SeriesId {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DatumId {
    pub series_id: SeriesId,
    pub point_index: usize,
}

impl DatumId {
    pub const fn new(series_id: SeriesId, point_index: usize) -> Self {
        Self {
            series_id,
            point_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineDatum {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarDatum {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterDatum {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartBounds {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Domain2D {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineConfig {
    pub bounds: ChartBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarConfig {
    pub bounds: ChartBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterConfig {
    pub bounds: ChartBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSeries {
    pub id: SeriesId,
    pub data: Vec<LineDatum>,
}

impl LineSeries {
    pub fn new(id: SeriesId, data: Vec<LineDatum>) -> Self {
        Self { id, data }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LineChartModel {
    pub series: Vec<LineSeries>,
}

impl LineChartModel {
    pub fn new(series: Vec<LineSeries>) -> Self {
        Self { series }
    }

    pub fn domain(&self) -> Option<Domain2D> {
        let mut raw = self
            .series
            .iter()
            .flat_map(|series| series.data.iter().map(|datum| (datum.x, datum.y)));
        let first = raw.next()?;
        let mut domain = Domain2D {
            x_min: first.0,
            x_max: first.0,
            y_min: first.1,
            y_max: first.1,
        };
        for point in raw {
            domain.x_min = domain.x_min.min(point.0);
            domain.x_max = domain.x_max.max(point.0);
            domain.y_min = domain.y_min.min(point.1);
            domain.y_max = domain.y_max.max(point.1);
        }
        Some(domain)
    }

    pub fn contains_series(&self, series_id: SeriesId) -> bool {
        self.series.iter().any(|series| series.id == series_id)
    }

    pub fn contains_datum(&self, datum_id: DatumId) -> bool {
        self.series(datum_id.series_id)
            .and_then(|series| series.data.get(datum_id.point_index))
            .is_some()
    }

    pub fn project_datum(
        &self,
        bounds: ChartBounds,
        domain: Domain2D,
        datum_id: DatumId,
    ) -> Option<ProjectedPoint> {
        let datum = self
            .series(datum_id.series_id)?
            .data
            .get(datum_id.point_index)
            .copied()?;
        Some(domain.project(bounds, (datum.x, datum.y)))
    }

    pub fn series_anchor(
        &self,
        bounds: ChartBounds,
        domain: Domain2D,
        series_id: SeriesId,
    ) -> Option<ProjectedPoint> {
        let projected = self.project_series(bounds, domain, series_id)?;
        let mut points = projected.iter();
        let first = *points.next()?;
        let mut min_x = first.x;
        let mut max_x = first.x;
        let mut min_y = first.y;
        let mut max_y = first.y;
        for point in points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        Some(ProjectedPoint {
            x: (min_x + max_x) / 2.0,
            y: (min_y + max_y) / 2.0,
        })
    }

    pub fn project_series(
        &self,
        bounds: ChartBounds,
        domain: Domain2D,
        series_id: SeriesId,
    ) -> Option<Vec<ProjectedPoint>> {
        let series = self.series(series_id)?;
        Some(
            series
                .data
                .iter()
                .map(|datum| domain.project(bounds, (datum.x, datum.y)))
                .collect(),
        )
    }

    fn series(&self, series_id: SeriesId) -> Option<&LineSeries> {
        self.series.iter().find(|series| series.id == series_id)
    }
}

impl Domain2D {
    pub fn from_xy<'a>(points: impl IntoIterator<Item = &'a (f64, f64)>) -> Option<Self> {
        let mut iter = points.into_iter();
        let first = iter.next()?;
        let mut domain = Self {
            x_min: first.0,
            x_max: first.0,
            y_min: first.1,
            y_max: first.1,
        };
        for point in iter {
            domain.x_min = domain.x_min.min(point.0);
            domain.x_max = domain.x_max.max(point.0);
            domain.y_min = domain.y_min.min(point.1);
            domain.y_max = domain.y_max.max(point.1);
        }
        Some(domain)
    }

    pub fn project(&self, bounds: ChartBounds, point: (f64, f64)) -> ProjectedPoint {
        let x_scale = LinearScale::new(self.x_min, self.x_max, 0.0, bounds.width);
        let y_scale = LinearScale::new(self.y_min, self.y_max, bounds.height, 0.0);
        ProjectedPoint {
            x: x_scale.scale(point.0),
            y: y_scale.scale(point.1),
        }
    }
}

impl LineConfig {
    pub fn project(&self, data: &[LineDatum]) -> Vec<ProjectedPoint> {
        let raw = data
            .iter()
            .map(|datum| (datum.x, datum.y))
            .collect::<Vec<_>>();
        let Some(domain) = Domain2D::from_xy(raw.iter()) else {
            return Vec::new();
        };
        raw.into_iter()
            .map(|point| domain.project(self.bounds, point))
            .collect()
    }
}

impl BarConfig {
    pub fn project(&self, data: &[BarDatum]) -> Vec<ProjectedPoint> {
        let raw = data
            .iter()
            .map(|datum| (datum.x, datum.y))
            .collect::<Vec<_>>();
        let Some(domain) = Domain2D::from_xy(raw.iter()) else {
            return Vec::new();
        };
        raw.into_iter()
            .map(|point| domain.project(self.bounds, point))
            .collect()
    }
}

impl ScatterConfig {
    pub fn project(&self, data: &[ScatterDatum]) -> Vec<ProjectedPoint> {
        let raw = data
            .iter()
            .map(|datum| (datum.x, datum.y))
            .collect::<Vec<_>>();
        let Some(domain) = Domain2D::from_xy(raw.iter()) else {
            return Vec::new();
        };
        raw.into_iter()
            .map(|point| domain.project(self.bounds, point))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_tracks_extents() {
        let points = vec![(2.0, -1.0), (5.0, 7.0), (-3.0, 4.0)];
        let domain = Domain2D::from_xy(points.iter()).expect("domain");
        assert_eq!(domain.x_min, -3.0);
        assert_eq!(domain.x_max, 5.0);
        assert_eq!(domain.y_min, -1.0);
        assert_eq!(domain.y_max, 7.0);
    }

    #[test]
    fn line_config_projects_into_bounds() {
        let config = LineConfig {
            bounds: ChartBounds {
                width: 300.0,
                height: 120.0,
            },
        };
        let projected =
            config.project(&[LineDatum { x: 0.0, y: 0.0 }, LineDatum { x: 10.0, y: 10.0 }]);
        assert_eq!(projected[0], ProjectedPoint { x: 0.0, y: 120.0 });
        assert_eq!(projected[1], ProjectedPoint { x: 300.0, y: 0.0 });
    }

    #[test]
    fn line_chart_model_projects_specific_datum() {
        let model = LineChartModel::new(vec![LineSeries::new(
            SeriesId::new(7),
            vec![LineDatum { x: 0.0, y: 0.0 }, LineDatum { x: 10.0, y: 10.0 }],
        )]);
        let point = model.project_datum(
            ChartBounds {
                width: 300.0,
                height: 120.0,
            },
            model.domain().expect("domain"),
            DatumId::new(SeriesId::new(7), 1),
        );

        assert_eq!(point, Some(ProjectedPoint { x: 300.0, y: 0.0 }));
    }

    #[test]
    fn series_anchor_centers_projected_extents() {
        let model = LineChartModel::new(vec![LineSeries::new(
            SeriesId::new(3),
            vec![LineDatum { x: 0.0, y: 0.0 }, LineDatum { x: 10.0, y: 10.0 }],
        )]);
        let anchor = model.series_anchor(
            ChartBounds {
                width: 300.0,
                height: 120.0,
            },
            model.domain().expect("domain"),
            SeriesId::new(3),
        );

        assert_eq!(anchor, Some(ProjectedPoint { x: 150.0, y: 60.0 }));
    }
}
