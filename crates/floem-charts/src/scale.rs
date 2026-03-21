#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    pub domain_min: f64,
    pub domain_max: f64,
    pub range_min: f64,
    pub range_max: f64,
}

impl LinearScale {
    pub fn new(domain_min: f64, domain_max: f64, range_min: f64, range_max: f64) -> Self {
        Self {
            domain_min,
            domain_max,
            range_min,
            range_max,
        }
    }

    pub fn scale(&self, value: f64) -> f64 {
        let span = self.domain_max - self.domain_min;
        if span.abs() < f64::EPSILON {
            return self.range_min;
        }
        let t = (value - self.domain_min) / span;
        self.range_min + t * (self.range_max - self.range_min)
    }

    pub fn invert(&self, value: f64) -> f64 {
        let span = self.range_max - self.range_min;
        if span.abs() < f64::EPSILON {
            return self.domain_min;
        }
        let t = (value - self.range_min) / span;
        self.domain_min + t * (self.domain_max - self.domain_min)
    }
}

pub fn nearest_point(points: &[(f64, f64)], target: (f64, f64)) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (a.0 - target.0).powi(2) + (a.1 - target.1).powi(2);
            let db = (b.0 - target.0).powi(2) + (b.1 - target.1).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
}

pub fn nearest_point_across_series(
    series: &[Vec<(f64, f64)>],
    target: (f64, f64),
) -> Option<(usize, usize)> {
    series
        .iter()
        .enumerate()
        .flat_map(|(series_idx, points)| {
            points.iter().enumerate().map(move |(point_idx, point)| {
                let distance = (point.0 - target.0).powi(2) + (point.1 - target.1).powi(2);
                (series_idx, point_idx, distance)
            })
        })
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(series_idx, point_idx, _)| (series_idx, point_idx))
}

#[cfg(test)]
mod tests {
    use super::{nearest_point, nearest_point_across_series, LinearScale};

    #[test]
    fn linear_scale_round_trip() {
        let scale = LinearScale::new(0.0, 100.0, 0.0, 400.0);
        assert_eq!(scale.scale(25.0), 100.0);
        assert_eq!(scale.invert(100.0), 25.0);
    }

    #[test]
    fn nearest_point_returns_closest_index() {
        let points = [(0.0, 0.0), (2.0, 2.0), (8.0, 8.0)];
        assert_eq!(nearest_point(&points, (1.8, 1.9)), Some(1));
    }

    #[test]
    fn nearest_point_across_series_returns_series_and_index() {
        let series = vec![vec![(0.0, 0.0), (1.0, 1.0)], vec![(5.0, 5.0), (6.0, 6.0)]];
        assert_eq!(
            nearest_point_across_series(&series, (5.1, 4.9)),
            Some((1, 0))
        );
    }
}
