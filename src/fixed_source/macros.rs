macro_rules! tuple_impl {
    ($list:ident; $($generics:ident),+; $($count:tt),*; $last:tt;
    $source_trait:ident$(<$sr:ident,$ch:ident>)?;
    fn next(&mut $self:ident) -> Option<Sample> $body:block) => {
        #[derive(Clone, Debug)]
        pub struct $list<$($generics),+> {
            sources: ($($generics),+),
            current: u8,
        }

        impl<$($generics: FixedSource),+> Iterator for $list<$($generics),+> {
            type Item = crate::Sample;

            fn next(&mut $self) -> Option<Self::Item> {
                $body
            }
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

pub(crate) use tuple_impl;

macro_rules! add_inner_methods {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<S: crate::FixedSource $(,$t$(:$bound)?)?> $name<S $(,$t)?> {
            pub fn inner(&self) -> &S {
                &self.inner
            }
            pub fn inner_mut(&mut self) -> &mut S {
                &mut self.inner
            }
            pub fn into_inner(self) -> S {
                self.inner
            }
        }
    };
}

pub(crate) use add_inner_methods;

macro_rules! impl_wrapper {
    ($name:ident$(<$t:ident$(:$bound:path)?>)?) => {
        impl<S: crate::FixedSource $(,$t$(:$bound)?)?> crate::FixedSource for $name<S $(,$t)?> {
            fn channels(&self) -> rodio::ChannelCount {
                self.inner.channels()
            }

            fn sample_rate(&self) -> rodio::SampleRate {
                self.inner.sample_rate()
            }

            fn total_duration(&self) -> Option<std::time::Duration> {
                self.inner.total_duration()
            }
        }
    };
}
pub(crate) use impl_wrapper;
