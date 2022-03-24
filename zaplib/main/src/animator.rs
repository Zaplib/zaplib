//! An animation system for transitioning between various kinds of values over time.

use crate::*;
use std::f64::consts::PI;

/// Manages animations. Assumes that you always pass it with [`Anim`] objects with
/// the same types of [`Track`]s. For example, if track 0 represents represents a
/// color Vec4, and track 1 represents a Float value, then make sure to use that
/// consistently throughout the lifetime of an Animator.
///
/// TODO(JP): Look into enforcing this through the type system, instead of at
/// runtime.
///
/// Also note that the [`Animator`] always contains the "source of truth" for the
/// values it manages, so whenever necessary you should copy the values kept
/// here to the actual "draw objects".
#[derive(Debug, Default)]
pub struct Animator {
    /// The current source of truth of values that this [`Animator`] manages.
    /// Initially undefined, until [`Animator::draw`] initializes the Animator.
    ///
    /// Call [`Animator::draw`] or [`Animator::handle`] to update these
    /// values based on the [`Animator::current`] animation and the current time.
    values: Option<Vec<AnimValue>>,

    /// The current [`Anim`] that is being played. Is [`None`] when there is no
    /// active animation.
    current: Option<Anim>,

    /// The [`Anim`] that will be played next when [`Animator::current`] is done playing. It
    /// should never be possible that [`Animator::next`] is set but [`Animator::current`] isn't. There
    /// can only be one animation queued up.
    next: Option<Anim>,

    /// The time that the [`Animator::current`] animation started playing.
    current_start_time: f64,

    /// The last timestamp we updated our animation, used to avoid computing the
    /// animation values multiple times for the same timestamp.
    last_processed_time: f64,
}

impl Animator {
    /// Play an animation. If an animation is already playing, it's either cut
    /// off, or remains playing with the new animation queued up, if [`Anim::chain`]
    /// is set in the new animation. If there was already another animation
    /// queued up, then it's kicked from the queue.
    pub fn play_anim(&mut self, cx: &mut Cx, anim: Anim) {
        if self.current.is_none() || !anim.chain {
            // If there is no current animation or if we're not chaining, just
            // overwrite the current animation.
            self.current = Some(anim);
            self.next = None;
            self.current_start_time = cx.last_event_time;
            // Make sure that we request a new frame to play our animation in.
            cx.request_next_frame();
        } else {
            // Otherwise, queue it, kicking out any previous animation in the
            // queue.
            self.next = Some(anim);
        }
    }

    /// Process animations from a "draw" function. This must be called before reading any values.
    ///
    /// The `anim_default` will initialze the Animator if it's currently uninitialized.
    pub fn draw(&mut self, cx: &mut Cx, anim_default: Anim) {
        if self.values.is_none() {
            self.values = Some(anim_default.get_last_values());
        }
        self.run_animator(cx);
    }

    /// Convenient function for only calling [`Animator::run_animator`] if the event is
    /// an [`Event::NextFrame`]. Returns true if we processed the animation so you
    /// can update your "draw objects".
    pub fn handle(&mut self, cx: &mut Cx, event: &Event) -> bool {
        match event {
            Event::NextFrame => self.values.is_some() && self.run_animator(cx),
            _ => false,
        }
    }

