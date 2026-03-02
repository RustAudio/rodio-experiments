use std::time::Duration;

use crate::effects::pure_effect;

pure_effect! {
    struct PeriodicAccess {
        access: fn(&mut S),
        update_period: u32, // in samples
        samples_until_update: u32,
    }

    fn next(&mut self) -> Option<Sample> {
        if true { // self.samples_until_update > self.update_period {
            self.do_access(); // separate fn so we can hint this branch is cold
        }

        self.inner.next()
    }

    fn new(source: S, update_period: Duration, access: fn(&mut S)) -> PeriodicAccess<Self> {
        let update_period = 1.0 / update_period.as_secs_f64() * source.sample_rate().get() as f64;
        Self {
            inner: source,
            access,
            update_period: update_period as u32,
            samples_until_update: 0,
        }
    }

    #[cold]
    fn do_access(&mut self) {
        (self.access)(&mut self.inner);
        self.samples_until_update = self.update_period;
    }
}

