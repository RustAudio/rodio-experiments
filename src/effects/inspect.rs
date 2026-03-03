use crate::effects::pure_effect;

pure_effect! {
    struct InspectFrame<F: FnMut(Vec<crate::Sample>) -> Vec<crate::Sample>> {
        f: F,
        frame: Vec<crate::Sample>,
    }

    fn next(&mut self) -> Option<Sample> {
        let sample = self.inner.next()?;
        self.frame.push(sample);
        if self.frame.len() == self.inner.channels().get().into() {
            self.frame = (self.f)(std::mem::take(&mut self.frame));
            self.frame.clear();
        }
        Some(sample)
    }

    fn new(inner: S, f: F) -> InspectFrame<Self> {
        let mut frame = Vec::new();
        frame.reserve_exact(inner.channels().get() as usize);
        Self { inner, f, frame }
    }
}
