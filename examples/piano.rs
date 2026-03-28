// use rodio::FixedSource;
// use rodio_experiments::effects::amplify::Factor;
// use rodio_experiments::fixed_source::FixedSourceExt;
// use rodio_experiments::fixed_source::mixed::IntoMixed;
// use rodio_experiments::generators::fixed_source::function::SineWave;
// use rodio_experiments::speakers::SpeakersBuilder;
// use std::error::Error;
// use std::thread;
// use std::time::Duration;
//
// struct Point {
//     x: f32,
//     y: f32,
// }
//
// struct CubicBezier {
//     p0: Point,
//     p1: Point,
//     p2: Point,
//     p3: Point,
// }
//
// impl CubicBezier {
//     fn new(controls: BezierControls, gain: std::ops::RangeInclusive<f32>) -> Self {
//         Self {
//             p0: Point {
//                 x: 0.,
//                 y: *gain.start(),
//             },
//             p1: controls.control1,
//             p2: controls.control2,
//             p3: Point {
//                 x: 1.,
//                 y: *gain.end(),
//             },
//         }
//     }
//
//     // https://stackoverflow.com/questions/66774792/given-a-cubic-bezier-curve-with-fixed-endpoints-how-can-i-find-the-x-position-o
//     fn t(&self, x: f32) -> f32 {
//         use roots::{Roots, find_roots_cubic};
//         let Self { p0, p1, p2, p3 } = self;
//
//         let a3 = p3.x - 3.0 * p2.x + 3.0 * p1.x - p0.x;
//         let a2 = 3.0 * p2.x - 6.0 * p1.x + 3.0 * p0.x;
//         let a1 = 3.0 * p1.x - 3.0 * p0.x;
//         let a0 = p0.x - x;
//
//         match find_roots_cubic::<f32>(a3, a2, a1, a0) {
//             Roots::No(_) => panic!("Curve should be in range 0-1"),
//             Roots::One([t]) => t,
//             Roots::Two([t1, _]) => t1,
//             Roots::Three([t1, ..]) => t1,
//             Roots::Four([t1, ..]) => t1,
//         }
//     }
//
//     fn y(&self, t: f32) -> f32 {
//         let Self { p0, p1, p2, p3 } = self;
//
//         p0.y * (1.0 - t).powi(3)
//             + 3.0 * p1.y * (1.0 - t).powi(2) * t
//             + 3.0 * p2.y * (1.0 - t) * t.powi(2)
//             + p3.y * t.powi(3)
//     }
// }
//
// struct EnvelopeBuilder {
//     curve: CubicBezier,
//     duration: Duration,
//     start_after: Duration,
// }
//
// struct BezierControls {
//     control1: Point,
//     control2: Point,
// }
//
// const ATTACK: BezierControls = BezierControls {
//     control1: Point { x: 0.0, y: 0.77 },
//     control2: Point { x: 0.32, y: 1.0 },
// };
//
// // nice editor for cubic bezier curves
// // https://www.desmos.com/calculator/iogphoixw4
// const DECAY: BezierControls = BezierControls {
//     control1: Point { x: 0.0, y: 0.0 },
//     control2: Point { x: 1.0, y: 0.0 },
// };
//
// const RELEASE: BezierControls = BezierControls {
//     control1: Point { x: 0.0, y: 1.0 },
//     control2: Point { x: 1.0, y: 0.0 },
// };
//
// trait IntoCurve {
//     fn into_curve(self) -> CubicBezier;
// }
//
// impl IntoCurve for (BezierControls, std::ops::RangeInclusive<f32>) {
//     fn into_curve(self) -> CubicBezier {
//         CubicBezier {
//             p0: Point {
//                 x: 0.0,
//                 y: *self.1.start(),
//             },
//             p1: self.0.control1,
//             p2: self.0.control2,
//             p3: Point {
//                 x: 1.0,
//                 y: *self.1.end(),
//             },
//         }
//     }
// }
//
// impl EnvelopeBuilder {
//     fn new() -> Self {
//         const LINEAR_DOWN: BezierControls = BezierControls {
//             control1: Point { x: 0.0, y: 1.0 },
//             control2: Point { x: 1.0, y: 0.0 },
//         };
//
//         Self {
//             curve: CubicBezier::new(LINEAR_DOWN, 1.0..=0.0),
//             duration: Duration::from_secs(1),
//             start_after: Duration::ZERO,
//         }
//     }
//
//     fn with_curve(mut self, curve: impl IntoCurve) -> Self {
//         self.curve = curve.into_curve();
//         self
//     }
//
//     fn takes(mut self, duration: Duration) -> Self {
//         self.duration = duration;
//         self
//     }
//
//     fn start_after(mut self, duration: Duration) -> Self {
//         self.start_after = duration;
//         self
//     }
// }
//
// struct Envelope {
//     curve: CubicBezier,
//     x: f32,
//     skip: usize,
//     step: usize,
//     steps: usize,
// }
//
// impl Iterator for Envelope {
//     type Item = f32;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if !(self.skip..self.skip + self.steps).contains(&self.step) {
//             return None;
//         }
//
//         let x = self.step as f32 / self.steps as f32;
//         self.steps += 1;
//         let t = self.curve.t(x);
//         Some(self.curve.y(t))
//     }
// }
//
// trait IntoAmplifyFactor {
//     type Factor: AmplifyFactor;
//     fn into(self, source: &impl FixedSource) -> Self::Factor;
// }
//
// // TODO what is a fadeout but a changing amplify?
// trait AmplifyFactor {
//     fn linear_gain(&mut self) -> f32;
//     fn seek(&mut self, position: usize);
// }
//
// impl IntoAmplifyFactor for Factor {
//     type Factor = Self;
//
//     fn into(self, _: &impl FixedSource) -> Self::Factor {
//         self
//     }
// }
//
// impl AmplifyFactor for Factor {
//     fn linear_gain(&mut self) -> f32 {
//         self.as_linear()
//     }
//     fn seek(&mut self, _: usize) {}
// }
//
// impl IntoAmplifyFactor for EnvelopeBuilder {
//     type Factor = Envelope;
//
//     fn into(self, source: &impl FixedSource) -> Self::Factor {
//         let skip = source.channels().get() as f32
//             * source.sample_rate().get() as f32
//             * self.start_after.as_secs_f32();
//         let steps = source.channels().get() as f32
//             * source.sample_rate().get() as f32
//             * self.duration.as_secs_f32();
//
//         Self::Factor {
//             curve: self.curve,
//             x: 0.0,
//             skip: skip as usize,
//             step: 0,
//             steps: steps as usize,
//         }
//     }
// }
//
// impl AmplifyFactor for Envelope {
//     fn linear_gain(&mut self) -> f32 {
//         self.next().unwrap_or(1.0)
//     }
//     fn seek(&mut self, _: usize) {
//         todo!()
//     }
// }
//
// struct Amplify2<I: FixedSource, F: AmplifyFactor> {
//     inner: I,
//     factor: F,
// }
//
// impl<I: FixedSource, F: AmplifyFactor> FixedSource for Amplify2<I, F> {
//     fn channels(&self) -> rodio::ChannelCount {
//         self.inner.channels()
//     }
//
//     fn sample_rate(&self) -> rodio::SampleRate {
//         self.inner.sample_rate()
//     }
//
//     fn total_duration(&self) -> Option<Duration> {
//         self.inner.total_duration()
//     }
// }
//
// impl<I: FixedSource, F: AmplifyFactor> Iterator for Amplify2<I, F> {
//     type Item = rodio::Sample;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let sample = self.inner.next()?;
//         Some(sample * dbg!(self.factor.linear_gain()))
//     }
// }
//
// trait FixedSourceExt2 {
//     fn amplify2<F: IntoAmplifyFactor>(self, factor: F) -> Amplify2<Self, F::Factor>
//     where
//         Self: FixedSource + Sized;
// }
//
// impl<S: FixedSource> FixedSourceExt2 for S {
//     fn amplify2<F: IntoAmplifyFactor>(self, factor: F) -> Amplify2<Self, F::Factor>
//     where
//         Self: Sized,
//     {
//         Amplify2 {
//             factor: factor.into(&self),
//             inner: self,
//         }
//     }
// }
//
// fn main() -> Result<(), Box<dyn Error>> {
//     let speakers = SpeakersBuilder::new().default_device()?.default_config()?;
//     let config = speakers.get_config();
//
//     let note = (1..8)
//         .into_iter()
//         .map(|i| {
//             SineWave::new(440.0 * (i as f32), config.sample_rate)
//                 .amplify(Factor::Linear(1.0 / 2.5f32.powf(i as f32)))
//         })
//         .collect::<Vec<_>>()
//         .try_into_mixed()
//         .expect("parameters all identical")
//         .amplify2(
//             EnvelopeBuilder::new()
//                 .with_curve((ATTACK, 0.0..=1.0))
//                 .takes(Duration::from_millis(400)),
//         )
//         // .amplify2(EnvelopeBuilder::test(
//         //     Duration::from_millis(400),
//         //     Duration::from_millis(1500),
//         // ))
//         // .amplify2(EnvelopeBuilder::test(
//         //     Duration::from_millis(1500),
//         //     Duration::from_millis(200),
//         // ))
//         .take_duration(Duration::from_millis(4))
//         .with_channel_count(config.channel_count);
//
//     let _speakers = speakers.play(note)?;
//
//     thread::sleep(Duration::from_secs(2));
//     Ok(())
// }

fn main() {}
