use crate::theme::{ColorDerivation, ThemeDefinition};
use floem_tokens::{ColorScale, ThemeMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Patch<T> {
    Keep,
    Set(T),
    Clear,
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Self::Keep
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemePatch {
    pub mode: Option<ThemeMode>,
    pub color_derivation: Option<ColorDerivation>,
    pub background: Patch<ColorScale>,
    pub foreground: Patch<ColorScale>,
    pub surface: Patch<ColorScale>,
    pub card: Patch<ColorScale>,
    pub popover: Patch<ColorScale>,
    pub primary: Patch<ColorScale>,
    pub secondary: Patch<ColorScale>,
    pub muted: Patch<ColorScale>,
    pub accent: Patch<ColorScale>,
    pub destructive: Patch<ColorScale>,
    pub border: Patch<ColorScale>,
    pub input: Patch<ColorScale>,
    pub ring: Patch<ColorScale>,
    pub chart_series: Option<Vec<ColorScale>>,
}

pub fn merge_theme(base: &ThemeDefinition, patch: &ThemePatch) -> ThemeDefinition {
    let mut merged = base.clone();
    if let Some(mode) = patch.mode {
        merged.mode = mode;
    }
    if let Some(color_derivation) = patch.color_derivation {
        merged.color_derivation = color_derivation;
    }
    apply_patch_value(&mut merged.exceptions.background, &patch.background);
    apply_patch_value(&mut merged.exceptions.foreground, &patch.foreground);
    apply_patch_value(&mut merged.exceptions.surface, &patch.surface);
    apply_patch_value(&mut merged.exceptions.card, &patch.card);
    apply_patch_value(&mut merged.exceptions.popover, &patch.popover);
    apply_patch_value(&mut merged.exceptions.primary, &patch.primary);
    apply_patch_value(&mut merged.exceptions.secondary, &patch.secondary);
    apply_patch_value(&mut merged.exceptions.muted, &patch.muted);
    apply_patch_value(&mut merged.exceptions.accent, &patch.accent);
    apply_patch_value(&mut merged.exceptions.destructive, &patch.destructive);
    apply_patch_value(&mut merged.exceptions.border, &patch.border);
    apply_patch_value(&mut merged.exceptions.input, &patch.input);
    apply_patch_value(&mut merged.exceptions.ring, &patch.ring);
    if let Some(series) = &patch.chart_series {
        merged.chart_series = series.clone();
    }
    merged
}

fn apply_patch_value(target: &mut Option<ColorScale>, patch: &Patch<ColorScale>) {
    match patch {
        Patch::Keep => {}
        Patch::Set(value) => *target = Some(*value),
        Patch::Clear => *target = None,
    }
}
