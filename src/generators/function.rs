// adapted from rodio. Copyright rodio contributors.
// Tests and docs removed for brevity (will be re-added once contributing to upstream)

use std::f32::consts::TAU;

pub type GeneratorFunction = fn(f32) -> f32;

pub mod const_source;
pub mod fixed_source;

#[derive(Clone, Debug)]
pub enum Function {
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

fn sine_signal(phase: f32) -> f32 {
    (TAU * phase).sin()
}

fn triangle_signal(phase: f32) -> f32 {
    4.0f32 * (phase - (phase + 0.5f32).floor()).abs() - 1f32
}

fn square_signal(phase: f32) -> f32 {
    if phase % 1.0f32 < 0.5f32 {
        1.0f32
    } else {
        -1.0f32
    }
}

fn sawtooth_signal(phase: f32) -> f32 {
    2.0f32 * (phase - (phase + 0.5f32).floor())
}
