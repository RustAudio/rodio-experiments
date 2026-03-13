use rodio_experiments::fixed_source::FixedSourceExt;
use rodio_experiments::generators::fixed_source::function::SineWave;
use rodio_experiments::speakers::SpeakersBuilder;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let speakers = SpeakersBuilder::new().default_device()?.default_config()?;
    let config = speakers.get_config();

    let beep = SineWave::new(440.0, config.sample_rate);
    // TODO add drop bomb upstream
    let _speakers = speakers.play(
        beep.take_duration(Duration::from_secs(2))
            .with_channel_count(config.channel_count)
            .inspect_frame(|frame| dbg!(frame)),
    )?;

    thread::sleep(Duration::from_secs(2));
    Ok(())
}
