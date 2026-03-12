use core::fmt;
use std::fmt::Display;

use super::conversions::channel_count::ChannelConverter;
use crate::FixedSource;
use crate::{ChannelCount, SampleRate};

pub trait IntoList {
    type TryListSource: FixedSource;
    type IntoListSource: FixedSource;

    fn try_into_list(self) -> Result<Self::TryListSource, ParamsMismatch>;
    fn into_list_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoListSource;
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub struct ParamsMismatch {
    tuple_index_that_mismatched: u8,
    sample_rate_left: SampleRate,
    channel_count_left: ChannelCount,
    sample_rate_right: SampleRate,
    channel_count_right: ChannelCount,
}

impl Display for ParamsMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParamsMismatch {
            tuple_index_that_mismatched,
            sample_rate_left,
            channel_count_left,
            sample_rate_right,
            channel_count_right,
        } = self;
        f.write_fmt(format_args!("Parameters mismatch, the left {} sources in the tuple have sample rate: {sample_rate_left} and channel count: {channel_count_left} the next source has sample rate: {sample_rate_right} and {channel_count_right} which are not the same", tuple_index_that_mismatched-1))
    }
}

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*; $last:tt) => {
        #[derive(Clone, Debug)]
        pub struct $list<$($generics),+> {
            sources: ($($generics),+),
            current: u8,
        }

        impl<$($generics: FixedSource),+> Iterator for $list<$($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match self.current {
                        $($count => {
                            let sample = self.sources.$count.next();
                            if sample.is_some() {
                                print!("{}", sample.unwrap() as u8);
                                return sample;
                            } else {
                                println!("EXHAUSTED MOVING TO NEXT");
                                self.current = $count + 1;
                                continue
                            }
                        }),*
                        $last => return self.sources.$last.next(),
                        _ => unreachable!("self.current is never increased beyond 1"),
                    } // match
                } // loop
            } // next
        } // impl iter

        impl<$($generics: FixedSource),+> FixedSource for $list<$($generics),+> {
            fn channels(&self) -> rodio::ChannelCount {
                self.sources.0.channels()
            }

            fn sample_rate(&self) -> rodio::SampleRate {
                self.sources.0.sample_rate()
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                self.sources
                    .0
                    .total_duration()
                    .and_then(|d0| self.sources.1.total_duration().map(|d1| d1 + d0))
            }
        } // impl FixedSource

        impl<$($generics: FixedSource),+> IntoList for ($($generics),+) {
            type TryListSource = $list<$($generics),+>;
            type IntoListSource = $list<$(MaybeConvert<$generics>),+>;

            fn try_into_list(self) -> Result<Self::TryListSource, ParamsMismatch> {
                let sample_rate_left = self.0.sample_rate();
                let channel_count_left = self.0.channels();

                $(
                let tuple_index_that_mismatched = $count;
                let sample_rate_right = self.$count.sample_rate();
                let channel_count_right = self.$count.channels();

                if sample_rate_left != sample_rate_right || channel_count_left != channel_count_right {
                    return Err(ParamsMismatch {
                        tuple_index_that_mismatched,
                        sample_rate_left,
                        channel_count_left,
                        sample_rate_right,
                        channel_count_right,
                    });
                }
                )+

                Ok($list {
                    sources: self,
                    current: 0,
                })
            }

            fn into_list_converted(
                self,
                sample_rate: SampleRate,
                channels: ChannelCount,
            ) -> Self::IntoListSource{

                let sources = (
                $(
                    convert_if_needed(self.$count, sample_rate, channels),
                )+
                    convert_if_needed(self.$last, sample_rate, channels)
                );

                $list {
                    sources,
                    current: 0,
                }
            }
        } // impl IntoList
    }; // transcriber
}

use super::conversions::sample_rate::Resampler;
pub enum MaybeConvert<S: FixedSource> {
    OnlyResample(Resampler<S>),
    OnlyRechannel(ChannelConverter<S>),
    // more efficient when adding channels
    ResampleThenRechannel(ChannelConverter<Resampler<S>>),
    // more efficient when removing channels
    RechannelThenResamples(Resampler<ChannelConverter<S>>),
    Unchanged(S),
}

