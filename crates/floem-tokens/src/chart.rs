use crate::color::ColorScale;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTokens {
    pub axis: ColorScale,
    pub grid: ColorScale,
    pub tooltip_bg: ColorScale,
    pub tooltip_text: ColorScale,
    pub selection: ColorScale,
    pub series: Vec<ColorScale>,
}
