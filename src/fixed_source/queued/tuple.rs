use crate::FixedSource;
use crate::{ChannelCount, SampleRate};

use super::{IntoQueued, ParamsMismatch};
use super::super::convert_if_needed;
use super::super::MaybeConvert;

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

        impl<$($generics: FixedSource),+> IntoQueued for ($($generics),+) {
            type TryQueuedSource = $list<$($generics),+>;
            type IntoQueuedSource = $list<$(MaybeConvert<$generics>),+>;

            fn try_into_list(self) -> Result<Self::TryQueuedSource, ParamsMismatch> {
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
                    current: 0,
                })
            }

            fn into_list_converted(
                self,
                sample_rate: SampleRate,
                channels: ChannelCount,
            ) -> Self::IntoQueuedSource{

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

tuple_impl! {Queued2Tuple; S1,S2; 0;1}
tuple_impl! {Queued3Tuple; S1,S2,S3; 0,1;2}
tuple_impl! {Queued4Tuple; S1,S2,S3,S4; 0,1,2;3}
tuple_impl! {Queued5Tuple; S1,S2,S3,S4,S5; 0,1,2,3;4}
tuple_impl! {Queued6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4;5}
tuple_impl! {Queued7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5;6}
tuple_impl! {Queued8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6;7}
tuple_impl! {Queued9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7;8}
tuple_impl! {Queued10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8;9}
tuple_impl! {Queued11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9;10}
tuple_impl! {Queued12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10;11}

#[cfg(test)]
pub(crate) mod tests {
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
                index_of_first_mismatch: 1,
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
