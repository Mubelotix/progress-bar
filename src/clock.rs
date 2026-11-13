//! Time sources used for ETA estimation.

use core::time::Duration;

/// A monotonic time source, used to estimate the remaining time.
///
/// [`Clock::now`] returns the time elapsed since an arbitrary, fixed origin (e.g. boot).
/// Any `Fn() -> Duration` is a clock, so on embedded targets you can pass a function
/// reading your timer:
///
/// ```
/// use core::time::Duration;
/// use progress_bar::Clock;
///
/// fn uptime() -> Duration {
///     // read your hardware timer here
///     Duration::from_millis(0)
/// }
///
/// let clock: fn() -> Duration = uptime;
/// assert_eq!(clock.now(), Duration::ZERO);
/// ```
pub trait Clock {
    fn now(&self) -> Duration;
}

impl<F: Fn() -> Duration> Clock for F {
    fn now(&self) -> Duration {
        self()
    }
}

/// A clock that never advances. ETA is never displayed with this clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoClock;

impl Clock for NoClock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}

/// A clock based on [`std::time::Instant`]. This is the default clock.
#[derive(Debug, Clone, Copy)]
pub struct StdClock {
    origin: std::time::Instant,
}

impl Default for StdClock {
    fn default() -> Self {
        StdClock { origin: std::time::Instant::now() }
    }
}

impl Clock for StdClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}
