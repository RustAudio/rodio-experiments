use std::time::Duration;

use rodio::{FixedSource, Sample};

use crate::{ChannelCount, SampleRate};

pub trait TupleSourceAccess {
    /// items in the tuple
    const LEN: usize;

    fn next(&mut self, idx: usize) -> Option<f32>;
    fn total_duration(&self, idx: usize) -> Option<Duration>;
    fn channels(&self, idx: usize) -> ChannelCount;
    fn sample_rate(&self, idx: usize) -> SampleRate;
}

pub trait ConvertedTuple {
    type Converted: TupleSourceAccess;
    fn mapped(self, sample_rate: SampleRate, channels: ChannelCount) -> Self::Converted;
}

macro_rules! tuple_impl {
    ($len:literal; $($generics:ident),+; $($count:tt),*) => {
        impl<$($generics),+> TupleSourceAccess for ($($generics),+)
        where
            $($generics: crate::FixedSource),+
        {
            const LEN: usize = $len;

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

        impl<$($generics),+> ConvertedTuple for ($($generics),+)
        where
            $($generics: crate::FixedSource),+
        {
            type Converted = ( $( crate::fixed_source::MaybeConvert<$generics>),+ );

            fn mapped(
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
