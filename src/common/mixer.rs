use std::fmt::Display;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::fixed_source::convert_if_needed;
use crate::{ChannelCount, SampleRate};

macro_rules! mixer_next_body {
    ($self:ident, $channel_count:ident) => {
        if $self.frame_offset == 0 && $self.handle.sources_changed.load(Ordering::Relaxed) {
            if let Ok(mut shared) = $self.handle.source_share.try_lock() {
                shared.update(&mut $self.sources);
                // ORDERING: this has to unset before the `tomato` Mutex guard is
                // released or we could overwrite a re-set. Thanks to that Mutex
                // this can be Relaxed
                $self.handle.sources_changed.store(false, Ordering::Relaxed);
            }
        }

        $self.frame_offset += 1;
        $self.frame_offset %= $channel_count.get();

        let (sum, summed) = $self
            .sources
            .iter_mut()
            .filter_map(|(source, _)| source.next())
            .map(|sample| sample as f64)
            .zip((1usize..).into_iter())
            .reduce(|(sum, _), (sample, summed)| (sum + sample, summed))?;
        Some((sum / summed as f64) as crate::Float)
    };
}
pub(crate) use mixer_next_body;

pub(crate) struct SourceShare<S> {
    capacity: usize,
    len: usize,
    to_remove: Vec<MixerKey>,
    /// we do not allocate on the audio thread
    /// instead the handle does that
    new_vec: Vec<(S, MixerKey)>,
    /// we do not deallocate on the audio thread
    /// instead the handle does that
    old_vec: Option<Vec<(S, MixerKey)>>,
}

pub(crate) struct MixerHandleInner<S> {
    next_key: AtomicUsize,
    pub(crate) sources_changed: AtomicBool,
    pub(crate) source_share: Mutex<SourceShare<S>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixerKey(NonZeroUsize);

impl<S> MixerHandleInner<S> {
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

    fn new_key(&self) -> MixerKey {
        let key = self.next_key.fetch_add(1, Ordering::Relaxed);

        assert!(
            key < usize::MAX,
            "Can only add usize::MAX -1 sources to a mixer"
        );
        MixerKey(key.try_into().expect("next_key initialized as one"))
    }

    pub(crate) fn add_unchecked(&self, source: S) -> MixerKey {
        let key = self.new_key();
        self.source_share
            .lock()
            .expect("audio thread should never panic")
            .schedule_addition(source, key);
        // ORDERING: needs to happen _before_ the self.source_share Mutex is unlocked
        self.sources_changed.store(true, Ordering::Relaxed);
        key
    }

    pub(crate) fn remove(&self, key: MixerKey) {
        self.source_share
            .lock()
            .expect("audio thread should never panic")
            .schedule_removal(key)
    }
}

impl<S> SourceShare<S> {
    pub(crate) fn schedule_addition(&mut self, source: S, key: MixerKey) {
        let _drop = self.old_vec.take();

        if self.len + 1 >= self.capacity {
            self.capacity *= 2;
        }
        self.new_vec
            .reserve(self.capacity.saturating_sub(self.new_vec.capacity()));
        self.new_vec.push((source, key));
    }

    pub(crate) fn update(&mut self, current_vec: &mut Vec<(S, MixerKey)>) {
        current_vec.retain(|(_, key)| !self.to_remove.contains(key));

        swap_append(current_vec, &mut self.new_vec);

        self.old_vec = Some(std::mem::take(&mut self.new_vec));
    }

    pub(crate) fn schedule_removal(&mut self, key: MixerKey) {
        self.to_remove.push(key);
    }
}

fn swap_append<T>(curr: &mut Vec<T>, new: &mut Vec<T>) {
    // dear compiler please optimize this to something sensible so I do not have
    // to use unsafe code
    for (idx, element) in curr.drain(..).enumerate() {
        new.insert(idx, element)
    }
    std::mem::swap(curr, new);
}
