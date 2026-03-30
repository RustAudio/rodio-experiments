use rodio::wav_to_file;
use rodio_experiments::effects::amplify::Factor;
use rodio_experiments::effects::fade::{CurveControls, CurveEnvelope, LinearEnvelope, Scale};
use rodio_experiments::fixed_source::FixedSourceExt;
use rodio_experiments::fixed_source::mixed::IntoMixed;
use rodio_experiments::generators::fixed_source::function::SineWave;
use rodio_experiments::speakers::SpeakersBuilder;

use std::error::Error;
use std::time::Duration;
use std::{array, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let speakers = SpeakersBuilder::new().default_device()?.default_config()?;
    let config = speakers.get_config();

    let center_freq = 440.0;
    let deviation = 0.0015;
    let note = array::from_fn::<_, 12, _>(|i| {
        [1.0 - deviation, 1.0, 1.0 + deviation]
            .map(|deviation| {
                let freq = center_freq * deviation * (i + 1) as f32;
                SineWave::new(freq, config.sample_rate)
                    .amplify(Factor::Linear(0.4f32.powf((i + 1) as f32)))
            })
            .try_into_mixed()
            .expect("sources with identical parameters can be mixed")
    })
    .try_into_mixed()
    .expect("sources with identical parameters can be mixed")
    .fade(
        CurveEnvelope::builder()
            .with_curve(ATTACK)
            .with_gain(0.0..=1.0)
            .takes(Duration::from_millis(200)),
    )
    .fade(
        LinearEnvelope::builder()
            .with_gain(1.0..=0.5)
            .with_scale(Scale::Normalized)
            .start_after(Duration::from_millis(200))
            .takes(Duration::from_millis(1500)),
    )
    .fade(
        CurveEnvelope::builder()
            .with_curve(RELEASE)
            .with_gain(1.0..=0.0)
            .start_after(Duration::from_millis(200 + 1500))
            .takes(Duration::from_millis(400)),
    )
    .take_duration(Duration::from_millis(200 + 1500 + 400))
    .amplify(Factor::Decibel(0.0))
    .with_channel_count(config.channel_count);

    // wav_to_file(note.into_dynamic_source(), "piano_maybe.wav").unwrap();

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
// https://www.azcalculator.com/calculators/bezier-curve-calculator
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