    /// Process any playing animations based on the current time from [`Cx`].
    /// Returns whether or not [`Animator::values`] have been updated, so you can update
    /// your "draw objects" accordingly. Note that [`Animator::values`] are the "source of
    /// truth", so when in doubt it's always safe to just update your objects
    /// based on [`Animator::values`] regardless of the return value of [`Animator::draw`].
    fn run_animator(&mut self, cx: &mut Cx) -> bool {
        // Skip if time hasn't changed, unless this is the initial call.
        if cx.last_event_time == self.last_processed_time {
            return false;
        }
        self.last_processed_time = cx.last_event_time;

        // First check if the current animation has expired, in which case we need to either stop
        // animating, or start with the queued up animation.
        if let Some(current_anim) = &self.current {
            if self.current_start_time + current_anim.duration <= cx.last_event_time {
                // Make sure that `values` actually reflects the "end state" of the animation, since
                // at the previous rendering step we were probably a little bit before the actual
                // end.
                self.values = Some(current_anim.get_last_values());

                if self.next.is_none() {
                    // If there was no animation queued up, just bail out, but still return `true`
                    // since we've changed `values`.
                    self.current = None;
                    return true;
                } else {
                    // Don't just set the new current_start_time to the current time, since most
                    // likely we have overshot a little and are actually a tiny bit into the next
                    // animation already.
                    self.current_start_time += current_anim.duration;
                    std::mem::swap(&mut self.current, &mut self.next);
                    self.next = None;
                    // Fall through, so we compute the current values based on the animation that
                    // was queued up (and which is now `current`).
                }
            }
        }

        // If we still have an active animation, compute `values`.
        if let Some(current_anim) = &self.current {
            // First, make sure that we will get a next frame for our animation.
            cx.request_next_frame();

            // Compute the fraction between 0 and 1 of how far we are into the current animation.
            let time_fraction = (cx.last_event_time - self.current_start_time) / current_anim.duration;

            let values = self.values.as_mut().unwrap();

            // Update all the individual values based on how far we are into the current animation.
            for (index, track) in current_anim.tracks.iter().enumerate() {
                match track {
                    Track::Float { key_frames, ease } => {
                        values[index] = AnimValue::Float(Track::compute_track_float(
                            time_fraction,
                            key_frames,
                            values[index].unwrap_float(),
                            ease,
                        ));
                    }
                    Track::Vec2 { key_frames, ease } => {
                        values[index] = AnimValue::Vec2(Track::compute_track_vec2(
                            time_fraction,
                            key_frames,
                            values[index].unwrap_vec2(),
                            ease,
                        ));
                    }
                    Track::Vec3 { key_frames, ease } => {
                        values[index] = AnimValue::Vec3(Track::compute_track_vec3(
                            time_fraction,
                            key_frames,
                            values[index].unwrap_vec3(),
                            ease,
                        ));
                    }
                    Track::Vec4 { key_frames, ease } => {
                        values[index] = AnimValue::Vec4(Track::compute_track_vec4(
                            time_fraction,
                            key_frames,
                            values[index].unwrap_vec4(),
                            ease,
                        ));
                    }
                }
            }
            return true;
        }
        false
    }

    /// Get the value of the given track as a float. Be sure to call this only if the given track is
    /// indeed always a float in the [`Anim`]s you pass into this [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    pub fn get_float(&self, track_index: usize) -> f32 {
        self.values.as_ref().unwrap()[track_index].unwrap_float()
    }

    /// Get the value of the given track as a [`Vec2`]. Be sure to call this only if the given track is
    /// indeed always a [`Vec2`] in the [`Anim`]s you pass into this [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    pub fn get_vec2(&self, track_index: usize) -> Vec2 {
        self.values.as_ref().unwrap()[track_index].unwrap_vec2()
    }

    /// Get the value of the given track as a [`Vec3`]. Be sure to call this only if the given track is
    /// indeed always a [`Vec3`] in the [`Anim`]s you pass into this [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    pub fn get_vec3(&self, track_index: usize) -> Vec3 {
        self.values.as_ref().unwrap()[track_index].unwrap_vec3()
    }

    /// Get the value of the given track as a [`Vec4`]. Be sure to call this only if the given track is
    /// indeed always a [`Vec4`] in the [`Anim`]s you pass into this [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    pub fn get_vec4(&self, track_index: usize) -> Vec4 {
        self.values.as_ref().unwrap()[track_index].unwrap_vec4()
    }
}

/// Represents an actual value in an [`Animator`], which can be of a few
/// different types, but should remain consistent in its type (for a
/// given [`Track`]) for the lifetime of an [`Animator`].
#[derive(Clone, Debug)]
pub enum AnimValue {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}

