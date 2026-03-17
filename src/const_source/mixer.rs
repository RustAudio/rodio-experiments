// Now is there this strange dance with Vec's?
// We can not allocate or deallocate in any Iterator implementation in Rodio.
// Why? (De)Allocating almost always requires taking a Mutex which may block
// if anything else is allocating. That may slow down the audio thread enough
// that it no longer meets it's deadline. Therefore we allocate on another
// thread and then move the sources over. To speed up the adding of a lot of
// sources in batches we already put those in the newly allocated memory.

use std::fmt::Display;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use itertools::Itertools;

use crate::common::mixer::{MixerHandleInner, MixerKey, mixer_next_body};
use crate::{ChannelCount, SampleRate};
use crate::ConstSource;

pub struct Mixer<const SR: u32, const CH: u16> {
    sources: Vec<(Box<dyn ConstSource<SR, CH>>, MixerKey)>,
    frame_offset: u16,
    handle: Arc<MixerHandleInner<Box<dyn ConstSource<SR, CH>>>>,
}

impl<const SR: u32, const CH: u16> Mixer<SR, CH> {
    pub fn new() -> (MixerHandle<SR, CH>, Self) {
        let handle = Arc::new(MixerHandleInner::new());

        (
            MixerHandle {
                inner: handle.clone(),
            },
            Self {
                sources: Vec::new(),
                frame_offset: 0,
                handle,
            },
        )
    }
}

impl<const SR: u32, const CH: u16> ConstSource<SR, CH> for Mixer<SR, CH> {
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .map(|s| s.0.total_duration())
            .fold_options(Duration::ZERO, |max, dur| max.max(dur))
    }
}

impl<const SR: u32, const CH: u16> Iterator for Mixer<SR, CH> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channel_count = NonZero::new(CH).unwrap();
        mixer_next_body! {self, channel_count}
    }
}

#[derive(Clone)]
pub struct MixerHandle<const SR: u32, const CH: u16> {
    inner: Arc<MixerHandleInner<Box<dyn ConstSource<SR, CH>>>>,
}

impl<const SR: u32, const CH: u16> MixerHandle<SR, CH> {
    pub fn add(
        &self,
        source: impl ConstSource<SR, CH> + 'static,
    ) -> MixerKey {
        let source = Box::new(source) as Box<dyn ConstSource<SR, CH>>;
        self.inner.add_unchecked(source)
    }

    pub fn remove(&self, key: MixerKey) {
        self.inner.remove(key)
    }

    pub fn sample_rate(&self) -> SampleRate {
        const { NonZero::new(SR).expect("SampleRate may never be zero") }
    }

    pub fn channels(&self) -> ChannelCount {
        const { NonZero::new(CH).expect("SampleRate may never be zero") }
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
    use crate::const_source::buffer::SamplesBuffer;
    use super::*;

    #[test]
    fn add_before_play() {
        let a: SamplesBuffer<44100, 1> = SamplesBuffer::new([1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new([1.0, 2.0]);
        let (mixer, source) = Mixer::new();
        mixer.add(a);
        mixer.add(b);

        assert_eq!(vec![1.0, 2.5, 3.0], source.collect_vec())
    }

    #[test]
    fn add_midway() {
        let a: SamplesBuffer<44100, 1> = SamplesBuffer::new([1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(/*                         */ [1.0, 2.0]);
        let (mixer, mut source) = Mixer::new();
        mixer.add(a);

        let _ = source.by_ref().next();
        mixer.add(b);

        assert_eq!(vec![2.0, 2.5], source.collect_vec())
    }

    #[test]
    fn add_is_frame_aligned() {
        let a: SamplesBuffer<44100, 2> = SamplesBuffer::new([1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(/*                              */ [1.0, 1.0, 2.0, 2.0]);
        let (mixer, mut source) = Mixer::new();
        mixer.add(a);
        assert_eq!(source.next(), Some(1.0));
        mixer.add(b);
        assert_eq!(source.next(), Some(1.0));

        assert_eq!(vec![1.5, 1.5, 2.5, 2.5], source.collect_vec())
    }
}
