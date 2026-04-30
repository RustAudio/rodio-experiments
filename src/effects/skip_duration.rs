use std::time::Duration;

use crate::effects::pure_effect;

pure_effect! {
    struct SkipDuration {}

    fn next(&mut self) -> Option<Sample> {
        self.inner.next()
    }

    fn new(mut source: S, duration: Duration) -> SkipDuration<Self> {
        let to_skip = duration.as_secs_f64() * source.sample_rate().get() as f64;
        let to_skip = to_skip.floor() as usize;

        for _ in 0..to_skip {
            if source.next().is_none() {
                break
            }
        }

        Self{
            inner: source,
        }
    }
}
