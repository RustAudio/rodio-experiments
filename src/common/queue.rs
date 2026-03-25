use std::fmt::Display;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::common::SourceShare;
use crate::fixed_source::convert_if_needed;
use crate::{ChannelCount, SampleRate};

macro_rules! queue_next_body {
    ($self:ident, $channel_count:ident) => {
        if $self.frame_offset == 0
            && $self
                .handle
                .sources_changed
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            if let Ok(mut shared) = $self.handle.source_share.try_lock() {
                shared.update(&mut $self.sources);
                // ORDERING: this has to unset before the `tomato` Mutex guard is
                // released or we could overwrite a re-set. Thanks to that Mutex
                // this can be Relaxed
                $self
                    .handle
                    .sources_changed
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }

        $self.frame_offset += 1;
        $self.frame_offset %= $channel_count.get();

        loop {
            let (playing, _) = $self.sources.get_mut($self.current)?;
            if let Some(sample) = playing.next() {
                return Some(sample);
            } else {
                $self.current += 1;
                continue;
            }
        }
    };
}
pub(crate) use queue_next_body;

pub(crate) struct QueueHandleInner<S: Send + 'static> {
    next_key: AtomicUsize,
    pub(crate) sources_changed: AtomicBool,
    pub(crate) source_share: Mutex<SourceShare<S, QueueKey>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueKey(NonZeroUsize);

impl<S: Send + 'static> QueueHandleInner<S> {
    pub(crate) fn new() -> Self {
        Self {
            next_key: AtomicUsize::new(1),
            sources_changed: AtomicBool::new(false),
            source_share: Mutex::new(SourceShare {
                capacity: 0,
                len: 0,
                to_remove: Vec::new(),
                new_vec: Vec::new(),
                old_vec: None,
            }),
        }
    }

    fn new_key(&self) -> QueueKey {
        let key = self.next_key.fetch_add(1, Ordering::Relaxed);

        assert!(
            key < usize::MAX,
            "Can only add usize::MAX -1 sources to a mixer"
        );
        QueueKey(key.try_into().expect("next_key initialized as one"))
    }

    pub(crate) fn add_unchecked(&self, source: S) -> QueueKey {
        let key = self.new_key();
        self.source_share
            .lock()
            .expect("audio thread should never panic")
            .schedule_addition(source, key);
        // ORDERING: needs to happen _before_ the self.source_share Mutex is unlocked
        self.sources_changed.store(true, Ordering::Relaxed);
        key
    }

    pub(crate) fn remove(&self, key: QueueKey) {
        self.source_share
            .lock()
            .expect("audio thread should never panic")
            .schedule_removal(key)
    }
}
