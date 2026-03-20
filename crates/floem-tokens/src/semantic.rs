use crate::chart::ChartTokens;
use crate::color::{alpha, darken, lighten, ColorScale, ThemeMode};
use crate::motion::MotionScale;
use crate::scales::{ElevationScale, RadiusScale, SpacingScale, TypographyScale};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticColors {
    pub background: ColorScale,
    pub foreground: ColorScale,
    pub surface: ColorScale,
    pub card: ColorScale,
    pub popover: ColorScale,
    pub primary: ColorScale,
    pub secondary: ColorScale,
    pub muted: ColorScale,
    pub accent: ColorScale,
    pub destructive: ColorScale,
    pub border: ColorScale,
    pub input: ColorScale,
    pub ring: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticTokens {
    pub colors: SemanticColors,
    pub spacing: SpacingScale,
    pub radius: RadiusScale,
    pub typography: TypographyScale,
    pub elevation: ElevationScale,
    pub motion: MotionScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSet {
    pub mode: ThemeMode,
    pub semantic: SemanticTokens,
    pub chart: ChartTokens,
}

impl TokenSet {
    pub fn new(mode: ThemeMode) -> Self {
        let semantic = SemanticTokens {
            colors: match mode {
                ThemeMode::Light => SemanticColors {
                    background: ColorScale::rgb(250, 250, 250),
                    foreground: ColorScale::rgb(15, 23, 42),
                    surface: ColorScale::rgb(255, 255, 255),
                    card: ColorScale::rgb(255, 255, 255),
                    popover: ColorScale::rgb(255, 255, 255),
                    primary: ColorScale::rgb(17, 24, 39),
                    secondary: ColorScale::rgb(241, 245, 249),
                    muted: ColorScale::rgb(245, 245, 245),
                    accent: ColorScale::rgb(15, 23, 42),
                    destructive: ColorScale::rgb(220, 38, 38),
                    border: ColorScale::rgb(226, 232, 240),
                    input: ColorScale::rgb(255, 255, 255),
                    ring: ColorScale::rgb(59, 130, 246),
                },
                ThemeMode::Dark => SemanticColors {
                    background: ColorScale::rgb(9, 9, 11),
                    foreground: ColorScale::rgb(250, 250, 250),
                    surface: ColorScale::rgb(24, 24, 27),
                    card: ColorScale::rgb(24, 24, 27),
                    popover: ColorScale::rgb(24, 24, 27),
                    primary: ColorScale::rgb(250, 250, 250),
                    secondary: ColorScale::rgb(39, 39, 42),
                    muted: ColorScale::rgb(39, 39, 42),
                    accent: ColorScale::rgb(39, 39, 42),
                    destructive: ColorScale::rgb(239, 68, 68),
                    border: ColorScale::rgb(63, 63, 70),
                    input: ColorScale::rgb(24, 24, 27),
                    ring: ColorScale::rgb(96, 165, 250),
                },
            },
            spacing: SpacingScale::default(),
            radius: RadiusScale::default(),
            typography: TypographyScale::default(),
            elevation: ElevationScale::default(),
            motion: MotionScale::default(),
        };

        let base_axis = match mode {
            ThemeMode::Light => semantic.colors.foreground,
            ThemeMode::Dark => semantic.colors.foreground,
        };

        let series = vec![
            ColorScale::rgb(59, 130, 246),
            ColorScale::rgb(16, 185, 129),
            ColorScale::rgb(249, 115, 22),
            ColorScale::rgb(168, 85, 247),
            ColorScale::rgb(236, 72, 153),
            ColorScale::rgb(234, 179, 8),
            ColorScale::rgb(20, 184, 166),
            ColorScale::rgb(239, 68, 68),
        ];

        Self {
            mode,
            chart: ChartTokens {
                axis: alpha(base_axis, 0.72),
                grid: alpha(base_axis, 0.18),
                tooltip_bg: semantic.colors.popover,
                tooltip_text: semantic.colors.foreground,
                selection: alpha(semantic.colors.ring, 0.25),
                series,
            },
            semantic,
        }
    }

    pub fn hover_for(&self, color: ColorScale) -> ColorScale {
        match self.mode {
            ThemeMode::Light => darken(color, 0.06),
            ThemeMode::Dark => lighten(color, 0.06),
        }
    }

    pub fn active_for(&self, color: ColorScale) -> ColorScale {
        match self.mode {
            ThemeMode::Light => darken(color, 0.12),
            ThemeMode::Dark => lighten(color, 0.12),
        }
    }

    pub fn muted_for(&self, color: ColorScale) -> ColorScale {
        alpha(color, 0.20)
    }

    pub fn subtle_for(&self, color: ColorScale) -> ColorScale {
        alpha(color, 0.10)
    }
}
