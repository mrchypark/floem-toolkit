use floem_tokens::{ColorScale, ThemeMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorDerivation {
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticExceptions {
    pub background: Option<ColorScale>,
    pub foreground: Option<ColorScale>,
    pub surface: Option<ColorScale>,
    pub card: Option<ColorScale>,
    pub popover: Option<ColorScale>,
    pub primary: Option<ColorScale>,
    pub secondary: Option<ColorScale>,
    pub muted: Option<ColorScale>,
    pub accent: Option<ColorScale>,
    pub destructive: Option<ColorScale>,
    pub border: Option<ColorScale>,
    pub input: Option<ColorScale>,
    pub ring: Option<ColorScale>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub mode: ThemeMode,
    pub exceptions: SemanticExceptions,
    pub chart_series: Vec<ColorScale>,
    pub color_derivation: ColorDerivation,
}

impl ThemeDefinition {
    pub fn new(mode: ThemeMode) -> Self {
        Self {
            mode,
            chart_series: floem_tokens::TokenSet::new(mode).chart.series.clone(),
            exceptions: SemanticExceptions {
                background: None,
                foreground: None,
                surface: None,
                card: None,
                popover: None,
                primary: None,
                secondary: None,
                muted: None,
                accent: None,
                destructive: None,
                border: None,
                input: None,
                ring: None,
            },
            color_derivation: ColorDerivation::Hybrid,
        }
    }
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        Self::new(ThemeMode::Dark)
    }
}
