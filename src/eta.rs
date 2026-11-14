//! Estimation of the remaining time (ETA).

use crate::style::{Color, Style};
use core::fmt;
use core::time::Duration;

/// The state of the ETA estimation of a [`ProgressBar`](crate::pb::ProgressBar).
///
/// The speed is an exponential moving average of the progress over time, updated at most once per
/// `update_period`.
pub struct EtaData {
    smoothing_factor: f64,
    last_progress: usize,
    last_time: Duration,
    speed: f64,
    pub(crate) color: Option<Color>,
    pub(crate) style: Option<Style>,
    pub(crate) update_period: Duration,
}

impl EtaData {
    pub(crate) fn new(smoothing_factor: f64, now: Duration) -> Self {
        EtaData {
            smoothing_factor,
            last_progress: 0,
            last_time: now,
            speed: 0.0,
            color: None,
            style: None,
            update_period: Duration::from_secs(1),
        }
    }

    /// Restarts the estimation from `progress`, forgetting the measured speed.
    pub(crate) fn reset(&mut self, progress: usize, now: Duration) {
        self.last_progress = progress;
        self.last_time = now;
        self.speed = 0.0;
    }

    /// Records that the progress increased to `progress` at time `now`.
    pub(crate) fn update(&mut self, progress: usize, now: Duration) {
        let delta_time = now.saturating_sub(self.last_time).as_secs_f64();

        // Only update EMA if a meaningful time window has elapsed
        // (and some time elapsed at all, a coarse clock could otherwise give an infinite speed)
        if delta_time >= self.update_period.as_secs_f64() && delta_time > 0.0 {
            let delta_progress = (progress - self.last_progress) as f64;
            let instant_speed = delta_progress / delta_time;
            if self.speed == 0.0 {
                self.speed = instant_speed;
            } else {
                self.speed = self.smoothing_factor * instant_speed + (1.0 - self.smoothing_factor) * self.speed;
            }
            self.last_progress = progress;
            self.last_time = now;
        }
    }

    /// The estimated remaining time, if it is known and the progress isn't complete.
    pub(crate) fn remaining(&self, progress: usize, max: usize) -> Option<Remaining> {
        if max == 0 || self.speed == 0.0 || progress == max {
            return None;
        }
        let remaining_progress = max.saturating_sub(progress) as f64;
        Some(Remaining(remaining_progress / self.speed))
    }
}

/// A remaining time in seconds, displayed in a human-readable unit (`3000ms`, `18s`, `5 minutes`...).
pub(crate) struct Remaining(f64);

impl fmt::Display for Remaining {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SECS_110: usize = 110 * 1000;
        const MINS_110: usize = 110 * 60 * 1000;
        const HOURS_46: usize = 46 * 60 * 60 * 1000;

        let remaining_time = self.0;
        let remaining_ms = (remaining_time * 1000.0) as usize;

        #[allow(overlapping_range_endpoints)]
        #[allow(clippy::match_overlapping_arm)]
        match remaining_ms {
            0..=3_000 => write!(f, "{}ms", remaining_ms),
            3_001..=SECS_110 => write!(f, "{}s", ceil_usize(remaining_time)),
            SECS_110..=MINS_110 => write!(f, "{} minutes", ceil_usize(remaining_time / 60.)),
            MINS_110..=HOURS_46 => write!(f, "{} hours", ceil_usize(remaining_time / 3600.)),
            _ => write!(f, "{} days", ceil_usize(remaining_time / 86400.)),
        }
    }
}

/// Rounds a non-negative float up, saturating at `usize::MAX`.
// `f64::ceil` isn't available in `core`, this will allow dropping `std`.
fn ceil_usize(x: f64) -> usize {
    let truncated = x as usize;
    if (truncated as f64) < x {
        truncated.saturating_add(1)
    } else {
        truncated
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn ceil() {
        assert_eq!(ceil_usize(0.0), 0);
        assert_eq!(ceil_usize(1.0), 1);
        assert_eq!(ceil_usize(1.2), 2);
        assert_eq!(ceil_usize(f64::INFINITY), usize::MAX);
    }

    #[test]
    fn units() {
        let remaining = |secs: f64| Remaining(secs).to_string();
        assert_eq!(remaining(0.5), "500ms");
        assert_eq!(remaining(3.0), "3000ms");
        assert_eq!(remaining(10.2), "11s");
        assert_eq!(remaining(120.0), "2 minutes");
        assert_eq!(remaining(2.0 * 3600.0), "2 hours");
        assert_eq!(remaining(4.0 * 86400.0), "4 days");
    }

    #[test]
    fn speed() {
        let mut eta = EtaData::new(0.2, Duration::ZERO);
        assert!(eta.remaining(0, 100).is_none());

        eta.update(10, Duration::from_secs(2)); // 5 items/s
        assert_eq!(eta.remaining(10, 100).unwrap().to_string(), "18s");

        // no time elapsed: the speed isn't updated
        eta.update(20, Duration::from_secs(2));
        assert_eq!(eta.remaining(20, 100).unwrap().to_string(), "16s");

        assert!(eta.remaining(100, 100).is_none());
        eta.reset(0, Duration::from_secs(3));
        assert!(eta.remaining(0, 100).is_none());
    }
}
