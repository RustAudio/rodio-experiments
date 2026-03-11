pub mod const_source;
pub mod fixed_source;

macro_rules! chirp_docs {
    ($struct:item) => {
        #[doc = "Generate a sine wave with an instantaneous frequency that changes/sweeps
        linearly over time. At the end of the chirp, once the `end_frequency` is
        reached, the source is exhausted."]
        $struct
    };
}
pub(crate) use chirp_docs;
