//! Version of [`std::time::Instant`] that also works in WebAssembly.
//!
//! Adapted from <https://github.com/rust-lang/rust/issues/48564#issuecomment-698712971>

use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

/// Version of [`std::time::Instant`] that also works in WebAssembly.
pub trait Instant<S>
where
    S: Sized,
{
    fn elapsed(&self) -> Duration;
    fn now() -> Self;
    fn duration_since(&self, earlier: UniversalInstant) -> Duration;
    fn checked_add(&self, duration: Duration) -> Option<S>;
    fn checked_sub(&self, duration: Duration) -> Option<S>;
}

/// Version of [`std::time::Instant`] that also works in WebAssembly.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniversalInstant(std::time::Instant);

#[cfg(not(target_arch = "wasm32"))]
impl Instant<Self> for UniversalInstant {
    /// See [`std::time::Instant::now`]
    fn now() -> Self {
        Self(std::time::Instant::now())
    }
    /// See [`std::time::Instant::duration_since`]
    fn duration_since(&self, earlier: UniversalInstant) -> Duration {
        self.0.duration_since(earlier.0)
    }
    /// See [`std::time::Instant::elapsed`]
    fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
    /// See [`std::time::Instant::checked_add`]
    fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add(duration).map(Self)
    }
    /// See [`std::time::Instant::checked_sub`]
    fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.0.checked_sub(duration).map(Self)
    }
}

#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

/// Version of [`std::time::Instant`] that also works in WebAssembly.
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniversalInstant(u64);

#[cfg(target_arch = "wasm32")]
impl Instant<Self> for UniversalInstant {
    /// See [`std::time::Instant::now`]
    fn now() -> Self {
        Self((unsafe { crate::cx_wasm32::performanceNow() } * 1000.0) as u64)
    }
    /// See [`std::time::Instant::duration_since`]
    fn duration_since(&self, earlier: UniversalInstant) -> Duration {
        Duration::from_micros(self.0 - earlier.0)
    }
    /// See [`std::time::Instant::elapsed`]
    fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }
    /// See [`std::time::Instant::checked_add`]
    fn checked_add(&self, duration: Duration) -> Option<Self> {
        match duration.as_micros().try_into() {
            Ok(duration) => self.0.checked_add(duration).map(|i| Self(i)),
            Err(_) => None,
        }
    }
    /// See [`std::time::Instant::checked_sub`]
    fn checked_sub(&self, duration: Duration) -> Option<Self> {
        match duration.as_micros().try_into() {
            Ok(duration) => self.0.checked_sub(duration).map(|i| Self(i)),
            Err(_) => None,
        }
    }
}

/// See `Add<Duration>` in [`std::time::Instant`].
impl Add<Duration> for UniversalInstant {
    type Output = UniversalInstant;
    fn add(self, other: Duration) -> UniversalInstant {
        self.checked_add(other).unwrap()
    }
}
/// See `Sub<Duration>` in [`std::time::Instant`].
impl Sub<Duration> for UniversalInstant {
    type Output = UniversalInstant;
    fn sub(self, other: Duration) -> UniversalInstant {
        self.checked_sub(other).unwrap()
    }
}
/// See `Sub<Instant>` in [`std::time::Instant`].
impl Sub<UniversalInstant> for UniversalInstant {
    type Output = Duration;
    fn sub(self, other: UniversalInstant) -> Duration {
        self.duration_since(other)
    }
}
/// See `AddAssign<Duration>` in [`std::time::Instant`].
impl AddAssign<Duration> for UniversalInstant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}
/// See `SubAssign<Duration>` in [`std::time::Instant`].
impl SubAssign<Duration> for UniversalInstant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}
