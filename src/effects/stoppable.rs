use crate::effects::pure_effect;

pure_effect! {
    supports_dynamic_source
    struct Stoppable {
        frame_offset: u16,
        stop: bool,
    }

    fn next(&mut self) -> Option<Sample> {
        self.frame_offset += 1;
        self.frame_offset %= self.inner.channels();
        if self.stop && self.frame_offset == self.inner.channels().get() {
            None
        } else {
            self.inner.next()
        }
    }

    fn new(source: S) -> Amplify<Self> {
        Self {
            inner: source,
            frame_offset: 0,
            stop: false,
        }
    }

    pub fn stop(&mut self) {
        self.stop = true
    }
}

// FIXME(yara) test this
