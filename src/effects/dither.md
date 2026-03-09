Applies dithering to the source at the specified bit depth.

Dithering eliminates quantization artifacts during digital audio playback
and when converting between bit depths. Apply at the target output bit depth.

# Example

```rust
use rodio::source::{SineWave, Source, DitherAlgorithm};
use rodio::BitDepth;

let source = SineWave::new(440.0)
    .amplify(0.5)
    .dither(BitDepth::new(16).unwrap(), DitherAlgorithm::default());
```
