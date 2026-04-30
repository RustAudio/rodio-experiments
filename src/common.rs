#![allow(unused)]
use std::fmt::{Debug, Display};
use std::num::NonZero;

pub(crate) mod mixer;
pub(crate) mod queue;

/// Sample rate (a frame rate or samples per second per channel).
pub type SampleRate = NonZero<u32>;

/// Number of channels in a stream. Can never be Zero
pub type ChannelCount = NonZero<u16>;

/// Number of bits per sample. Can never be zero.
pub type BitDepth = NonZero<u32>;

// NOTE on numeric precision:
//
// While `f32` is transparent for typical playback use cases, it does not guarantee preservation of
// full 24-bit source fidelity across arbitrary processing chains. Each floating-point operation
// rounds its result to `f32` precision (~24-bit significand). In DSP pipelines (filters, mixing,
// modulation), many operations are applied per sample and over time, so rounding noise accumulates
// and long-running state (e.g. oscillator phase) can drift.
//
// For use cases where numerical accuracy must be preserved through extended processing (recording,
// editing, analysis, long-running generators, or complex DSP graphs), enabling 64-bit processing
// reduces accumulated rounding error and drift.
//
// This mirrors common practice in professional audio software and DSP libraries, which often use
// 64-bit internal processing even when the final output is 16- or 24-bit.

/// Floating point type used for internal calculations. Can be configured to be
/// either `f32` (default) or `f64` using the `64bit` feature flag.
#[cfg(not(feature = "64bit"))]
pub type Float = f32;

/// Floating point type used for internal calculations. Can be configured to be
/// either `f32` (default) or `f64` using the `64bit` feature flag.
#[cfg(feature = "64bit")]
pub type Float = f64;

/// Represents value of a single sample.
/// Silence corresponds to the value `0.0`. The expected amplitude range is  -1.0...1.0.
/// Values below and above this range are clipped in conversion to other sample types.
/// Use conversion traits from [dasp_sample] crate or [rodio::conversions::SampleTypeConverter]
/// to convert between sample types if necessary.
pub type Sample = Float;

/// Used to test at compile time that a struct/enum implements Send, Sync and
/// is 'static. These are common requirements for dynamic error management
/// libs like color-eyre and anyhow
///
/// # Examples
/// ```compile_fail
/// struct NotSend {
///   foo: Rc<String>,
/// }
///
/// assert_error_traits!(NotSend)
/// ```
///
/// ```compile_fail
/// struct NotSync {
///   foo: std::cell::RefCell<String>,
/// }
/// assert_error_traits!(NotSync)
/// ```
///
/// ```compile_fail
/// struct NotStatic<'a> {
///   foo: &'a str,
/// }
///
/// assert_error_traits!(NotStatic)
/// ```
macro_rules! assert_error_traits {
    ($to_test:path) => {
        const _: () = { $crate::common::use_required_traits::<$to_test>() };
    };
}

pub(crate) use assert_error_traits;
#[allow(dead_code)]
pub(crate) const fn use_required_traits<T: Send + Sync + 'static + Display + Debug + Clone>() {}

// Note: if you change this you also need to change the tuple impl!!
macro_rules! mixed_next_body {
    ($self:ident) => {
        let (sum, summed) = $self
            .sources
            .iter_mut()
            .filter_map(|source| source.next())
            .map(|sample| sample as f64)
            .zip((1usize..).into_iter())
            .reduce(|(sum, _), (sample, summed)| (sum + sample, summed))?;
        Some((sum / summed as f64) as crate::Float)
    };
}
pub(crate) use mixed_next_body;

// Note: if you change this you also need to change the tuple impl!!
macro_rules! queued_next_body {
    ($self:ident) => {
        loop {
            if let Some(sample) = $self.sources.get_mut($self.current)?.next() {
                return Some(sample);
            }
            $self.current += 1;
        }
    };
}
pub(crate) use queued_next_body;

// Note: if you change this you also need to change the tuple impl!!
macro_rules! channel_combined_next_body {
    ($self:ident) => {
        let channels = $self.channels().get();
        let mut channel = 0;
        for item in &mut $self.sources {
            if (channel..(channel + item.channels().get())).contains(&$self.current) {
                $self.current += 1;
                $self.current %= channels;
                return item.next();
            } else {
                channel += item.channels().get()
            }
        }
        None
    };
}
pub(crate) use channel_combined_next_body;

macro_rules! check_params_for_list {
    ($self:ident) => {
        let mut list = $self.iter().map(|s| (s.sample_rate(), s.channels()));
        if let Some(first) = list.next() {
            if let Some((pos, (sample_rate_right, channel_count_right))) =
                list.find_position(|params| *params != first)
            {
                return Err(ParamsMismatch {
                    index_of_first_mismatch: pos,
                    sample_rate_left: first.0,
                    channel_count_left: first.1,
                    sample_rate_right,
                    channel_count_right,
                });
            }
        };
    };
}
pub(crate) use check_params_for_list;

macro_rules! for_in_tuple {
    ($($index:tt),+;
     for $tuple_item:ident in $tuple:expr; $do:block
     ) => {
        $(
            let $tuple_item = &$tuple.$index;
            $do
        )+
    };
    ($($index:tt),+;
     for mut $tuple_item:ident in $tuple:expr; $do:block
     ) => {
        $(
            let $tuple_item = &mut $tuple.$index;
            $do
        )+
    };
}
pub(crate) use for_in_tuple;

macro_rules! make_params_mismatch_error {
    ($combiner:literal, $combiner_handle_ty:literal) => {
        #[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
        pub struct ParamsMismatch {
            pub(crate) sample_rate_mixer: crate::SampleRate,
            pub(crate) channel_count_mixer: crate::ChannelCount,
            pub(crate) sample_rate_new: crate::SampleRate,
            pub(crate) channel_count_new: crate::ChannelCount,
        }

        impl std::fmt::Display for ParamsMismatch {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let ParamsMismatch {
                    sample_rate_mixer,
                    channel_count_mixer,
                    sample_rate_new,
                    channel_count_new,
                } = self;
                f.write_fmt(format_args!("Parameters mismatch, the {} is set up with sample rate: {sample_rate_mixer} and channel count: {channel_count_mixer}. You are trying to add a source with sample rate: {sample_rate_new} and {channel_count_new}. Try using `{}::add_converted` instead", $combiner, $combiner_handle_ty))
            }
        }
    }
}
pub(crate) use make_params_mismatch_error;

pub(crate) struct SourceShare<S, K> {
    capacity: usize,
    len: usize,
    to_remove: Vec<K>,
    /// we do not allocate on the audio thread
    /// instead the handle does that
    new_vec: Vec<(S, K)>,
    /// we do not deallocate on the audio thread
    /// instead the handle does that
    old_vec: Option<Vec<(S, K)>>,
}

impl<S, K: PartialEq> SourceShare<S, K> {
    pub(crate) fn schedule_addition(&mut self, source: S, key: K) {
        let _drop = self.old_vec.take();

        if self.len + 1 >= self.capacity {
            self.capacity *= 2;
        }
        self.new_vec
            .reserve(self.capacity.saturating_sub(self.new_vec.capacity()));
        self.new_vec.push((source, key));
    }

    pub(crate) fn update(&mut self, current_vec: &mut Vec<(S, K)>) {
        current_vec.retain(|(_, key)| !self.to_remove.contains(key));

        swap_append(current_vec, &mut self.new_vec);

        self.old_vec = Some(std::mem::take(&mut self.new_vec));
    }

    pub(crate) fn schedule_removal(&mut self, key: K) {
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
