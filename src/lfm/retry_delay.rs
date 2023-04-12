use std::time::Duration;

#[derive(Debug, Clone, Default)]

pub(crate) struct RetryDelay {
    n: usize,
    max_retry: usize,
}

impl RetryDelay {
    pub fn new(max_retry: usize) -> Self {
        Self { n: 0, max_retry }
    }
}

impl Iterator for RetryDelay {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.max_retry == self.n {
            return None;
        }

        if self.n == 0 {
            self.n += 1;
            return Some(Duration::from_millis(0));
        }

        let jitter = rand::random::<u64>() % 1000;
        let value = Duration::from_millis(2_u64.pow(self.n as u32) * 1000 - jitter);

        self.n += 1;

        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_delay() {
        let mut delay = RetryDelay::new(5);
        let n = delay.next().unwrap();
        assert_eq!(n, Duration::from_millis(0));
        let n = delay.next().unwrap();
        assert!(n < Duration::from_millis(2000));
        let n = delay.next().unwrap();
        assert!(n < Duration::from_millis(4000));
        let n = delay.next().unwrap();
        assert!(n < Duration::from_millis(8000));
        let n = delay.next().unwrap();
        assert!(n < Duration::from_millis(16000));
        assert!(delay.next().is_none());
    }
}
