//! We didn't have the technology, but I really wanted it. So now we do.
//!
//! (You can view this as a testbed for some of the ideas I've had for Rodio)
//! - Yara

// use std::time::Duration;

// use rodio::{ChannelCount, Sample, SampleRate, Source, source::SineWave};
pub use rodio::buffer;
pub use rodio::{BitDepth, ChannelCount, Float, Sample, SampleRate};
// pub use rodio::conversions;
pub use rodio::Player;
pub use rodio::SpatialPlayer;
pub use rodio::decoder;
pub use rodio::decoder::Decoder;
pub use rodio::microphone;
pub use rodio::mixer;
pub use rodio::queue;
pub use rodio::source;
pub use rodio::source::Source;
pub use rodio::speakers;
pub use rodio::static_buffer;
pub use rodio::stream;
pub use rodio::stream::{DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, PlayError, play};
pub use rodio::wav_to_file;
pub use rodio::wav_to_writer;

pub mod const_source;
pub mod conversions;
pub mod dynamic_source;
pub mod dynamic_source_ext;
pub mod fixed_source;

pub mod effects;
mod generators;
pub use generators::{function, noise, chirp};

pub mod common;
pub mod math;

pub use rodio::cpal;

pub use const_source::ConstSource;
pub use rodio::FixedSource;
pub use rodio::Source as DynamicSource;
