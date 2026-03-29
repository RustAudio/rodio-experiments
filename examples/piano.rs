use rodio_experiments::effects::amplify::Factor;
use rodio_experiments::effects::fade::{CurveControls, CurveEnvelope, LinearEnvelope, Scale};
use rodio_experiments::fixed_source::FixedSourceExt;
use rodio_experiments::fixed_source::mixed::IntoMixed;
use rodio_experiments::generators::fixed_source::function::SineWave;
use rodio_experiments::speakers::SpeakersBuilder;

use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let speakers = SpeakersBuilder::new().default_device()?.default_config()?;
    let config = speakers.get_config();

    let note = (1..8)
        .into_iter()
        .map(|i| {
            SineWave::new(440.0 * (i as f32), config.sample_rate)
                .amplify(Factor::Linear(1.0 / 2.5f32.powf(i as f32)))
        })
        .collect::<Vec<_>>()
        .try_into_mixed()
        .expect("parameters all identical")
        .fade(
            CurveEnvelope::builder()
                .with_curve(ATTACK)
                .with_gain(0.0..=1.0)
                .takes(Duration::from_millis(400)),
        )
        .fade(
            LinearEnvelope::builder()
                .with_gain(1.0..=0.0)
                .with_scale(Scale::Normalized)
                .start_after(Duration::from_millis(400))
                .takes(Duration::from_millis(1500)),
        )
        .fade(
            CurveEnvelope::builder()
                .with_curve(RELEASE)
                .with_gain(1.0..=0.0)
                .start_after(Duration::from_millis(1500))
                .takes(Duration::from_millis(400)),
        )
        .take_duration(Duration::from_millis(4))
        .with_channel_count(config.channel_count);

    let _speakers = speakers.play(note)?;

    thread::sleep(Duration::from_secs(2));
    Ok(())
}

const ATTACK: CurveControls = CurveControls {
    x1: 0.0,
    y1: 0.77,
    x2: 0.32,
    y2: 1.0,
};

// nice editor for cubic bezier curves
// https://www.desmos.com/calculator/iogphoixw4
const DECAY: CurveControls = CurveControls {
    x1: 0.0,
    y1: 0.0,
    x2: 1.0,
    y2: 0.0,
};

const RELEASE: CurveControls = CurveControls {
    x1: 0.0,
    y1: 1.0,
    x2: 1.0,
    y2: 0.0,
};
