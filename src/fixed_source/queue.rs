use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::common::make_params_mismatch_error;
use crate::common::queue::queue_next_body;
use crate::common::queue::{QueueHandleInner, QueueKey};
use crate::fixed_source::convert_if_needed;
use crate::{ChannelCount, SampleRate};

pub struct Queue {
    sources: Vec<(Box<dyn FixedSource + Send + 'static>, QueueKey)>,
    current: usize,
    frame_offset: u16,
    sample_rate: SampleRate,
    channel_count: ChannelCount,
    handle: Arc<QueueHandleInner<Box<dyn FixedSource + Send + 'static>>>,
}

impl Queue {
    pub fn new(sample_rate: SampleRate, channel_count: ChannelCount) -> (QueueHandle, Self) {
        let handle = Arc::new(QueueHandleInner::new());

        (
            QueueHandle {
                sample_rate,
                channel_count,
                inner: handle.clone(),
            },
            Self {
                current: 0,
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
            .fold_options(Duration::ZERO, |sum, dur| sum + dur)
    }
}

impl Iterator for Queue {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channel_count = self.channel_count;
        queue_next_body! {self, channel_count}
    }
}

#[derive(Clone)]
pub struct QueueHandle {
    pub(crate) sample_rate: SampleRate,
    pub(crate) channel_count: ChannelCount,
    pub(crate) inner: Arc<QueueHandleInner<Box<dyn FixedSource + Send + 'static>>>,
}

impl QueueHandle {
    pub fn try_add(
        &self,
        source: impl FixedSource + Send + 'static,
    ) -> Result<QueueKey, ParamsMismatch> {
        if (source.sample_rate(), source.channels()) != (self.sample_rate, self.channel_count) {
            return Err(ParamsMismatch {
                sample_rate_mixer: self.sample_rate,
                channel_count_mixer: self.channel_count,
                sample_rate_new: source.sample_rate(),
                channel_count_new: source.channels(),
            });
        }

        let source = Box::new(source) as Box<dyn FixedSource + Send + 'static>;
        Ok(self.inner.add_unchecked(source))
    }

    pub fn add_converted(&self, source: impl FixedSource + Send + 'static) -> QueueKey {
        let source = convert_if_needed(source, self.sample_rate, self.channel_count).into_box_dyn();
        self.inner.add_unchecked(source)
    }

    pub fn remove(&self, key: QueueKey) {
        self.inner.remove(key)
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn channels(&self) -> ChannelCount {
        self.channel_count
    }
}

make_params_mismatch_error! { "queue", "QueueHandle" }

#[cfg(test)]
mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn add_before_play() {
        let a = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 2.0]);
        let b = SamplesBuffer::new(nz!(1), nz!(44100), [3.0, 4.0]);
        let (queue, source) = Queue::new(nz!(44100), nz!(1));
        queue.try_add(a).unwrap();
        queue.try_add(b).unwrap();

        assert_eq!(vec![1., 2., 3., 4.], source.collect_vec())
    }

    #[test]
    fn add_midway() {
        let a = SamplesBuffer::new(nz!(1), nz!(44100), [1.0, 2.0, 3.0]);
        let b = SamplesBuffer::new(nz!(1), nz!(44100), /**/ [4.0, 5.0]);
        let (queue, mut source) = Queue::new(nz!(44100), nz!(1));
        queue.try_add(a).unwrap();

        let _ = source.by_ref().next();
        queue.try_add(b).unwrap();

        assert_eq!(vec![2., 3., 4., 5.], source.collect_vec())
    }

    #[test]
    fn start_empty() {
        let (_, mut source) = Queue::new(nz!(44100), nz!(1));
        assert_eq!(source.next(), None);
        assert_eq!(source.next(), None);
    }

    #[test]
    fn different_params_is_refused() {
        let a = SamplesBuffer::new(nz!(2), nz!(44100), [1.0, 1.0]);
        let (queue, _) = Queue::new(nz!(44100), nz!(1));
        assert!(queue.try_add(a).is_err());
    }
}
