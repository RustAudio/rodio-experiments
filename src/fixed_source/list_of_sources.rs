use std::time::Duration;

use rodio::FixedSource;

use crate::fixed_source::{MaybeConvert, convert_if_needed};
use crate::{ChannelCount, SampleRate};

pub trait ListOfSources {
    /// number of sources in the "list"
    fn len(&self) -> usize;

    /// get the next sample for the idx-th source in the "list"
    fn next(&mut self, idx: usize) -> Option<f32>;
    fn total_duration(&self, idx: usize) -> Option<Duration>;
    fn channels(&self, idx: usize) -> ChannelCount;
    fn sample_rate(&self, idx: usize) -> SampleRate;
}

impl<const N: usize, S: FixedSource> ListOfSources for [S; N] {
    fn len(&self) -> usize {
        N
    }

    fn next(&mut self, idx: usize) -> Option<f32> {
        self[idx].next()
    }

    fn total_duration(&self, idx: usize) -> Option<Duration> {
        self[idx].total_duration()
    }

    fn channels(&self, idx: usize) -> ChannelCount {
        self[idx].channels()
    }

    fn sample_rate(&self, idx: usize) -> SampleRate {
        self[idx].sample_rate()
    }
}

impl<S: FixedSource> ListOfSources for Vec<S> {
    fn len(&self) -> usize {
        self.len()
    }

    fn next(&mut self, idx: usize) -> Option<f32> {
        self[idx].next()
    }

    fn total_duration(&self, idx: usize) -> Option<Duration> {
        self[idx].total_duration()
    }

    fn channels(&self, idx: usize) -> ChannelCount {
        self[idx].channels()
    }

    fn sample_rate(&self, idx: usize) -> SampleRate {
        self[idx].sample_rate()
    }
}

pub trait ConvertedListOfSources {
    type Converted: ListOfSources;
    fn converted(self, sample_rate: SampleRate, channels: ChannelCount) -> Self::Converted;
}

impl<const N: usize, S: FixedSource> ConvertedListOfSources for [S; N] {
    type Converted = [MaybeConvert<S>; N];
    fn converted(self, sample_rate: SampleRate, channels: ChannelCount) -> Self::Converted {
        self.map(|source| convert_if_needed(source, sample_rate, channels))
    }
}

impl<S: FixedSource> ConvertedListOfSources for Vec<S> {
    type Converted = Vec<MaybeConvert<S>>;
    fn converted(self, sample_rate: SampleRate, channels: ChannelCount) -> Self::Converted {
        self.into_iter()
            .map(|source| convert_if_needed(source, sample_rate, channels))
            .collect()
    }
}

macro_rules! tuple_impl {
    ($len:literal; $($generics:ident),+; $($count:tt),*) => {
        impl<$($generics),+> ListOfSources for ($($generics),+)
        where
            $($generics: crate::FixedSource),+
        {
            fn len(&self) -> usize {
                $len
            }

            fn next(&mut self, idx: usize) -> Option<crate::Sample> {
                match idx {
                    $($count => self.$count.next(),)*
                    _ => unreachable!(),
                }
            }

            fn sample_rate(&self, idx: usize) -> crate::SampleRate {
                match idx {
                    $($count => self.$count.sample_rate(),)*
                    _ => unreachable!(),
                }
            }

            fn channels(&self, idx: usize) -> crate::ChannelCount {
                match idx {
                    $($count => self.$count.channels(),)*
                    _ => unreachable!(),
                }
            }

            fn total_duration(&self, idx: usize) -> Option<std::time::Duration> {
                match idx {
                    $($count => self.$count.total_duration(),)*
                    _ => unreachable!(),
                }
            }
        } // impl trait

        impl<$($generics),+> ConvertedListOfSources for ($($generics),+)
        where
            $($generics: crate::FixedSource),+
        {
            type Converted = ( $( crate::fixed_source::MaybeConvert<$generics>),+ );

            fn converted(
                self,
                sample_rate: SampleRate,
                channels: ChannelCount,
            ) -> Self::Converted {
                (
                    $(
                        crate::fixed_source::convert_if_needed(
                            self.$count,
                            sample_rate,
                            channels
                        ),
                    )+
                )
            }
        }


    } // transcriber
} // macro rules

tuple_impl! {2; S1,S2; 0,1}
tuple_impl! {3; S1,S2,S3; 0,1,2}
tuple_impl! {4; S1,S2,S3,S4; 0,1,2,3}
tuple_impl! {5; S1,S2,S3,S4,S5; 0,1,2,3,4}
tuple_impl! {6; S1,S2,S3,S4,S5,S6; 0,1,2,3,4,5}
tuple_impl! {7; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5,6}
tuple_impl! {8; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6,7}
tuple_impl! {9; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7,8}
tuple_impl! {10; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8,9}
tuple_impl! {11; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9,10}
tuple_impl! {12; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10,11}
