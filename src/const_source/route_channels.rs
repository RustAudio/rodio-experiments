use crate::ConstSource;

// Three separate implementations since:
// - for array we want to statically check that the channel sum is correct
// - for tuple we do the same (but it requires a different implementation).
// - the vec needs runtime verification.
pub mod array;
pub mod tuple;
pub mod vec;

pub trait CombineChannels<const SR: u32, const CH_IN: u16> {
    type CombinerSource<const CH: u16>: ConstSource<SR, CH>;
    // This is just T for an array or tuple which makes the function
    // return just the CombinerSource.
    type Result<T>; 

    fn combine_channels<const CH: u16>(self) -> Self::Result<Self::CombinerSource<CH>>;
}
