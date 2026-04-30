use crate::effects::pure_effect;

pure_effect! {
    supports_dynamic_source
    struct SkipSamples {}

    fn next(&mut self) -> Option<Sample> {
        self.inner.next()
    }

    fn new(mut source: S, samples: usize) -> SkipSamples<Self> {
        for _ in 0..samples {
            if source.next().is_none() {
                break
            }
        }

        Self{
            inner: source,
        }
    }
}
