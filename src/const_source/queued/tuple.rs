use super::IntoQueued;
use crate::ConstSource;
use crate::common::for_in_tuple;
use std::time::Duration;

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*) => {
        #[derive(Clone, Debug)]
        pub struct $list<const SR: u32, const CH: u16, $($generics),+> {
            sources: ($($generics),+),
            current: u8,
        }

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+>
            Iterator for $list<SR, CH, $($generics),+> {

            type Item = crate::Sample;

            fn next(&mut self) -> Option<Self::Item> {
                #![expect(unused, reason = "index += 1 is not used in the last iteration")]

                let mut index = 0;
                for_in_tuple! { $($count),*;
                    for mut source in self.sources; {
                        if index >= self.current {
                            if let Some(sample) = source.next() {
                                self.current = index;
                                return Some(sample)
                            }
                        }
                        index += 1;
                    }
                }
                None
            } // next
        } // impl iter

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+>
            ConstSource<SR, CH> for $list<SR, CH, $($generics),+> {

            fn total_duration(&self) -> Option<std::time::Duration> {
                let mut total_duration = Some(Duration::ZERO);
                for_in_tuple! { $($count),*;
                    for source in self.sources; {
                        total_duration = total_duration
                            .and_then(|d| source.total_duration()
                                .map(|sd| d + sd))
                    }
                }
                total_duration
            }
        } // impl FixedSource

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+> IntoQueued<SR, CH> for ($($generics),+) {
            type QueuedSource = $list<SR, CH, $($generics),+>;

            fn into_list(self) -> Self::QueuedSource {
                $list {
                    sources: self,
                    current: 0,
                }
            }
        } // impl IntoList
    }; // transcriber
}

tuple_impl! {Queued2Tuple; S1,S2; 0,1}
tuple_impl! {Queued3Tuple; S1,S2,S3; 0,1,2}
tuple_impl! {Queued4Tuple; S1,S2,S3,S4; 0,1,2,3}
tuple_impl! {Queued5Tuple; S1,S2,S3,S4,S5; 0,1,2,3,4}
tuple_impl! {Queued6Tuple; S1,S2,S3,S4,S5,S6; 0,1,2,3,4,5}
tuple_impl! {Queued7Tuple; S1,S2,S3,S4,S5,S6,S7; 0,1,2,3,4,5,6}
tuple_impl! {Queued8Tuple; S1,S2,S3,S4,S5,S6,S7,S8; 0,1,2,3,4,5,6,7}
tuple_impl! {Queued9Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9; 0,1,2,3,4,5,6,7,8}
tuple_impl! {Queued10Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10; 0,1,2,3,4,5,6,7,8,9}
tuple_impl! {Queued11Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11; 0,1,2,3,4,5,6,7,8,9,10}
tuple_impl! {Queued12Tuple; S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,S12; 0,1,2,3,4,5,6,7,8,9,10,11}

#[cfg(test)]
pub(crate) mod tests {
    use crate::const_source::buffer::SamplesBuffer;

    use super::*;

    #[test]
    fn queue2() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            (s1, s2).into_list().collect::<Vec<_>>()
        );
    }

    #[test]
    fn duration3() {
        let s1 = SamplesBuffer::<1, 1>::new(vec![1.0; 3]);
        let s2 = SamplesBuffer::<1, 1>::new(vec![1.0; 1]);
        let s3 = SamplesBuffer::<1, 1>::new(vec![1.0; 2]);

        let mut queued = (s1, s2, s3).into_list();

        assert_eq!(Some(Duration::from_secs(6)), queued.total_duration());

        let _ = queued.by_ref().take(4).count();
        assert_eq!(Some(Duration::from_secs(6)), queued.total_duration());
    }
}
