use super::pure_effect;

pure_effect! {
    supports_dynamic_source
    struct Pausable {
        paused: bool,
    }

    fn next(&mut self) -> Option<Sample> {
        if self.paused {
            Some(0.0)
        } else {
            self.inner.next()
        }
    }

    fn new(source: S, paused: bool) -> Pausable<Self> {
        Self {
            inner: source,
            paused,
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
}
