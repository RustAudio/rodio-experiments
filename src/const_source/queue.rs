use std::num::NonZero;
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;

use crate::ConstSource;
use crate::common::queue::queue_next_body;
use crate::common::queue::{QueueHandleInner, QueueKey};

pub struct Queue<const SR: u32, const CH: u16> {
    sources: Vec<(Box<dyn ConstSource<SR, CH> + Send + 'static>, QueueKey)>,
    current: usize,
    frame_offset: u16,
    handle: Arc<QueueHandleInner<Box<dyn ConstSource<SR, CH> + Send + 'static>>>,
}

impl<const SR: u32, const CH: u16> Queue<SR, CH> {
    pub fn new() -> (QueueHandle<SR, CH>, Self) {
        let handle = Arc::new(QueueHandleInner::new());

        (
            QueueHandle {
                inner: handle.clone(),
            },
            Self {
                sources: Vec::new(),
                current: 0,
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
            .fold_options(Duration::ZERO, |sum, dur| sum + dur)
    }
}

impl<const SR: u32, const CH: u16> Iterator for Queue<SR, CH> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let channel_count = NonZero::new(CH).unwrap();
        queue_next_body! {self, channel_count}
    }
}

#[derive(Clone)]
pub struct QueueHandle<const SR: u32, const CH: u16> {
    pub(crate) inner: Arc<QueueHandleInner<Box<dyn ConstSource<SR, CH> + Send + 'static>>>,
}

impl<const SR: u32, const CH: u16> QueueHandle<SR, CH> {
    pub fn add(&self, source: impl ConstSource<SR, CH> + Send + 'static) -> QueueKey {
        let source = Box::new(source) as Box<dyn ConstSource<SR, CH> + Send + 'static>;
        self.inner.add_unchecked(source)
    }

    pub fn remove(&self, key: QueueKey) {
        self.inner.remove(key)
    }

    pub fn sample_rate(&self) -> crate::SampleRate {
        const { NonZero::new(SR).expect("Sample rate can't be zero") }
    }

    pub fn channels(&self) -> crate::ChannelCount {
        const { NonZero::new(CH).expect("Channel count can't be zero") }
    }
}
