use crate::FixedSource;
use crate::fixed_source::queued::ParamsMismatch;
use crate::{ChannelCount, SampleRate};

mod array;
mod tuple;
mod vec;

pub trait IntoMixed {
    type TryMixedSource: FixedSource;
    type IntoMixedSource: FixedSource;

    fn try_into_mixed(self) -> Result<Self::TryMixedSource, ParamsMismatch>;
    fn into_mixed_converted(
        self,
        sample_rate: SampleRate,
        channels: ChannelCount,
    ) -> Self::IntoMixedSource;
}
