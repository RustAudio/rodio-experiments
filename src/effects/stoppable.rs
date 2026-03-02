use crate::effects::pure_effect;

pure_effect! {
    supports_dynamic_source
    struct Stoppable {
        stop: bool,
    }

    fn next(&mut self) -> Option<Sample> {
        if self.stop { None } else { self.inner.next() }
    }

    fn new(source: S) -> Amplify<Self> {
        Self {
            inner: source,
            stop: false,
        }
    }

    pub fn stop(&mut self) {
        self.stop = true
    }
}
