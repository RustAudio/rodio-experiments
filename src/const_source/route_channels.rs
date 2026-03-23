use crate::ConstSource;

pub mod array;
pub mod tuple;
pub mod vec;

pub trait CombineChannels<const SR: u32, const CH_IN: u16> {
    type CombinerSource<const CH: u16>: ConstSource<SR, CH>;
    type Result<T>;

    fn combine_channels<const CH: u16>(self) -> Self::Result<Self::CombinerSource<CH>>;
}