impl<S: FixedSource> fmt::Debug for MaybeConvert<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnlyResample(_) => f.debug_tuple("OnlyResample").finish_non_exhaustive(),
            Self::OnlyRechannel(_) => f.debug_tuple("OnlyRechannel").finish_non_exhaustive(),
            Self::ResampleThenRechannel(_) => f
                .debug_tuple("ResampleThenRechannel")
                .finish_non_exhaustive(),
            Self::RechannelThenResamples(_) => f
                .debug_tuple("RechannelThenResamples")
                .finish_non_exhaustive(),
            Self::Unchanged(_) => f.debug_tuple("Unchanged").finish_non_exhaustive(),
        }
    }
}

impl<S: FixedSource> FixedSource for MaybeConvert<S> {
    fn channels(&self) -> ChannelCount {
        match self {
            MaybeConvert::OnlyResample(s) => s.channels(),
            MaybeConvert::OnlyRechannel(s) => s.channels(),
            MaybeConvert::ResampleThenRechannel(s) => s.channels(),
            MaybeConvert::RechannelThenResamples(s) => s.channels(),
            MaybeConvert::Unchanged(s) => s.channels(),
        }
    }

    fn sample_rate(&self) -> SampleRate {
        match self {
            MaybeConvert::OnlyResample(s) => s.sample_rate(),
            MaybeConvert::OnlyRechannel(s) => s.sample_rate(),
            MaybeConvert::ResampleThenRechannel(s) => s.sample_rate(),
            MaybeConvert::RechannelThenResamples(s) => s.sample_rate(),
            MaybeConvert::Unchanged(s) => s.sample_rate(),
        }
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        match self {
            MaybeConvert::OnlyResample(s) => s.total_duration(),
            MaybeConvert::OnlyRechannel(s) => s.total_duration(),
            MaybeConvert::ResampleThenRechannel(s) => s.total_duration(),
            MaybeConvert::RechannelThenResamples(s) => s.total_duration(),
            MaybeConvert::Unchanged(s) => s.total_duration(),
        }
    }
}

impl<S: FixedSource> Iterator for MaybeConvert<S> {
    type Item = crate::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MaybeConvert::OnlyResample(s) => s.next(),
            MaybeConvert::OnlyRechannel(s) => s.next(),
            MaybeConvert::ResampleThenRechannel(s) => s.next(),
            MaybeConvert::RechannelThenResamples(s) => s.next(),
            MaybeConvert::Unchanged(s) => s.next(),
        }
    }
}

fn convert_if_needed<S: FixedSource>(
    s: S,
    sample_rate: SampleRate,
    channels: ChannelCount,
) -> MaybeConvert<S> {
    use crate::fixed_source::FixedSourceExt;
    use MaybeConvert as M;
    match (s.sample_rate() == sample_rate, s.channels() == channels) {
        (true, true) => M::Unchanged(s),
        (true, false) => M::OnlyRechannel(s.with_channel_count(channels)),
        (false, true) => M::OnlyResample(s.with_sample_rate(sample_rate)),
        (false, false) if s.channels() > channels => {
            M::ResampleThenRechannel(s.with_sample_rate(sample_rate).with_channel_count(channels))
        }
        (false, false) => {
            M::RechannelThenResamples(s.with_channel_count(channels).with_sample_rate(sample_rate))
        }
    }
}

tuple_impl! {List2; S1,S2; 0;1}
tuple_impl! {List3; S1,S2,S3; 0,1;2}
tuple_impl! {List4; S1,S2,S3,S4; 0,1,2;3}
tuple_impl! {List5; S1,S2,S3,S4,S5; 0,1,2,3;4}
tuple_impl! {List6; S1,S2,S3,S4,S5,S6; 0,1,2,3,4;5}
tuple_impl! {List7; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5;6}
tuple_impl! {List8; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6;7}
tuple_impl! {List9; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7;8}
tuple_impl! {List10; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8;9}
tuple_impl! {List11; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9;10}
tuple_impl! {List12; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10;11}

#[cfg(test)]
mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn list2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            (s1, s2).try_into_list().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn list3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            vec![1., 2., 3.],
            (s1, s2, s3).try_into_list().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn list3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            ParamsMismatch {
                tuple_index_that_mismatched: 1,
                sample_rate_left: nz!(1),
                channel_count_left: nz!(1),
                sample_rate_right: nz!(2),
                channel_count_right: nz!(2),
            },
            (s1, s2, s3).try_into_list().unwrap_err()
        );
    }

    #[test]
    fn into_list_converted_all_rechanneled() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]);
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 2]);

        assert_eq!(
            vec![1.0, 2.0, 3.0],
            (s1, s2, s3)
                .into_list_converted(nz!(1), nz!(1))
                .collect::<Vec<_>>()
        );
    }
}
