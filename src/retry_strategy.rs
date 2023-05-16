use std::time::Duration;

pub trait RetryStrategy {
    fn should_retry_after(&self, attempt: usize) -> Option<Duration>;
}

pub struct JitteredBackoff {
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

impl Default for JitteredBackoff {
    fn default() -> Self {
        JitteredBackoff::new(5)
    }
}