impl AnimValue {
    /// Get the value as a float. Be sure to call this only if the given track is
    /// indeed always a float in the [`Anim`]s you pass into the [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    fn unwrap_float(&self) -> f32 {
        match self {
            AnimValue::Float(cur_val) => *cur_val,
            _ => panic!("Unexpected AnimValue type"),
        }
    }

    /// Get the value as a [`Vec2`]. Be sure to call this only if the given track is
    /// indeed always a [`Vec2`] in the [`Anim`]s you pass into the [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    fn unwrap_vec2(&self) -> Vec2 {
        match self {
            AnimValue::Vec2(cur_val) => *cur_val,
            _ => panic!("Unexpected AnimValue type"),
        }
    }

    /// Get the value as a [`Vec3`]. Be sure to call this only if the given track is
    /// indeed always a [`Vec3`] in the [`Anim`]s you pass into the [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    fn unwrap_vec3(&self) -> Vec3 {
        match self {
            AnimValue::Vec3(cur_val) => *cur_val,
            _ => panic!("Unexpected AnimValue type"),
        }
    }

    /// Get the value as a [`Vec4`]. Be sure to call this only if the given track is
    /// indeed always a [`Vec4`] in the [`Anim`]s you pass into the [`Animator`].
    /// TODO(JP): Instead of having multiple functions here, perhaps we can use [`Into`?
    fn unwrap_vec4(&self) -> Vec4 {
        match self {
            AnimValue::Vec4(cur_val) => *cur_val,
            _ => panic!("Unexpected AnimValue type"),
        }
    }
}

/// An actual animation that can be played.
#[derive(Clone, Debug, PartialEq)]
pub struct Anim {
    /// The time it should take for this animation to complete, in seconds.
    pub duration: f64,

    /// If set, this animation will get queued up if there is an existing
    /// animation playing.
    pub chain: bool,

    /// The actual tracks of values that will change during this animation.
    /// Should remain consistent between the different animations that you pass
    /// into a single [`Animator`].
    ///
    /// TODO(JP): Allow for dynamically defined animations:
    /// https://github.com/Zaplib/zaplib/issues/167
    pub tracks: &'static [Track],
}

impl Anim {
    /// TODO(JP): Replace these with Anim::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Anim = Anim { duration: 0., chain: false, tracks: &[] };

    /// Get the values for the "end state" of an animation, ie. the values for
    /// when the animation is done.
    fn get_last_values(&self) -> Vec<AnimValue> {
        self.tracks
            .iter()
            .map(|track| match track {
                Track::Vec4 { key_frames, .. } => AnimValue::Vec4(key_frames.last().unwrap().1),
                Track::Vec3 { key_frames, .. } => AnimValue::Vec3(key_frames.last().unwrap().1),
                Track::Vec2 { key_frames, .. } => AnimValue::Vec2(key_frames.last().unwrap().1),
                Track::Float { key_frames, .. } => AnimValue::Float(key_frames.last().unwrap().1),
            })
            .collect()
    }
}
impl Default for Anim {
    fn default() -> Self {
        Anim::DEFAULT
    }
}

/// Describes how output values of a [`Track`] get mapped for fractions in between
/// keyframes. See these pages for more explanations:
/// * <https://developer.mozilla.org/en-US/docs/Web/CSS/easing-function>
/// * <https://easings.net>
#[derive(Clone, Debug, PartialEq)]
pub enum Ease {
    Lin,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InSine,
    OutSine,
    InOutSine,
    InExp,
    OutExp,
    InOutExp,
    InCirc,
    OutCirc,
    InOutCirc,
    InElastic,
    OutElastic,
    InOutElastic,
    InBack,
    OutBack,
    InOutBack,
    InBounce,
    OutBounce,
    InOutBounce,
    Pow { begin: f64, end: f64 },
    Bezier { cp0: f64, cp1: f64, cp2: f64, cp3: f64 },
}
impl Ease {
    /// TODO(JP): Replace these with Ease::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Ease = Ease::InOutCubic;
}
impl Default for Ease {
    fn default() -> Self {
        Ease::DEFAULT
    }
}

