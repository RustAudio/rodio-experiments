use crate::effects::pure_effect;

use crate::Float;
use crate::Sample;

pure_effect! {
    struct ChannelVolume {
        channel_volumes: Vec<Float>,
        current_channel: usize,
        current_sample: Option<Sample>,
    }

    fn next(&mut self) -> Option<Sample> {
        if self.current_channel >= self.channel_volumes.len() {
            self.current_channel = 0;
            self.current_sample = None;
            let num_channels = self.inner().channels();
            for _ in 0..num_channels.get() {
                if let Some(s) = self.inner.next() {
                    self.current_sample = Some(self.current_sample.unwrap_or(0.0) + s);
                }
            }
            self.current_sample.map(|s| s / num_channels.get() as Float);
        }
        let result = self
            .current_sample
            .map(|s| s * self.channel_volumes[self.current_channel]);
        self.current_channel += 1;
        result
    }

    fn new(source: S, channel_volumes: Vec<Float>) -> Amplify<Self> {
        Self {
            inner: source,
            channel_volumes,
            current_channel: 0,
            current_sample: None,
        }
    }
}
