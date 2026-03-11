use std::time::Duration;

use crate::effects::pure_effect;

pure_effect! {
    struct TrackPosition {
        // u32 gives 'only' 1 day at 44.1khz mono
        samples_counted: u64,
    }

    fn next(&mut self) -> Option<Sample> {
        let sample = self.inner.next()?;
        self.samples_counted += 1;
        Some(sample)
    }

    fn new(source: S) -> TrackPosition<Self> {
        Self { inner: source, samples_counted: 0 }
    }

    pub fn get_pos(&self) -> Duration {
        let seconds = self.samples_counted
            / self.inner.sample_rate().get() as u64
            / self.inner.channels().get() as u64;
        let rest_samples = self.samples_counted
            % self.inner.sample_rate().get() as u64
            % self.inner.channels().get() as u64;
        let rest_nanos = rest_samples * 1_000_000_000 / self.inner.sample_rate().get() as u64 / self.inner.channels().get() as u64;
        Duration::new(seconds, rest_nanos as u32)
    }
}
