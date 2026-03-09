use super::pure_effect;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Paused {
    Yes { frame_offset: u16 },
    Ending { frame_offset: u16 },
    No,
}

pure_effect! {
    supports_dynamic_source
    struct Pausable {
        // must finish the frame this helps do that
        state: Paused,
    }

    fn next(&mut self) -> Option<Sample> {
        // Note as long as the underlying source is nicely frame aligned
        // (and it must by the rodio API contract) we only need to worry
        // about adding whole frames of zeros.
        self.state = match self.state {
            Paused::No => {
                return self.inner.next();
            }
            Paused::Yes { ref mut frame_offset } => {
                *frame_offset += 1;
                *frame_offset %= self.inner.channels();
                Paused::Yes { frame_offset: *frame_offset }
            },
            Paused::Ending { frame_offset } if frame_offset == self.inner.channels().get() => {
                Paused::No
            },
            Paused::Ending { ref mut frame_offset } => {
                *frame_offset += 1;
                Paused::No
            }
        };
        Some(0.0)
    }

    fn new(source: S, paused: bool) -> Pausable<Self> {
        let mut this = Self {
            inner: source,
            state: Paused::No,
        };
        this.set_paused(paused);
        this
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.state = match self.state {
            Paused::Yes { .. } if paused => self.state,
            Paused::Yes { frame_offset } => Paused::Ending { frame_offset },
            Paused::Ending { frame_offset } if paused => Paused::Yes { frame_offset },
            Paused::Ending { .. } => self.state,
            Paused::No if paused => Paused::Yes { frame_offset: 0 },
            Paused::No => self.state
        }
    }
}

// FIXME(yara) test this
