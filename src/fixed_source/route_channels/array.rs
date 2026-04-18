use crate::ChannelCount;
use crate::FixedSource;

use crate::common::channel_combined_next_body;

use super::CombineChannels;
use super::CombineChannelsError;

#[derive(Clone, Debug)]
pub struct ChannelCombiningArray<const N: usize, S> {
    channels: ChannelCount,
    sources: [S; N],
    current: u16,
}

impl<const N: usize, S: FixedSource> Iterator for ChannelCombiningArray<N, S> {
    type Item = crate::Sample;
    fn next(&mut self) -> Option<Self::Item> {
        channel_combined_next_body! {self}
    }
}

impl<const N: usize, S: FixedSource> FixedSource for ChannelCombiningArray<N, S> {
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.sources
            .iter()
            .filter_map(FixedSource::total_duration)
            .reduce(Ord::min)
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.sources
            .first()
            .expect("We do not allow an empty list (that would imply zero channels)")
            .sample_rate()
    }
}

impl<const N: usize, S: FixedSource> CombineChannels for [S; N] {
    type TryCombinerSource = ChannelCombiningArray<N, S>;

    fn try_combine_channels(self) -> Result<Self::TryCombinerSource, CombineChannelsError> {
        let channels = super::verify_params_and_determine_channel_count(self.as_slice())?;

        Ok(Self::TryCombinerSource {
            channels,
            sources: self,
            current: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::fixed_source::buffer::SamplesBuffer;
    use crate::nz;

    use super::*;

    #[test]
    fn combine_vec() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(44100), vec![1.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(44100), vec![2.0, 4.0, 5.0, 6.0]);

        assert_eq!(
            vec![1.0, 2.0, 3.0, 4.0],
            [s1, s2].try_combine_channels().unwrap().collect::<Vec<_>>()
        );
    }

    #[test]
    fn refuse_mismatch() {
        let s1 = SamplesBuffer::new(nz!(1), nz!(48000), vec![1.0, 3.0]);
        let s2 = SamplesBuffer::new(nz!(1), nz!(44100), vec![2.0, 4.0, 5.0, 6.0]);

        assert!([s1, s2].try_combine_channels().is_err());
    }
}
