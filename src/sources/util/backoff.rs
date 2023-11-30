use std::{
    ops::{Mul, Sub},
    time::{Duration, Instant},
};

const MAX_DELAY: Duration = Duration::from_secs(60);
const BASE_DELAY: Duration = Duration::from_millis(100);
const LOGGING_INTERVAL: Duration = Duration::from_secs(1);

/// Maintains state for exponential backoff strategy
#[allow(dead_code)]
pub struct Backoff {
    step: u32,
}

#[allow(dead_code)]
impl Backoff {
    pub const fn new() -> Backoff {
        Backoff { step: 0 }
    }

    /// Resets the internal state to the base delay.
    pub fn reset(&mut self) {
        self.step = 0;
    }

    /// Gets the next delay.
    pub fn next(&mut self) -> Duration {
        let value = BASE_DELAY.mul(u32::pow(2, self.step));
        if value >= MAX_DELAY {
            return MAX_DELAY;
        }
        self.step += 1;
        value
    }
}

/// Maintains state for avoiding spamming the logs by allowing a single
/// message per interval
#[allow(dead_code)]
pub struct LogBackoff {
    t: Instant,
    interval: Duration,
}

// unused in tests and only relevant outside of this module
#[allow(dead_code)]
impl LogBackoff {
    pub fn new() -> Self {
        Self {
            interval: LOGGING_INTERVAL,
            t: Instant::now().sub(LOGGING_INTERVAL),
        }
    }

    /// Gets whether it should log for this call, modifying the internal state.
    pub fn should_log(&mut self) -> bool {
        if Instant::now().duration_since(self.t) < self.interval {
            false
        } else {
            self.t = Instant::now();
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::time::sleep;

    const TIMER_PRECISION: Duration = Duration::from_millis(20);

    #[tokio::test]
    async fn log_backoff_test() {
        const INTERVAL: Duration = Duration::from_millis(20);
        let mut backoff = LogBackoff {
            interval: INTERVAL,
            t: Instant::now().sub(INTERVAL + TIMER_PRECISION),
        };

        assert!(backoff.should_log(), "first call should log");
        assert!(!backoff.should_log(), "should not log until time elapses");

        sleep(INTERVAL).await;
        sleep(TIMER_PRECISION).await;

        assert!(
            backoff.should_log(),
            "after time passes, it should log once"
        );
        assert!(!backoff.should_log(), "should log only once");
    }

    #[tokio::test]
    async fn exponential_backoff_test() {
        let mut backoff = Backoff::new();
        for i in 0..10 {
            assert_eq!(backoff.next(), BASE_DELAY * (1 << i));
        }

        // i << 10 = 1024
        // 1024 * BASE_DELAY = 102400, which is greater than MAX_DELAY
        assert_eq!(backoff.next(), MAX_DELAY);

        backoff.reset();
        assert_eq!(backoff.next(), BASE_DELAY);
        assert_eq!(backoff.next(), BASE_DELAY * 2);
    }
}
