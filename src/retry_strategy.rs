//! # Retry strategy
//!
//! The default retry strategy and how to write your own custom retry logic.
use std::time::Duration;

/// Trait to define a retry strategy
pub trait RetryStrategy : Send + Sync {
    /// This function is called every time a request to the remote Last.fm APIs fails
    /// to determine if a retry attempt should be made and how much time to wait
    /// before the next attempt is made.
    ///
    /// When this function returns `None` there will be no more retries and the execution fails.
    /// When this function returns `Some(duration)` the client will wait as long as the specified duration
    /// before performing the request again.
    ///
    /// You could write a very simple retry strategy that always retries immediately as follow:
    ///
    /// ```rust
    /// use lastfm::retry_strategy::RetryStrategy;
    /// use std::time::Duration;
    ///
    /// struct AlwaysRetry {}
    ///
    /// impl RetryStrategy for AlwaysRetry {
    ///     fn should_retry_after(&self, attempt: usize) -> Option<Duration> {
    ///         Some(Duration::from_secs(0))
    ///     }
    /// }
    /// ```
    ///
    /// Or a strategy to never retry
    ///
    /// ```rust
    /// use lastfm::retry_strategy::RetryStrategy;
    /// use std::time::Duration;
    ///
    /// struct NeverRetry {}
    ///
    /// impl RetryStrategy for NeverRetry {
    ///     fn should_retry_after(&self, attempt: usize) -> Option<Duration> {
    ///         None
    ///     }
    /// }
    /// ```
    ///
    /// Check out the [`JitteredBackoff`] retry strategy
    /// and the `examples` folder for more examples.
    fn should_retry_after(&self, attempt: usize) -> Option<Duration>;
}

/// The default retry strategy.
///
/// It performs a Jittered backoff for a given maximum number of times.
///
/// The wait duration is calculated using the formula (in milliseconds):
///
/// ```plain
/// 2 ^ (num_retry) * 1000 - random_jitter
/// ```
///
/// Where `random_jitter` is a random number between `0` and `999`.
pub struct JitteredBackoff {
    /// The maximum number of retries before giving up
    max_retry: usize,
}

impl JitteredBackoff {
    pub fn new(max_retry: usize) -> Self {
        Self { max_retry }
    }
}

impl RetryStrategy for JitteredBackoff {
    fn should_retry_after(&self, num_retry: usize) -> Option<Duration> {
        if self.max_retry == num_retry {
            return None;
        }

        let jitter = rand::random::<u64>() % 1000;
        let value = Duration::from_millis(2_u64.pow(num_retry as u32) * 1000 - jitter);

        Some(value)
    }
}

/// Creates a new [`JitteredBackoff`] with the default maximum number of retries (5).
impl Default for JitteredBackoff {
    fn default() -> Self {
        JitteredBackoff::new(5)
    }
}
