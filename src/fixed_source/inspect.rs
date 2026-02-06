use rodio::{FixedSource, Sample};

pub struct InspectFrame<S: FixedSource, F> {
    inner: S,
    f: F,
    frame: Vec<Sample>,
}

impl<S: FixedSource, F: FnMut(Vec<Sample>) -> Vec<Sample>> InspectFrame<S, F> {
    pub(crate) fn new(inner: S, f: F) -> Self {
        let mut frame = Vec::new();
        frame.reserve_exact(inner.channels().get() as usize);
        Self { inner, f, frame }
    }
}

// Ideally the FnMut should borrow a slice, unfortunately we can not borrow from
// self as we can not name the lifetime
impl<S: FixedSource, F: FnMut(Vec<Sample>) -> Vec<Sample>> Iterator for InspectFrame<S, F> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        self.frame.push(sample);
        if self.frame.len() == self.inner.channels().get().into() {
            self.frame = (self.f)(std::mem::take(&mut self.frame));
            self.frame.clear();
        }
        Some(sample)
    }
}

impl<S: FixedSource, F: FnMut(Vec<Sample>) -> Vec<Sample>> FixedSource for InspectFrame<S, F> {
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
