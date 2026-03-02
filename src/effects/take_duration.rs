use std::time::Duration;

use crate::effects::pure_effect;

pure_effect! {
    struct TakeDuration {
        left: usize,
    }

    fn next(&mut self) -> Option<Sample> {
        if self.left > 0 {
            self.left -= 1;
            self.inner.next()
        } else {
            None
        }
    }

    fn new(source: S, duration: Duration) -> TakeDuration<Self> {
        let left = duration.as_secs_f64() * source.sample_rate().get() as f64;
        let left = left.ceil() as usize;
        Self{
            inner: source,
            left,
        }
    }
}
