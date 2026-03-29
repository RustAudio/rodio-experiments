use rodio::math::db_to_linear;

use crate::effects::pure_effect;
use crate::math::normalized_to_linear;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Factor {
    Linear(f32),
    /// Amplifies the sound logarithmically by the given value.
    ///   - 0 dB = linear value of 1.0 (no change)
    ///   - Positive dB values represent amplification (> 1.0)
    ///   - Negative dB values represent attenuation (< 1.0)
    ///   - -60 dB ≈ 0.001 (barely audible)
    ///   - +20 dB = 10.0 (10x amplification)
    ///
    Decibel(f32),
    /// Normalized amplification in `[0.0, 1.0]` range. This method better
    /// matches the perceived loudness of sounds in human hearing and is
    /// recommended to use when you want to change volume in `[0.0, 1.0]` range.
    /// based on article: <https://www.dr-lex.be/info-stuff/volumecontrols.html>
    ///
    /// **note: it clamps values outside this range.**
    Normalized(f32),
}

impl Factor {
    pub fn input_volume() -> Self {
        Self::Linear(1.0)
    }
    pub fn as_linear(&self) -> f32 {
        match self {
            Factor::Linear(v) => *v,
            Factor::Decibel(db) => db_to_linear(*db),
            Factor::Normalized(normalized) => normalized_to_linear(*normalized),
        }
    }
}

pure_effect! {
    supports_dynamic_source
    struct Amplify {
        factor: f32,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next().map(|value| value * self.factor)
    }

    fn new(source: S, factor: Factor) -> Amplify<Self> {
        Self {
            inner: source,
            factor: factor.as_linear(),
        }
    }

    pub fn set_factor(&mut self, factor: Factor) {
        self.factor = factor.as_linear()
    }
}
