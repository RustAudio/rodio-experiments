use crate::FixedSource;
use crate::{ChannelCount, SampleRate};

use super::super::MaybeConvert;
use super::super::convert_if_needed;
use super::{IntoMixed, ParamsMismatch};

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*; $last:tt) => {
        #[derive(Clone, Debug)]
        pub struct $list<$($generics),+> {
            sources: ($($generics),+),
        }

        impl<$($generics: FixedSource),+> Iterator for $list<$($generics),+> {
            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                let mut samples = 0u8;
                let mut sum = None;
                $(
                    let sample = self.sources.$count.next().map(|s| s as f64);
                    samples += sample.is_some() as u8;
                    sum = match [sum, sample] {
                        [Some(sum), None] => Some(sum),
                        [Some(sum), Some(sample)] => Some(sum + sample),
                        [None, Some(sample)] => Some(sample),
                        [None, None] => None,
                    };
                )*
                let sample = self.sources.$last.next().map(|s| s as f64);
                samples += sample.is_some() as u8;
                sum = match [sum, sample] {
                    [Some(sum), None] => Some(sum),
                    [Some(sum), Some(sample)] => Some(sum + sample),
                    [None, Some(sample)] => Some(sample),
                    [None, None] => None,
                };
                sum.map(|sum| sum / samples as f64).map(|sum| sum as f32)
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
                let mut max = None;
                $(
                    let dur= self.sources.$count.total_duration();
                    max = match [max, dur] {
                        [Some(max), None] => Some(max),
                        [Some(max), Some(dur)] => Some(max.max(dur)),
                        [None, Some(dur)] => Some(dur),
                        [None, None] => None,
                    };
                )*
                let dur= self.sources.$last.total_duration();
                max = match [max, dur] {
                    [Some(max), None] => Some(max),
                    [Some(max), Some(dur)] => Some(max.max(dur)),
                    [None, Some(dur)] => Some(dur),
                    [None, None] => None,
                };
                max
            }
        } // impl FixedSource

        impl<$($generics: FixedSource),+> IntoMixed for ($($generics),+) {
            type TryMixedSource = $list<$($generics),+>;
            type IntoMixedSource = $list<$(MaybeConvert<$generics>),+>;

            fn try_into_mixed(self) -> Result<Self::TryMixedSource, ParamsMismatch> {
                let sample_rate_left = self.0.sample_rate();
                let channel_count_left = self.0.channels();

                $(
                let tuple_index_that_mismatched = $count;
                let sample_rate_right = self.$count.sample_rate();
                let channel_count_right = self.$count.channels();

                if sample_rate_left != sample_rate_right || channel_count_left != channel_count_right {
                    return Err(ParamsMismatch {
                        index_of_first_mismatch: tuple_index_that_mismatched as usize,
                        sample_rate_left,
                        channel_count_left,
                        sample_rate_right,
                        channel_count_right,
                    });
                }
                )+

                Ok($list {
                    sources: self,
                })
            }

            fn into_mixed_converted(
                self,
                sample_rate: SampleRate,
                channels: ChannelCount,
            ) -> Self::IntoMixedSource{

                let sources = (
                $(
                    convert_if_needed(self.$count, sample_rate, channels),
                )+
                    convert_if_needed(self.$last, sample_rate, channels)
                );

                $list {
                    sources,
                }
            }
        } // impl IntoList
    }; // transcriber
}

tuple_impl! {Mixed2Tuple; S1,S2; 0;1}
tuple_impl! {Mixed3Tuple; S1,S2,S3; 0,1;2}
tuple_impl! {Mixed4Tuple; S1,S2,S3,S4; 0,1,2;3}
tuple_impl! {Mixed5Tuple; S1,S2,S3,S4,S5; 0,1,2,3;4}
tuple_impl! {Mixed6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4;5}
tuple_impl! {Mixed7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5;6}
tuple_impl! {Mixed8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6;7}
tuple_impl! {Mixed9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7;8}
tuple_impl! {Mixed10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8;9}
tuple_impl! {Mixed11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9;10}
tuple_impl! {Mixed12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10;11}

#[cfg(test)]
pub(crate) mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn mixed2() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![2.5, 3.5, 4.5],
            (s1, s2).try_into_mixed().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn mixed3() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(1), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert_eq!(
            vec![2.],
            (s1, s2, s3).try_into_mixed().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn mixed3_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0]);
        let s2 = SamplesBuffer::new(nz!(2), nz!(2), vec![2.0]);
        let s3 = SamplesBuffer::new(nz!(1), nz!(1), vec![3.0]);

        assert!((s1, s2, s3).try_into_mixed().is_err());
    }

    #[test]
    fn into_mixed_converted_all_rechanneled() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(1), vec![1.0; 1]);
        let s2 = SamplesBuffer::new(nz!(3), nz!(1), vec![2.0; 3]);
        let s3 = SamplesBuffer::new(nz!(2), nz!(1), vec![3.0; 2]);

        assert_eq!(
            vec![2.0],
            (s1, s2, s3)
                .into_mixed_converted(nz!(1), nz!(1))
                .collect::<Vec<_>>()
        );
    }
}
