use std::time::Duration;

use crate::common::Float;

/// Configuration settings for audio limiting.
///
/// This struct defines how the limiter behaves, including when to start limiting
/// (threshold), how gradually to apply it (knee width), and how quickly to respond
/// to level changes (attack/release times).
///
/// # Parameters
///
/// * **Threshold** - Level in dB where limiting begins (must be negative, typically -1 to -6 dB)
/// * **Knee Width** - Range in dB over which limiting gradually increases (wider = smoother)
/// * **Attack** - Time to respond to level increases (shorter = faster but may distort)
/// * **Release** - Time to recover after level decreases (longer = smoother)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use rodio::source::{SineWave, Source, LimitSettings};
/// use std::time::Duration;
///
/// // Use default settings (-1 dB threshold, 4 dB knee, 5ms attack, 100ms release)
/// let source = SineWave::new(440.0).amplify(2.0);
/// let limited = source.limit(LimitSettings::default());
/// ```
///
/// ## Custom Settings with Builder Pattern
///
/// ```rust
/// use rodio::source::{SineWave, Source, LimitSettings};
/// use std::time::Duration;
///
/// let source = SineWave::new(440.0).amplify(3.0);
/// let settings = LimitSettings::new()
///     .with_threshold(-6.0)                    // Limit peaks above -6dB
///     .with_knee_width(2.0)                    // 2dB soft knee for smooth limiting
///     .with_attack(Duration::from_millis(3))   // Fast 3ms attack
///     .with_release(Duration::from_millis(50)); // 50ms release
///
/// let limited = source.limit(settings);
/// ```
///
/// ## Common Adjustments
///
/// ```rust
/// use rodio::source::LimitSettings;
/// use std::time::Duration;
///
/// // More headroom for dynamic content
/// let conservative = LimitSettings::default()
///     .with_threshold(-3.0)                    // More headroom
///     .with_knee_width(6.0);                   // Wide knee for transparency
///
/// // Tighter control for broadcast/streaming
/// let broadcast = LimitSettings::default()
///     .with_knee_width(2.0)                     // Narrower knee for firmer limiting
///     .with_attack(Duration::from_millis(3))    // Faster attack
///     .with_release(Duration::from_millis(50)); // Faster release
/// ```
///
/// # dB vs. dBFS Reference
///
/// This limiter uses **dBFS (decibels relative to Full Scale)** for all level measurements:
/// - **0 dBFS** = maximum possible digital level (1.0 in linear scale)
/// - **Negative dBFS** = levels below maximum (e.g., -6 dBFS = 0.5 in linear scale)
/// - **Positive dBFS** = levels above maximum (causes digital clipping)
///
/// Unlike absolute dB measurements (dB SPL), dBFS is relative to the digital system's
/// maximum representable value, making it the standard for digital audio processing.
///
/// ## Common dBFS Reference Points
/// - **0 dBFS**: Digital maximum (clipping threshold)
/// - **-1 dBFS**: Just below clipping (tight limiting)
/// - **-3 dBFS**: Moderate headroom (balanced limiting)
/// - **-6 dBFS**: Generous headroom (gentle limiting)
/// - **-12 dBFS**: Conservative level (preserves significant dynamics)
/// - **-20 dBFS**: Very quiet level (background/ambient sounds)
#[derive(Debug, Clone)]
pub struct LimitSettings {
    /// Level where limiting begins (dBFS, must be negative).
    ///
    /// Specifies the threshold in dBFS where the limiter starts to reduce gain:
    /// - `-1.0` = limit at -1 dBFS (tight limiting, prevents clipping)
    /// - `-3.0` = limit at -3 dBFS (balanced approach with headroom)
    /// - `-6.0` = limit at -6 dBFS (gentle limiting, preserves dynamics)
    ///
    /// Values must be negative - positive values would attempt limiting above
    /// 0 dBFS, which cannot prevent clipping.
    pub threshold: Float,
    /// Range over which limiting gradually increases (dB).
    ///
    /// Defines the transition zone width in dB where limiting gradually increases
    /// from no effect to full limiting:
    /// - `0.0` = hard limiting (abrupt transition)
    /// - `2.0` = moderate knee (some gradual transition)
    /// - `4.0` = soft knee (smooth, transparent transition)
    /// - `8.0` = very soft knee (very gradual, musical transition)
    pub knee_width: Float,
    /// Time to respond to level increases
    pub attack: Duration,
    /// Time to recover after level decreases
    pub release: Duration,
}

impl Default for LimitSettings {
    fn default() -> Self {
        Self {
            threshold: -1.0,                     // -1 dB
            knee_width: 4.0,                     // 4 dB
            attack: Duration::from_millis(5),    // 5 ms
            release: Duration::from_millis(100), // 100 ms
        }
    }
}

