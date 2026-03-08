use crate::Float;
use crate::effects::pure_effect;

pure_effect! {
    supports_dynamic_source
    struct Distortion {
        gain: Float,
        threshold: Float,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next().map(|value| {
            let v = value * self.gain;
            let t = self.threshold;
            v.clamp(-t, t)
        })
    }

    fn new(source: S, gain: Float, threshold: Float) -> Amplify<Self> {
        Self {
            inner: source,
            gain,
            threshold,
        }
    }

    pub fn set_gain(&mut self, gain: Float) {
        self.gain = gain;
    }

    pub fn set_threshold(&mut self, threshold: Float) {
        self.threshold = threshold;
    }
}
