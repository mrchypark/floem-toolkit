use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpacingScale {
    pub xxs: f64,
    pub xs: f64,
    pub sm: f64,
    pub md: f64,
    pub lg: f64,
    pub xl: f64,
    pub xxl: f64,
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xxs: 2.0,
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
            xxl: 32.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadiusScale {
    pub sm: f64,
    pub md: f64,
    pub lg: f64,
    pub xl: f64,
}

impl Default for RadiusScale {
    fn default() -> Self {
        Self {
            sm: 6.0,
            md: 10.0,
            lg: 14.0,
            xl: 18.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyScale {
    pub xs: f64,
    pub sm: f64,
    pub md: f64,
    pub lg: f64,
    pub xl: f64,
    pub xxl: f64,
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            xs: 11.0,
            sm: 12.0,
            md: 14.0,
            lg: 16.0,
            xl: 20.0,
            xxl: 28.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElevationScale {
    pub low: f64,
    pub mid: f64,
    pub high: f64,
}

impl Default for ElevationScale {
    fn default() -> Self {
        Self {
            low: 1.0,
            mid: 2.0,
            high: 3.0,
        }
    }
}
