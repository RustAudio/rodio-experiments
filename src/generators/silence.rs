pub mod fixed_source {
    use crate::FixedSource;
    use crate::{ChannelCount, SampleRate};

    super::silence_docs! {"fixed_source"
    pub struct Silence {
        channel_count: ChannelCount,
        sample_rate: SampleRate,
    }
    }

    impl FixedSource for Silence {
        fn channels(&self) -> ChannelCount {
            self.channel_count
        }

        fn sample_rate(&self) -> SampleRate {
            self.sample_rate
        }

        fn total_duration(&self) -> Option<std::time::Duration> {
            None
        }
    }

    impl Iterator for Silence {
        type Item = crate::Sample;

        fn next(&mut self) -> Option<Self::Item> {
            Some(0.0)
        }
    }
}

pub mod const_source {
    use crate::ConstSource;

    super::silence_docs! {"const_source"
    pub struct Silence<const SR: u32, const CH: u16>;
    }

    impl<const SR: u32, const CH: u16> ConstSource<SR, CH> for Silence<SR, CH> {
        fn total_duration(&self) -> Option<std::time::Duration> {
            None
        }
    }

    impl<const SR: u32, const CH: u16> Iterator for Silence<SR, CH> {
        type Item = crate::Sample;

        fn next(&mut self) -> Option<Self::Item> {
            Some(0.0)
        }
    }
}

macro_rules! silence_docs {
    ($mod:literal $struct:item) => {
        #[doc = concat!("\
A source producing an infinite amount of Silence. Like all generators you
probably want to limit the duration of this source.

# Example
Padding a [`TakeDuration`](crate::effects::", $mod, "::TakeDuration) to 
guarantee an exact playtime:

```rust,no_run
# use rodio_experiments::generators::functions::", $mod, "::SineWave;
# let unknown_length = SineWave::new(440);
let two_seconds = unknown_length
    .chain(silence)
    .take_duration(Duration::from_secs(2));
```
")]
        $struct
    };
}
pub(crate) use silence_docs;
