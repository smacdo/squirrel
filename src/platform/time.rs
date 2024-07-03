/// An opaque value representing a snapshot in time captured from the underlying
/// platform.
///
/// Implements a subset of `std::time::Instant`, see:
/// https://doc.rust-lang.org/std/time/struct.Instant.html
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct SystemTime {
    /// Normal non-wasm time measurement provided by std
    #[cfg(not(target_arch = "wasm32"))]
    instant: std::time::Instant,
    /// JavaScript measures time since January 1, 1970 00:00:00 UTC in
    /// milliseconds.
    #[cfg(target_arch = "wasm32")]
    millis_since_epoch: f64,
}

// TODO(scott): Implement other useful methods
//  - Add<Duration> -> SystemTime
//  - Sub<Duration> -> SystemTime
//  - Display/ToString
//  - Hash
//  - unit tests

impl SystemTime {
    /// Get the current system time.
    pub fn now() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self {
                    millis_since_epoch: js_sys::Date::now()
                }
            } else {
                Self {
                    instant: std::time::Instant::now()
                }
            }
        }
    }
}

impl std::ops::Sub<SystemTime> for SystemTime {
    type Output = std::time::Duration;

    fn sub(self, rhs: SystemTime) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::time::Duration::from_millis((self.millis_since_epoch - rhs.millis_since_epoch) as u64)
            } else {
                self.instant - rhs.instant
            }
        }
    }
}
