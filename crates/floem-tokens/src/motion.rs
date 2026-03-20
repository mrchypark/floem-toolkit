use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionScale {
    pub fast_ms: u64,
    pub normal_ms: u64,
    pub slow_ms: u64,
}

impl Default for MotionScale {
    fn default() -> Self {
        Self {
            fast_ms: 90,
            normal_ms: 160,
            slow_ms: 240,
        }
    }
}
