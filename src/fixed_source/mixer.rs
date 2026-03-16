/// This uses a fair amount of unsafe combined with concurrent & parallel code.
/// This is done to uphold realtime performance constraints. Mistakes here can
/// introduce widespread horrible undefined behavior. As the Mixer is the core
/// or Rodio the undefined behavior will be wide spread. Please ensure you know
/// what you are doing when making changes here. If you do not know what pointer
/// provenance, memory fences or atomic ordering is about please do not make
/// changes to this module.
///
/// After EVERY CHANGE you MUST run the test suite under MIRI.
///
///
/// Now why do we need this?
/// We can not allocate or deallocate in any Iterator implementation in Rodio.
/// Why? (De)Allocating almost always requires taking a Mutex which may block
/// if anything else is allocating. That may slow down the audio thread enough
/// that it no longer meets it's deadline. Therefore we allocate on another
/// thread and then move the sources over. To speed up the adding of a lot of
/// sources in batches we already put those in the newly allocated memory.

use std::fmt::Display;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use itertools::Itertools;

use crate::FixedSource;
use crate::common::mixed_next_body;
use crate::fixed_source::convert_if_needed;
use crate::{ChannelCount, SampleRate};

pub struct Mixer {
    sources: Vec<Box<dyn FixedSource>>,
    frame_offset: u16,
    handle: MixerHandleInner,
}

impl FixedSource for Mixer {
    fn channels(&self) -> crate::ChannelCount {
        self.handle.channel_count
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.handle.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .map(|s| s.total_duration())
            .fold_options(Duration::ZERO, |max, dur| max.max(dur))
    }
}

impl Iterator for Mixer {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.handle.add.load(Ordering::Relaxed) && self.frame_offset == 0 {
            if let Ok(mut tomato) = self.handle.tomato.try_lock() {
                tomato.take(&mut self.sources);
                // ORDERING: this has to unset before the `tomato` Mutex guard is
                // released or we could overwrite a re-set. Thanks to that Mutex
                // this can be Relaxed
                self.handle.add.store(false, Ordering::Relaxed);
            }
        }

        self.frame_offset += 1;
        self.frame_offset %= self.handle.channel_count.get();

        mixed_next_body! {self}
    }
}

#[derive(Clone)]
pub struct MixerHandle(Arc<MixerHandleInner>);

impl MixerHandle {
    pub fn try_add(&self, source: impl FixedSource + 'static) -> Result<MixerKey, ParamsMismatch> {
        self.0.try_add(source)
    }

    pub fn add_converted(&self, source: impl FixedSource + 'static) -> MixerKey {
        self.0.add_converted(source)
    }

    pub fn remove(&self, key: MixerKey) -> () {
        todo!()
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.0.sample_rate
    }

    pub fn channels(&self) -> ChannelCount {
        self.0.channel_count
    }
}

struct Tomato {
    capacity: usize,
    len: usize,
    /// we do not allocate on the audio thread
    /// instead the handle does that
    /// # SAFETY:
    /// this is partially initialized!
    new_vec: Vec<Box<dyn FixedSource>>,
    /// we do not deallocate on the audio thread
    /// instead the handle does that
    old_vec: Option<Vec<Box<dyn FixedSource>>>,
}

struct MixerHandleInner {
    sample_rate: SampleRate,
    channel_count: ChannelCount,
    next_key: AtomicUsize,
    add: AtomicBool,
    tomato: Mutex<Tomato>,
}

pub struct MixerKey;

impl MixerHandleInner {
    fn try_add(&self, source: impl FixedSource + 'static) -> Result<MixerKey, ParamsMismatch> {
        if (source.sample_rate(), source.channels()) != (self.sample_rate, self.channel_count) {
            return Err(ParamsMismatch {
                sample_rate_mixer: self.sample_rate,
                channel_count_mixer: self.channel_count,
                sample_rate_new: source.sample_rate(),
                channel_count_new: source.channels(),
            });
        }

        let source = Box::new(source) as Box<dyn FixedSource>;
        self.tomato
            .lock()
            .expect("audio thread should never panic")
            .add(source);
        todo!()
    }

    fn add_converted(&self, source: impl FixedSource + 'static) -> MixerKey {
        let source = convert_if_needed(source, self.sample_rate, self.channel_count).into_box_dyn();
        self.tomato
            .lock()
            .expect("audio thread should never panic")
            .add(source);
        todo!();
    }
}

impl Tomato {
    fn add(&mut self, source: Box<dyn FixedSource>) {
        let _drop = self.old_vec.take();

        if self.len + 1 >= self.capacity {
            self.capacity *= 2;
            if self.new_vec.is_empty() {
                self.new_vec = Vec::with_capacity(self.capacity);
                let uninit = self.new_vec.spare_capacity_mut();
                uninit[self.len - 1].write(source);
                self.len += 1;
                // SAFETY: only one element (at index self.len) is initialized
                // it is fully initialized on the audio thread by copying
                // elements into the range 0..self.len. It is not read until
                // that has happened.
                unsafe {
                    self.new_vec.set_len(self.len);
                }
            } else {
                // len has already been set
                // SAFETY: if this causes the Vec to reallocate this will read
                // uninitialized data. It is only read for copying if that
                // copying is optimized out that is fine, great even.
                self.new_vec.push(source);
            }
        }
    }

    fn take(&mut self, current_vec: &mut Vec<Box<dyn FixedSource>>) {
        // SAFETY: ..0 is always initialized
        unsafe {
            self.new_vec.set_len(0);
        }
        let uninit = self.new_vec.spare_capacity_mut();
        for (element, place) in current_vec.drain(..).zip(uninit) {
            place.write(element);
        }
        // # SAFETY
        // - we copied over current_vec.len() elements. Specifically:
        //    - current_vec.len() did not change since the last call to add
        //      TODO watch for this when removing items, we might need to handle
        //      that here
        // - current_vec.len()..self.len elements where already
        //   initialized in Tomato::add.
        unsafe {
            self.new_vec.set_len(self.len);
        }

        std::mem::swap(current_vec, &mut self.new_vec);
        self.old_vec = Some(std::mem::take(&mut self.new_vec));
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
