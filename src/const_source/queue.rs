use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use itertools::Itertools;

use crate::ConstSource;
use crate::common::mixer::{MixerHandleInner, MixerKey, mixer_next_body};
use crate::const_source::mixer::MixerHandle;

pub struct Queue<const SR: u32, const CH: u16> {
    sources: Vec<(Box<dyn ConstSource<SR, CH>>, MixerKey)>,
    frame_offset: u16,
    handle: Arc<MixerHandleInner<Box<dyn ConstSource<SR, CH>>>>,
}

impl<const SR: u32, const CH: u16> Queue<SR, CH> {
    pub fn new() -> (QueueHandle<SR, CH>, Self) {
        let handle = Arc::new(MixerHandleInner::new());

        (
            QueueHandle {
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

impl<const SR: u32, const CH: u16> ConstSource<SR, CH> for Queue<SR, CH> {
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .map(|s| s.0.total_duration())
            .fold_options(Duration::ZERO, |max, dur| max.max(dur))
    }
}

impl<const SR: u32, const CH: u16> Iterator for Queue<SR, CH> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channel_count = NonZero::new(CH).unwrap();
        mixer_next_body! {self, channel_count}
    }
}

pub type QueueHandle<const SR: u32, const CH: u16> = MixerHandle<SR, CH>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::const_source::buffer::SamplesBuffer;

    #[test]
    fn add_before_play() {
        let a: SamplesBuffer<44100, 1> = SamplesBuffer::new([1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new([1.0, 2.0]);
        let (mixer, source) = Queue::new();
        mixer.add(a);
        mixer.add(b);

        assert_eq!(vec![1.0, 2.5, 3.0], source.collect_vec())
    }

    #[test]
    fn add_midway() {
        let a: SamplesBuffer<44100, 1> = SamplesBuffer::new([1.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(/*                         */ [1.0, 2.0]);
        let (mixer, mut source) = Queue::new();
        mixer.add(a);

        let _ = source.by_ref().next();
        mixer.add(b);

        assert_eq!(vec![2.0, 2.5], source.collect_vec())
    }

    #[test]
    fn add_is_frame_aligned() {
        let a: SamplesBuffer<44100, 2> = SamplesBuffer::new([1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        let b = SamplesBuffer::new(/*                              */ [1.0, 1.0, 2.0, 2.0]);
        let (mixer, mut source) = Queue::new();
        mixer.add(a);
        assert_eq!(source.next(), Some(1.0));
        mixer.add(b);
        assert_eq!(source.next(), Some(1.0));

        assert_eq!(vec![1.5, 1.5, 2.5, 2.5], source.collect_vec())
    }
}

