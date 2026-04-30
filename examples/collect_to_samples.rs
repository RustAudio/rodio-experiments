use std::time::Duration;

use rodio_experiments::Decoder;
use rodio_experiments::effects::automatic_gain_control::AutomaticGainControlSettings;
use rodio_experiments::nz;
use rodio_experiments::{DynamicSourceExt, FixedSourceExt};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open("assets/music.flac")?;
    let samples: Vec<f32> = Decoder::try_from(file)?
        .resample_into_fixed_source(nz!(44100), nz!(1))
        .automatic_gain_control(AutomaticGainControlSettings::default())
        .take_duration(Duration::from_secs(5))
        .collect();

    // Could use this to power a visualization or something...
    // If you also want to play it back simultaneous you'd need
    // something like rodio-tap (see crates.io)
    println!("Number of samples in 5 seconds of audio: {}", samples.len());

    Ok(())
}
