// Placeholder for rodio Source trait right now we can only extend it (orphan
// rule) which happens dynamic_source_ext. That will be removed and the code
// will live under Source here directly. Open question: do we rename Source to
// DynamicSource?


macro_rules! add_inner_methods {
    ($name:ident$(<$t:ident>)?) => {
        impl<S: crate::Source$(,$t)?> $name<S$(,$t)?> {
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

macro_rules! impl_wrapper {
    ($name:ident$(<$t:ident>)?) => {
        impl<S: crate::Source$(,$t)?> crate::Source for $name<S$(,$t)?> {
            fn current_span_len(&self) -> Option<usize> {
                self.inner.current_span_len()
            }

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

pub(crate) use add_inner_methods;
pub(crate) use impl_wrapper;