impl LimitSettings {
    /// Creates new limit settings with default values.
    ///
    /// Equivalent to [`LimitSettings::default()`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates settings optimized for dynamic content like music and sound effects.
    ///
    /// Designed for content with varying dynamics where you want to preserve
    /// the natural feel while preventing occasional peaks from clipping.
    ///
    /// # Configuration
    ///
    /// - **Threshold**: -3.0 dBFS (more headroom than default)
    /// - **Knee width**: 6.0 dB (wide, transparent transition)
    /// - **Attack**: 5 ms (default, balanced response)
    /// - **Release**: 100 ms (default, smooth recovery)
    ///
    /// # Use Cases
    ///
    /// - Music playback with occasional loud peaks
    /// - Sound effects that need natural dynamics
    /// - Content where transparency is more important than tight control
    /// - Game audio with varying intensity levels
    ///
    /// # Examples
    ///
    /// ```
    /// use rodio::source::{SineWave, Source, LimitSettings};
    ///
    /// let music = SineWave::new(440.0).amplify(1.5);
    /// let limited = music.limit(LimitSettings::dynamic_content());
    /// ```
    #[inline]
    pub fn dynamic_content() -> Self {
        Self::default()
            .with_threshold(-3.0) // More headroom for dynamics
            .with_knee_width(6.0) // Wide knee for transparency
    }

    /// Creates settings optimized for broadcast and streaming applications.
    ///
    /// Designed for consistent loudness and reliable peak control in scenarios
    /// where clipping absolutely cannot occur and consistent levels are critical.
    ///
    /// # Configuration
    ///
    /// - **Threshold**: -1.0 dBFS (default, tight control)
    /// - **Knee width**: 2.0 dB (narrower, more decisive limiting)
    /// - **Attack**: 3 ms (faster response to catch transients)
    /// - **Release**: 50 ms (faster recovery for consistent levels)
    ///
    /// # Use Cases
    ///
    /// - Live streaming where clipping would be catastrophic
    /// - Broadcast audio that must meet loudness standards
    /// - Voice chat applications requiring consistent levels
    /// - Podcast production for consistent listening experience
    /// - Game voice communication systems
    ///
    /// # Examples
    ///
    /// ```
    /// use rodio::source::{SineWave, Source, LimitSettings};
    ///
    /// let voice_chat = SineWave::new(440.0).amplify(2.0);
    /// let limited = voice_chat.limit(LimitSettings::broadcast());
    /// ```
    #[inline]
    pub fn broadcast() -> Self {
        Self::default()
            .with_knee_width(2.0) // Narrower knee for decisive limiting
            .with_attack(Duration::from_millis(3)) // Faster attack for transients
            .with_release(Duration::from_millis(50)) // Faster recovery for consistency
    }

    /// Creates settings optimized for mastering and final audio production.
    ///
    /// Designed for the final stage of audio production where tight peak control
    /// is needed while maintaining audio quality and preventing any clipping.
    ///
    /// # Configuration
    ///
    /// - **Threshold**: -0.5 dBFS (very tight, maximum loudness)
    /// - **Knee width**: 1.0 dB (narrow, precise control)
    /// - **Attack**: 1 ms (very fast, catches all transients)
    /// - **Release**: 200 ms (slower, maintains natural envelope)
    ///
    /// # Use Cases
    ///
    /// - Final mastering stage for tight peak control
    /// - Preparing audio for streaming platforms (after loudness processing)
    /// - Album mastering where consistent peak levels are critical
    /// - Audio post-production for film/video
    ///
    /// # Examples
    ///
    /// ```
    /// use rodio::source::{SineWave, Source, LimitSettings};
    ///
    /// let master_track = SineWave::new(440.0).amplify(3.0);
    /// let mastered = master_track.limit(LimitSettings::mastering());
    /// ```
    #[inline]
    pub fn mastering() -> Self {
        Self {
            threshold: -0.5,                     // Very tight for peak control
            knee_width: 1.0,                     // Narrow knee for precise control
            attack: Duration::from_millis(1),    // Very fast attack
            release: Duration::from_millis(200), // Slower release for natural envelope
        }
    }

    /// Creates settings optimized for live performance and real-time applications.
    ///
    /// Designed for scenarios where low latency is critical and the limiter
    /// must respond quickly to protect equipment and audiences.
    ///
    /// # Configuration
    ///
    /// - **Threshold**: -2.0 dBFS (some headroom for safety)
    /// - **Knee width**: 3.0 dB (moderate, good compromise)
    /// - **Attack**: 0.5 ms (extremely fast for protection)
    /// - **Release**: 30 ms (fast recovery for live feel)
    ///
    /// # Use Cases
    ///
    /// - Live concert sound reinforcement
    /// - DJ mixing and live electronic music
    /// - Real-time audio processing where latency matters
    /// - Equipment protection in live settings
    /// - Interactive audio applications and games
    ///
    /// # Examples
    ///
    /// ```
    /// use rodio::source::{SineWave, Source, LimitSettings};
    ///
    /// let live_input = SineWave::new(440.0).amplify(2.5);
    /// let protected = live_input.limit(LimitSettings::live_performance());
    /// ```
    #[inline]
    pub fn live_performance() -> Self {
        Self {
            threshold: -2.0,                    // Some headroom for safety
            knee_width: 3.0,                    // Moderate knee
            attack: Duration::from_micros(500), // Extremely fast for protection
            release: Duration::from_millis(30), // Fast recovery for live feel
        }
    }

