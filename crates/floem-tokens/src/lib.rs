mod chart;
mod color;
mod motion;
mod scales;
mod semantic;

pub use chart::ChartTokens;
pub use color::{alpha, darken, lighten, with_alpha, ColorScale, ThemeMode};
pub use motion::MotionScale;
pub use scales::{ElevationScale, RadiusScale, SpacingScale, TypographyScale};
pub use semantic::{SemanticColors, SemanticTokens, TokenSet};

#[cfg(test)]
mod tests {
    use crate::{ThemeMode, TokenSet};

    #[test]
    fn token_sets_expose_chart_series() {
        let tokens = TokenSet::new(ThemeMode::Dark);
        assert_eq!(tokens.chart.series.len(), 8);
    }

    #[test]
    fn derivation_helpers_are_mode_sensitive() {
        let light = TokenSet::new(ThemeMode::Light);
        let dark = TokenSet::new(ThemeMode::Dark);
        let base = light.semantic.colors.primary;
        assert_ne!(light.hover_for(base), dark.hover_for(base));
        assert_ne!(light.active_for(base), dark.active_for(base));
    }
}
