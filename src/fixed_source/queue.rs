use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::common::mixer::{MixerHandleInner, MixerKey, mixer_next_body};
use crate::fixed_source::mixer::MixerHandle;
use crate::{ChannelCount, SampleRate};

pub struct Queue {
    sources: Vec<(Box<dyn FixedSource>, MixerKey)>,
    frame_offset: u16,
    sample_rate: SampleRate,
    channel_count: ChannelCount,
    handle: Arc<MixerHandleInner<Box<dyn FixedSource>>>,
}

impl Queue {
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

impl FixedSource for Queue {
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

impl Iterator for Queue {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channel_count = self.channel_count;
        mixer_next_body! {self, channel_count}
    }
}

pub type QueueHandle = MixerHandle;

#[cfg(test)]
mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn add_before_play() {
        let a = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 2.0]);
        let (mixer, source) = Queue::new(nz!(44100), nz!(1));
        mixer.try_add(a).unwrap();
        mixer.try_add(b).unwrap();

        assert_eq!(vec![1.0, 2.5, 3.0], source.collect_vec())
    }

    #[test]
    fn add_midway() {
        let a = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(nz!(1), nz!(44100), /* */ [1.0, 2.0]);
        let (mixer, mut source) = Queue::new(nz!(44100), nz!(1));
        mixer.try_add(a).unwrap();

        let _ = source.by_ref().next();
        mixer.try_add(b).unwrap();

        assert_eq!(vec![2.0, 2.5], source.collect_vec())
    }

    #[test]
    fn start_empty() {
        let (_, mut source) = Queue::new(nz!(44100), nz!(1));
        assert_eq!(source.next(), None);
        assert_eq!(source.next(), None);
    }

    #[test]
    fn add_is_frame_aligned() {
        let a = SamplesBuffer::new(nz!(2), nz!(44100), [1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(nz!(2), nz!(44100), /*     */ [1.0, 1.0, 2.0, 2.0]);
        let (mixer, mut source) = Queue::new(nz!(44100), nz!(2));
        mixer.try_add(a).unwrap();
        assert_eq!(source.next(), Some(1.0));
        mixer.try_add(b).unwrap();
        assert_eq!(source.next(), Some(1.0));

        assert_eq!(vec![1.5, 1.5, 2.5, 2.5], source.collect_vec())
    }

    #[test]
    fn different_params_is_refused() {
        let a = SamplesBuffer::new(nz!(2), nz!(44100), [1.0, 1.0]);
        let (mixer, _) = Queue::new(nz!(44100), nz!(1));
        assert!(mixer.try_add(a).is_err());
    }
}
