// Now is there this strange dance with Vec's?
// We can not allocate or deallocate in any Iterator implementation in Rodio.
// Why? (De)Allocating almost always requires taking a Mutex which may block
// if anything else is allocating. That may slow down the audio thread enough
// that it no longer meets it's deadline. Therefore we allocate on another
// thread and then move the sources over. To speed up the adding of a lot of
// sources in batches we already put those in the newly allocated memory.

use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::common::mixer::{MixerHandleInner, MixerKey, mixer_next_body};
use crate::fixed_source::convert_if_needed;
use crate::{ChannelCount, SampleRate};

pub struct Mixer {
    sources: Vec<(Box<dyn FixedSource>, MixerKey)>,
    frame_offset: u16,
    sample_rate: SampleRate,
    channel_count: ChannelCount,
    handle: Arc<MixerHandleInner<Box<dyn FixedSource>>>,
}

impl Mixer {
    pub fn new(sample_rate: SampleRate, channel_count: ChannelCount) -> (MixerHandle, Self) {
        let handle = Arc::new(MixerHandleInner::new());

        (
            MixerHandle {
                sample_rate,
                channel_count,
                inner: handle.clone(),
            },
            Self {
                sample_rate,
                channel_count,
                sources: Vec::new(),
                frame_offset: 0,
                handle,
            },
        )
    }
}

impl FixedSource for Mixer {
    fn channels(&self) -> crate::ChannelCount {
        self.channel_count
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .map(|s| s.0.total_duration())
            .fold_options(Duration::ZERO, |max, dur| max.max(dur))
    }
}

impl Iterator for Mixer {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channel_count = self.channel_count;
        mixer_next_body! {self, channel_count}
    }
}

#[derive(Clone)]
pub struct MixerHandle {
    sample_rate: SampleRate,
    channel_count: ChannelCount,
    inner: Arc<MixerHandleInner<Box<dyn FixedSource>>>,
}

impl MixerHandle {
    pub fn try_add(&self, source: impl FixedSource + 'static) -> Result<MixerKey, ParamsMismatch> {
        if (source.sample_rate(), source.channels()) != (self.sample_rate, self.channel_count) {
            return Err(ParamsMismatch {
                sample_rate_mixer: self.sample_rate,
                channel_count_mixer: self.channel_count,
                sample_rate_new: source.sample_rate(),
                channel_count_new: source.channels(),
            });
        }

        let source = Box::new(source) as Box<dyn FixedSource>;
        Ok(self.inner.add_unchecked(source))
    }

    pub fn add_converted(&self, source: impl FixedSource + 'static) -> MixerKey {
        let source = convert_if_needed(source, self.sample_rate, self.channel_count).into_box_dyn();
        self.inner.add_unchecked(source)
    }

    pub fn remove(&self, key: MixerKey) {
        self.inner.remove(key)
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn channels(&self) -> ChannelCount {
        self.channel_count
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub struct ParamsMismatch {
    pub(crate) sample_rate_mixer: SampleRate,
    pub(crate) channel_count_mixer: ChannelCount,
    pub(crate) sample_rate_new: SampleRate,
    pub(crate) channel_count_new: ChannelCount,
}

impl Display for ParamsMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParamsMismatch {
            sample_rate_mixer,
            channel_count_mixer,
            sample_rate_new,
            channel_count_new,
        } = self;
        f.write_fmt(format_args!("Parameters mismatch, the mixer is set up with sample rate: {sample_rate_mixer} and channel count: {channel_count_mixer}. You are trying to add a source with sample rate: {sample_rate_new} and {channel_count_new}. Try using `MixerHandle::add_converted` instead"))
    }
}

#[cfg(test)]
mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn add_before_play() {
        let a = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 2.0]);
        let (mixer, source) = Mixer::new(nz!(44100), nz!(1));
        mixer.try_add(a).unwrap();
        mixer.try_add(b).unwrap();

        assert_eq!(vec![1.0, 2.5, 3.0], source.collect_vec())
    }

    #[test]
    fn add_midway() {
        let a = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(nz!(1), nz!(44100), /* */ [1.0, 2.0]);
        let (mixer, mut source) = Mixer::new(nz!(44100), nz!(1));
        mixer.try_add(a).unwrap();

        let _ = source.by_ref().next();
        mixer.try_add(b).unwrap();

        assert_eq!(vec![2.0, 2.5], source.collect_vec())
    }

    #[test]
    fn start_empty() {
        let (_, mut source) = Mixer::new(nz!(44100), nz!(1));
        assert_eq!(source.next(), None);
        assert_eq!(source.next(), None);
    }

    #[test]
    fn add_is_frame_aligned() {
        let a = SamplesBuffer::new(nz!(2), nz!(44100), [1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(nz!(2), nz!(44100), /*     */ [1.0, 1.0, 2.0, 2.0]);
        let (mixer, mut source) = Mixer::new(nz!(44100), nz!(2));
        mixer.try_add(a).unwrap();
        assert_eq!(source.next(), Some(1.0));
        mixer.try_add(b).unwrap();
        assert_eq!(source.next(), Some(1.0));

        assert_eq!(vec![1.5, 1.5, 2.5, 2.5], source.collect_vec())
    }

    #[test]
    fn different_params_is_refused() {
        let a = SamplesBuffer::new(nz!(2), nz!(44100), [1.0, 1.0]);
        let (mixer, _) = Mixer::new(nz!(44100), nz!(1));
        assert!(mixer.try_add(a).is_err());
    }
}
