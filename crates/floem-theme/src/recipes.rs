use floem_tokens::ColorScale;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentSize {
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonRecipe {
    pub background: ColorScale,
    pub foreground: ColorScale,
    pub border: ColorScale,
    pub hover_background: ColorScale,
    pub active_background: ColorScale,
    pub ring: ColorScale,
    pub radius: f64,
    pub padding_x: f64,
    pub padding_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputRecipe {
    pub background: ColorScale,
    pub foreground: ColorScale,
    pub border: ColorScale,
    pub border_focus: ColorScale,
    pub border_invalid: ColorScale,
    pub placeholder: ColorScale,
    pub radius: f64,
    pub height: f64,
    pub font_size: f64,
    pub line_height: f64,
    pub padding_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelRecipe {
    pub foreground: ColorScale,
    pub font_size: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardRecipe {
    pub background: ColorScale,
    pub foreground: ColorScale,
    pub border: ColorScale,
    pub radius: f64,
    pub padding: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogRecipe {
    pub panel: CardRecipe,
    pub overlay: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxRecipe {
    pub background: ColorScale,
    pub border: ColorScale,
    pub checked: ColorScale,
    pub checked_foreground: ColorScale,
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabsRecipe {
    pub tab_background: ColorScale,
    pub tab_foreground: ColorScale,
    pub tab_active_background: ColorScale,
    pub tab_active_foreground: ColorScale,
    pub border: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopoverRecipe {
    pub panel: CardRecipe,
    pub shadow: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectRecipe {
    pub input: InputRecipe,
    pub popover: PopoverRecipe,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxisTheme {
    pub color: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridTheme {
    pub color: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesPalette {
    pub series: Vec<ColorScale>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTooltipTheme {
    pub background: ColorScale,
    pub foreground: ColorScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTheme {
    pub axis: AxisTheme,
    pub grid: GridTheme,
    pub palette: SeriesPalette,
    pub tooltip: ChartTooltipTheme,
}
