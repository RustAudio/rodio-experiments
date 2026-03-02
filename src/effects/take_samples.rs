use crate::effects::pure_effect;

pure_effect! {
    struct TakeSamples {
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

    fn new(source: S, samples: usize) -> TakeSamples<Self> {
        Self{
            inner: source,
            left: samples,
        }
    }
}
