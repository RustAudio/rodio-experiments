Applies automatic gain control to the sound.

Automatic Gain Control (AGC) adjusts the amplitude of the audio signal
to maintain a consistent output level.

# Parameters

`target_level`:
  **TL;DR**: Desired output level. 1.0 = original level, > 1.0 amplifies, < 1.0 reduces.

  The desired output level, where 1.0 represents the original sound level.
  Values above 1.0 will amplify the sound, while values below 1.0 will lower it.
  For example, a target_level of 1.4 means that at normal sound levels, the AGC
  will aim to increase the gain by a factor of 1.4, resulting in a minimum 40% amplification.
  A recommended level is `1.0`, which maintains the original sound level.

`attack_time`:
  **TL;DR**: Response time for volume increases. Shorter = faster but may cause abrupt changes. **Recommended: `4.0` seconds**.

  The time (in seconds) for the AGC to respond to input level increases.
  Shorter times mean faster response but may cause abrupt changes. Longer times result
  in smoother transitions but slower reactions to sudden volume changes. Too short can
  lead to overreaction to peaks, causing unnecessary adjustments. Too long can make the
  AGC miss important volume changes or react too slowly to sudden loud passages. Very
  high values might result in excessively loud output or sluggish response, as the AGC's
  adjustment speed is limited by the attack time. Balance is key for optimal performance.
  A recommended attack_time of `4.0` seconds provides a sweet spot for most applications.

`release_time`:
  **TL;DR**: Response time for volume decreases. Shorter = faster gain reduction. **Recommended: `0.0` seconds**.

  The time (in seconds) for the AGC to respond to input level decreases.
  This parameter controls how quickly the gain is reduced when the signal level drops.
  Shorter release times result in faster gain reduction, which can be useful for quick
  adaptation to quieter passages but may lead to pumping effects. Longer release times
  provide smoother transitions but may be slower to respond to sudden decreases in volume.
  However, if the release_time is too high, the AGC may not be able to lower the gain
  quickly enough, potentially leading to clipping and distorted sound before it can adjust.
  Finding the right balance is crucial for maintaining natural-sounding dynamics and
  preventing distortion. A recommended release_time of `0.0` seconds works well for
  general use, allowing the AGC to decrease the gain immediately with no delay, ensuring there is no clipping.

`absolute_max_gain`:
  **TL;DR**: Maximum allowed gain. Prevents over-amplification. **Recommended: `5.0`**.

  The maximum gain that can be applied to the signal.
  This parameter acts as a safeguard against excessive amplification of quiet signals
  or background noise. It establishes an upper boundary for the AGC's signal boost,
  effectively preventing distortion or over-amplification of low-level sounds.
  This is crucial for maintaining audio quality and preventing unexpected volume spikes.
  A recommended value for `absolute_max_gain` is `5`, which provides a good balance between
  amplification capability and protection against distortion in most scenarios.

`automatic_gain_control` example in this project shows a pattern you can use
to enable/disable the AGC filter dynamically.

# Example (Quick start)

```rust
// Apply Automatic Gain Control to the source (AGC is on by default)
use rodio::source::{Source, SineWave, AutomaticGainControlSettings};
use rodio::Player;
use std::time::Duration;
let source = SineWave::new(444.0); // An example.
let (player, output) = Player::new(); // An example.

let agc_source = source.automatic_gain_control(AutomaticGainControlSettings::default());

// Add the AGC-controlled source to the sink
player.append(agc_source);

