use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorScale {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorScale {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

pub const fn with_alpha(color: ColorScale, a: u8) -> ColorScale {
    ColorScale { a, ..color }
}

pub fn alpha(color: ColorScale, amount: f32) -> ColorScale {
    let clamped = amount.clamp(0.0, 1.0);
    with_alpha(color, (255.0 * clamped) as u8)
}

pub fn lighten(color: ColorScale, amount: f32) -> ColorScale {
    shift(color, amount.abs())
}

pub fn darken(color: ColorScale, amount: f32) -> ColorScale {
    shift(color, -amount.abs())
}

fn shift(color: ColorScale, amount: f32) -> ColorScale {
    let delta = (255.0 * amount.clamp(-1.0, 1.0)) as i16;
    let adjust = |value: u8| -> u8 { ((value as i16 + delta).clamp(0, 255)) as u8 };
    ColorScale::rgba(adjust(color.r), adjust(color.g), adjust(color.b), color.a)
}
