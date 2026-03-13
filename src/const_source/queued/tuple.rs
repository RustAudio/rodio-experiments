use super::IntoQueued;
use crate::ConstSource;

macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*; $last:tt) => {
        #[derive(Clone, Debug)]
        pub struct $list<const SR: u32, const CH: u16, $($generics),+> {
            sources: ($($generics),+),
            current: u8,
        }

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+>
            Iterator for $list<SR, CH, $($generics),+> {

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

        impl<const SR: u32, const CH: u16, $($generics: ConstSource<SR, CH>),+>
            ConstSource<SR, CH> for $list<SR, CH, $($generics),+> {

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
    use crate::const_source::buffer::SamplesBuffer;

    use super::*;

    #[test]
    fn list2() {
        let s1 = SamplesBuffer::<44100, 1>::new(vec![1.0, 2.0, 3.0]);
        let s2 = SamplesBuffer::<44100, 1>::new(vec![4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1., 2., 3., 4., 5., 6.],
            (s1, s2).into_list().collect::<Vec<_>>()
        );
    }
}
