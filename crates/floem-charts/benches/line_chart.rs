use criterion::{black_box, criterion_group, criterion_main, Criterion};
use floem_charts::interaction_state::{
    ChartInteractionState, HoverSource, HoverTarget, SelectedChartItem, ViewportState,
};
use floem_charts::model::{ChartBounds, LineChartModel, LineDatum, LineSeries, SeriesId};
use floem_charts::scale::{nearest_point, nearest_point_across_series, LinearScale};

fn bench_placeholder(c: &mut Criterion) {
    let scale = LinearScale::new(0.0, 10_000.0, 0.0, 1024.0);
    let points: Vec<_> = (0..4096)
        .map(|i| (i as f64, ((i as f64) / 3.0).sin() * 100.0))
        .collect();
    let series: Vec<_> = (0..8)
        .map(|series_idx| {
            (0..1024)
                .map(|point_idx| {
                    let x = point_idx as f64;
                    let y = ((x / 12.0) + series_idx as f64).sin() * 90.0 + series_idx as f64;
                    (x, y)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let line_model = LineChartModel::new(
        (0..8)
            .map(|series_idx| {
                let data = (0..1024)
                    .map(|point_idx| {
                        let x = point_idx as f64;
                        let y = ((x / 12.0) + series_idx as f64).sin() * 90.0 + series_idx as f64;
                        LineDatum { x, y }
                    })
                    .collect::<Vec<_>>();
                LineSeries::new(SeriesId::new(series_idx), data)
            })
            .collect(),
    );
    let bounds = ChartBounds {
        width: 1024.0,
        height: 480.0,
    };
    let viewport = ViewportState::from_domain(line_model.domain().expect("domain"));
    let mut hover_state = ChartInteractionState::new();
    hover_state.set_viewport(Some(viewport));
    hover_state.set_hover(
        HoverSource::Legend,
        Some(HoverTarget::series(SeriesId::new(4))),
    );
    let mut selection_state = ChartInteractionState::new();
    selection_state.set_viewport(Some(viewport));
    selection_state.set_selection(Some(SelectedChartItem::Series(SeriesId::new(4))));

    c.bench_function("scale_mapping_4096", |b| {
        b.iter(|| {
            for i in 0..4096 {
                black_box(scale.scale(i as f64));
            }
        })
    });

    c.bench_function("nearest_point_lookup_4096", |b| {
        b.iter(|| black_box(nearest_point(&points, (2048.0, 0.0))))
    });

    c.bench_function("nearest_point_lookup_multi_series_8x1024", |b| {
        b.iter(|| black_box(nearest_point_across_series(&series, (512.0, 8.0))))
    });

    c.bench_function("hover_projection_8x1024", |b| {
        b.iter(|| {
            black_box(hover_state.hover_projection(black_box(&line_model), black_box(bounds)))
        })
    });

    c.bench_function("selection_projection_8x1024", |b| {
        b.iter(|| {
            black_box(
                selection_state.selection_projection(black_box(&line_model), black_box(bounds)),
            )
        })
    });
}

criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