impl Ease {
    // Clippy TODO
    #[warn(clippy::many_single_char_names)]
    pub fn map(&self, t: f64) -> f64 {
        match self {
            Ease::Lin => t.max(0.0).min(1.0),
            Ease::Pow { begin, end } => {
                if t < 0. {
                    return 0.;
                }
                if t > 1. {
                    return 1.;
                }
                let a = -1. / (begin * begin).max(1.0);
                let b = 1. + 1. / (end * end).max(1.0);
                let t2 = (((a - 1.) * -b) / (a * (1. - b))).powf(t);
                (-a * b + b * a * t2) / (a * t2 - b)
            }

            Ease::InQuad => t * t,
            Ease::OutQuad => t * (2.0 - t),
            Ease::InOutQuad => {
                let t = t * 2.0;
                if t < 1. {
                    0.5 * t * t
                } else {
                    let t = t - 1.;
                    -0.5 * (t * (t - 2.) - 1.)
                }
            }
            Ease::InCubic => t * t * t,
            Ease::OutCubic => {
                let t2 = t - 1.0;
                t2 * t2 * t2 + 1.0
            }
            Ease::InOutCubic => {
                let t = t * 2.0;
                if t < 1. {
                    0.5 * t * t * t
                } else {
                    let t = t - 2.;
                    1. / 2. * (t * t * t + 2.)
                }
            }
            Ease::InQuart => t * t * t * t,
            Ease::OutQuart => {
                let t = t - 1.;
                -(t * t * t * t - 1.)
            }
            Ease::InOutQuart => {
                let t = t * 2.0;
                if t < 1. {
                    0.5 * t * t * t * t
                } else {
                    let t = t - 2.;
                    -0.5 * (t * t * t * t - 2.)
                }
            }
            Ease::InQuint => t * t * t * t * t,
            Ease::OutQuint => {
                let t = t - 1.;
                t * t * t * t * t + 1.
            }
            Ease::InOutQuint => {
                let t = t * 2.0;
                if t < 1. {
                    0.5 * t * t * t * t * t
                } else {
                    let t = t - 2.;
                    0.5 * (t * t * t * t * t + 2.)
                }
            }
            Ease::InSine => -(t * PI * 0.5).cos() + 1.,
            Ease::OutSine => (t * PI * 0.5).sin(),
            Ease::InOutSine => -0.5 * ((t * PI).cos() - 1.),
            Ease::InExp => {
                if t < 0.001 {
                    0.
                } else {
                    2.0f64.powf(10. * (t - 1.))
                }
            }
            Ease::OutExp => {
                if t > 0.999 {
                    1.
                } else {
                    -(2.0f64.powf(-10. * t)) + 1.
                }
            }
            Ease::InOutExp => {
                if t < 0.001 {
                    return 0.;
                }
                if t > 0.999 {
                    return 1.;
                }
                let t = t * 2.0;
                if t < 1. {
                    0.5 * 2.0f64.powf(10. * (t - 1.))
                } else {
                    let t = t - 1.;
                    0.5 * (-(2.0f64.powf(-10. * t)) + 2.)
                }
            }
            Ease::InCirc => -((1. - t * t).sqrt() - 1.),
            Ease::OutCirc => {
                let t = t - 1.;
                (1. - t * t).sqrt()
            }
            Ease::InOutCirc => {
                let t = t * 2.;
                if t < 1. {
                    -0.5 * ((1. - t * t).sqrt() - 1.)
                } else {
                    let t = t - 2.;
                    0.5 * ((1. - t * t).sqrt() + 1.)
                }
            }
            Ease::InElastic => {
                let p = 0.3;
                let s = p / 4.0; // c = 1.0, b = 0.0, d = 1.0
                if t < 0.001 {
                    return 0.;
                }
                if t > 0.999 {
                    return 1.;
                }
                let t = t - 1.0;
                -(2.0f64.powf(10.0 * t) * ((t - s) * (2.0 * PI) / p).sin())
            }
            Ease::OutElastic => {
                let p = 0.3;
                let s = p / 4.0; // c = 1.0, b = 0.0, d = 1.0

                if t < 0.001 {
                    return 0.;
                }
                if t > 0.999 {
                    return 1.;
                }
                2.0f64.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
            }
            Ease::InOutElastic => {
                let p = 0.3;
                let s = p / 4.0; // c = 1.0, b = 0.0, d = 1.0
                if t < 0.001 {
                    return 0.;
                }
                if t > 0.999 {
                    return 1.;
                }
                let t = t * 2.0;
                if t < 1. {
                    let t = t - 1.0;
                    -0.5 * (2.0f64.powf(10.0 * t) * ((t - s) * (2.0 * PI) / p).sin())
                } else {
                    let t = t - 1.0;
                    0.5 * 2.0f64.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
                }
            }
            Ease::InBack => {
                let s = 1.70158;
                t * t * ((s + 1.) * t - s)
            }
            Ease::OutBack => {
                let s = 1.70158;
                let t = t - 1.;
                t * t * ((s + 1.) * t + s) + 1.
            }
            Ease::InOutBack => {
                let s = 1.70158;
                let t = t * 2.0;
                if t < 1. {
                    let s = s * 1.525;
                    0.5 * (t * t * ((s + 1.) * t - s))
                } else {
                    let t = t - 2.;
                    0.5 * (t * t * ((s + 1.) * t + s) + 2.)
                }
            }
            Ease::InBounce => 1.0 - Ease::OutBounce.map(1.0 - t),
            Ease::OutBounce => {
                if t < (1. / 2.75) {
                    return 7.5625 * t * t;
                }
                if t < (2. / 2.75) {
                    let t = t - (1.5 / 2.75);
                    return 7.5625 * t * t + 0.75;
                }
                if t < (2.5 / 2.75) {
                    let t = t - (2.25 / 2.75);
                    return 7.5625 * t * t + 0.9375;
                }
                let t = t - (2.625 / 2.75);
                7.5625 * t * t + 0.984375
            }
            Ease::InOutBounce => {
                if t < 0.5 {
                    Ease::InBounce.map(t * 2.) * 0.5
                } else {
                    Ease::OutBounce.map(t * 2. - 1.) * 0.5 + 0.5
                }
            }
            Ease::Bezier { cp0, cp1, cp2, cp3 } => {
                if t < 0. {
                    return 0.;
                }
                if t > 1. {
                    return 1.;
                }

                if (cp0 - cp1).abs() < 0.001 && (cp2 - cp3).abs() < 0.001 {
                    return t;
                }

                let epsilon = 1.0 / 200.0 * t;
                let cx = 3.0 * cp0;
                let bx = 3.0 * (cp2 - cp0) - cx;
                let ax = 1.0 - cx - bx;
                let cy = 3.0 * cp1;
                let by = 3.0 * (cp3 - cp1) - cy;
                let ay = 1.0 - cy - by;
                let mut u = t;

                for _i in 0..6 {
                    let x = ((ax * u + bx) * u + cx) * u - t;
                    if x.abs() < epsilon {
                        return ((ay * u + by) * u + cy) * u;
                    }
                    let d = (3.0 * ax * u + 2.0 * bx) * u + cx;
                    if d.abs() < 1e-6 {
                        break;
                    }
                    u -= x / d;
                }

                if t > 1. {
                    return (ay + by) + cy;
                }
                if t < 0. {
                    return 0.0;
                }

                let mut w = 0.0;
                let mut v = 1.0;
                u = t;
                for _i in 0..8 {
                    let x = ((ax * u + bx) * u + cx) * u;
                    if (x - t).abs() < epsilon {
                        return ((ay * u + by) * u + cy) * u;
                    }

                    if t > x {
                        w = u;
                    } else {
                        v = u;
                    }
                    u = (v - w) * 0.5 + w;
                }

                ((ay * u + by) * u + cy) * u
            }
        }
    }
}