    /// Creates settings optimized for gaming and interactive audio.
    ///
    /// Designed for games where audio levels can vary dramatically between
    /// quiet ambient sounds and loud action sequences, requiring responsive
    /// limiting that maintains immersion.
    ///
    /// # Configuration
    ///
    /// - **Threshold**: -3.0 dBFS (balanced headroom for dynamic range)
    /// - **Knee width**: 3.0 dB (moderate transition for natural feel)
    /// - **Attack**: 2 ms (fast enough for sound effects, not harsh)
    /// - **Release**: 75 ms (quick recovery for interactive responsiveness)
    ///
    /// # Use Cases
    ///
    /// - Game audio mixing for consistent player experience
    /// - Interactive audio applications requiring dynamic response
    /// - VR/AR audio where sudden loud sounds could be jarring
    /// - Mobile games needing battery-efficient processing
    /// - Streaming gameplay audio for viewers
    ///
    /// # Examples
    ///
    /// ```
    /// use rodio::source::{SineWave, Source, LimitSettings};
    ///
    /// let game_audio = SineWave::new(440.0).amplify(2.0);
    /// let limited = game_audio.limit(LimitSettings::gaming());
    /// ```
    #[inline]
    pub fn gaming() -> Self {
        Self {
            threshold: -3.0,                    // Balanced headroom for dynamics
            knee_width: 3.0,                    // Moderate for natural feel
            attack: Duration::from_millis(2),   // Fast but not harsh
            release: Duration::from_millis(75), // Quick for interactivity
        }
    }

    /// Sets the threshold level where limiting begins.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Level in dBFS where limiting starts (must be negative)
    ///   - `-1.0` = limiting starts at -1 dBFS (tight limiting, prevents clipping)
    ///   - `-3.0` = limiting starts at -3 dBFS (balanced approach with headroom)
    ///   - `-6.0` = limiting starts at -6 dBFS (gentle limiting, preserves dynamics)
    ///   - `-12.0` = limiting starts at -12 dBFS (very aggressive, significantly reduces dynamics)
    ///
    /// # dBFS Context
    ///
    /// Remember that 0 dBFS is the digital maximum. Negative dBFS values represent
    /// levels below this maximum:
    /// - `-1 dBFS` ≈ 89% of maximum amplitude (very loud, limiting triggers late)
    /// - `-3 dBFS` ≈ 71% of maximum amplitude (loud, moderate limiting)
    /// - `-6 dBFS` ≈ 50% of maximum amplitude (moderate, gentle limiting)
    /// - `-12 dBFS` ≈ 25% of maximum amplitude (quiet, aggressive limiting)
    ///
    /// Lower thresholds (more negative) trigger limiting earlier and reduce dynamics more.
    /// Only negative values are meaningful - positive values would attempt limiting
    /// above 0 dBFS, which cannot prevent clipping.
    #[inline]
    pub fn with_threshold(mut self, threshold: Float) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the knee width - range over which limiting gradually increases.
    ///
    /// # Arguments
    ///
    /// * `knee_width` - Range in dB over which limiting transitions from off to full effect
    ///   - `0.0` dB = hard knee (abrupt limiting, may sound harsh)
    ///   - `1.0-2.0` dB = moderate knee (noticeable but controlled limiting)
    ///   - `4.0` dB = soft knee (smooth, transparent limiting) \[default\]
    ///   - `6.0-8.0` dB = very soft knee (very gradual, musical limiting)
    ///
    /// # How Knee Width Works
    ///
    /// The knee creates a transition zone around the threshold. For example, with
    /// `threshold = -3.0` dBFS and `knee_width = 4.0` dB:
    /// - No limiting below -5 dBFS (threshold - knee_width/2)
    /// - Gradual limiting from -5 dBFS to -1 dBFS
    /// - Full limiting above -1 dBFS (threshold + knee_width/2)
    #[inline]
    pub fn with_knee_width(mut self, knee_width: Float) -> Self {
        self.knee_width = knee_width;
        self
    }

    /// Sets the attack time - how quickly the limiter responds to level increases.
    ///
    /// # Arguments
    ///
    /// * `attack` - Time duration for the limiter to react to peaks
    ///   - Shorter (1-5 ms) = faster response, may cause distortion
    ///   - Longer (10-20 ms) = smoother sound, may allow brief overshoots
    #[inline]
    pub fn with_attack(mut self, attack: Duration) -> Self {
        self.attack = attack;
        self
    }

    /// Sets the release time - how quickly the limiter recovers after level decreases.
    ///
    /// # Arguments
    ///
    /// * `release` - Time duration for the limiter to stop limiting
    ///   - Shorter (10-50 ms) = quick recovery, may sound pumping
    ///   - Longer (100-500 ms) = smooth recovery, more natural sound
    #[inline]
    pub fn with_release(mut self, release: Duration) -> Self {
        self.release = release;
        self
    }
}
