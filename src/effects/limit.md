 Applies limiting to prevent audio peaks from exceeding a threshold.

 A limiter reduces the amplitude of audio signals that exceed a specified level,
 preventing clipping and maintaining consistent output levels. The limiter processes
 each channel independently for envelope detection but applies gain reduction uniformly
 across all channels to preserve stereo imaging.

 # Examples

 ## Basic Usage with Default Settings

 ```rust
 use rodio::source::{SineWave, Source, LimitSettings};
 use std::time::Duration;

 // Create a loud sine wave and apply default limiting (-1dB threshold)
 let source = SineWave::new(440.0).amplify(2.0);
 let limited = source.limit(LimitSettings::default());
 ```

 ## Custom Settings with Builder Pattern

 ```rust
 use rodio::source::{SineWave, Source, LimitSettings};
 use std::time::Duration;

 let source = SineWave::new(440.0).amplify(3.0);
 let settings = LimitSettings::default()
     .with_threshold(-6.0)                    // Limit at -6dB
     .with_knee_width(2.0)                    // 2dB soft knee
     .with_attack(Duration::from_millis(3))   // Fast 3ms attack
     .with_release(Duration::from_millis(50)); // 50ms release

 let limited = source.limit(settings);
 ```