/// Represents a single value that changes during the course of an animation.
/// Should remain consistent in its type and what it represents between the
/// different animations that you pass into a single [`Animator`].
///
/// `key_frames` are tuples, where the first value is the fraction between 0 and
/// 1 that represents how much of the animation has been played so far, and the
/// second value is the actual value that this track should take on at that time.
#[derive(Clone, Debug, PartialEq)]
pub enum Track {
    Float { ease: Ease, key_frames: &'static [(f64, f32)] },
    Vec2 { ease: Ease, key_frames: &'static [(f64, Vec2)] },
    Vec3 { ease: Ease, key_frames: &'static [(f64, Vec3)] },
    Vec4 { ease: Ease, key_frames: &'static [(f64, Vec4)] },
}

impl Track {
    fn compute_track_float(time: f64, track: &[(f64, f32)], init: f32, ease: &Ease) -> f32 {
        if track.is_empty() {
            return init;
        }
        fn lerp(a: f32, b: f32, f: f32) -> f32 {
            a * (1.0 - f) + b * f
        }
        // find the 2 keys we want
        for i in 0..track.len() {
            if time >= track[i].0 {
                // we found the left key
                let val1 = &track[i];
                if i == track.len() - 1 {
                    // last key
                    return val1.1;
                }
                let val2 = &track[i + 1];
                // lerp it
                let f = ease.map((time - val1.0) / (val2.0 - val1.0)) as f32;
                return lerp(val1.1, val2.1, f);
            }
        }
        let val2 = &track[0];
        let f = ease.map(time / val2.0) as f32;
        lerp(init, val2.1, f)
    }

    fn compute_track_vec2(time: f64, track: &[(f64, Vec2)], init: Vec2, ease: &Ease) -> Vec2 {
        if track.is_empty() {
            return init;
        }
        fn lerp(a: Vec2, b: Vec2, f: f32) -> Vec2 {
            let nf = 1.0 - f;
            Vec2 { x: a.x * nf + b.x * f, y: a.y * nf + b.y * f }
        }
        // find the 2 keys we want
        for i in 0..track.len() {
            if time >= track[i].0 {
                // we found the left key
                let val1 = &track[i];
                if i == track.len() - 1 {
                    // last key
                    return val1.1;
                }
                let val2 = &track[i + 1];
                // lerp it
                let f = ease.map((time - val1.0) / (val2.0 - val1.0)) as f32;
                return lerp(val1.1, val2.1, f);
            }
        }
        let val2 = &track[0];
        let f = ease.map(time / val2.0) as f32;
        lerp(init, val2.1, f)
    }

    fn compute_track_vec3(time: f64, track: &[(f64, Vec3)], init: Vec3, ease: &Ease) -> Vec3 {
        if track.is_empty() {
            return init;
        }
        fn lerp(a: Vec3, b: Vec3, f: f32) -> Vec3 {
            let nf = 1.0 - f;
            Vec3 { x: a.x * nf + b.x * f, y: a.y * nf + b.y * f, z: a.z * nf + b.z * f }
        }
        // find the 2 keys we want
        for i in 0..track.len() {
            if time >= track[i].0 {
                // we found the left key
                let val1 = &track[i];
                if i == track.len() - 1 {
                    // last key
                    return val1.1;
                }
                let val2 = &track[i + 1];
                // lerp it
                let f = ease.map((time - val1.0) / (val2.0 - val1.0)) as f32;
                return lerp(val1.1, val2.1, f);
            }
        }
        let val2 = &track[0];
        let f = ease.map(time / val2.0) as f32;
        lerp(init, val2.1, f)
    }

    fn compute_track_vec4(time: f64, track: &[(f64, Vec4)], init: Vec4, ease: &Ease) -> Vec4 {
        if track.is_empty() {
            return init;
        }
        fn lerp(a: Vec4, b: Vec4, f: f32) -> Vec4 {
            let nf = 1.0 - f;
            Vec4 { x: a.x * nf + b.x * f, y: a.y * nf + b.y * f, z: a.z * nf + b.z * f, w: a.w * nf + b.w * f }
        }
        // find the 2 keys we want
        for i in 0..track.len() {
            if time >= track[i].0 {
                // we found the left key
                let val1 = &track[i];
                if i == track.len() - 1 {
                    // last key
                    return val1.1;
                }
                let val2 = &track[i + 1];
                // lerp it
                let f = ease.map((time - val1.0) / (val2.0 - val1.0)) as f32;
                return lerp(val1.1, val2.1, f);
            }
        }
        let val2 = &track[0];
        let f = ease.map(time / val2.0) as f32;
        lerp(init, val2.1, f)
    }
}
