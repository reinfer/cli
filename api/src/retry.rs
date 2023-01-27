use reqwest::{Response, Result};
use std::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    thread::sleep,
    time::Duration,
};

/// Strategy to use if retrying .
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RetryStrategy {
    /// The first request by the client will not be retried, but subsequent requests will.
    /// This allows fast failure if the client cannot reach the API endpoint at all, but
    /// helps to mitigate failure in long-running operations spanning multiple requests.
    Automatic,
    /// Always attempt to retry requests.
    Always,
}

/// Configuration for the Reinfer client if retrying timeouts.
#[derive(Clone, Debug, PartialEq)]
pub struct RetryConfig {
    /// Strategy for when to retry after a timeout
    pub strategy: RetryStrategy,
    /// Maximum number of retries to attempt.
    pub max_retry_count: u8,
    /// Amount of time to wait for first retry.
    pub base_wait: Duration,
    /// Amount of time to scale retry waits. The wait before retry N is an exponential backoff
    /// using the formula `wait = retry_wait * (backoff_factor * N)`.
    pub backoff_factor: f64,
}

#[derive(Debug)]
pub(crate) struct Retrier {
    config: RetryConfig,
    is_first_request: AtomicBool,
}

impl Retrier {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            is_first_request: AtomicBool::new(true),
        }
    }

    pub async fn with_retries<CallbackT, ResponseT>(
        &self,
        send_request: CallbackT,
    ) -> Result<Response>
    where
        CallbackT: Fn() -> ResponseT,
        ResponseT: Future<Output = Result<Response>>,
    {
        if self.is_first_request.swap(false, SeqCst)
            && self.config.strategy == RetryStrategy::Automatic
        {
            return send_request().await;
        }

        for i_retry in 0..self.config.max_retry_count {
            macro_rules! warn_and_sleep {
                ($src:expr) => {{
                    let wait_factor = self.config.backoff_factor.powi(i_retry.into());
                    let duration = self.config.base_wait.mul_f64(wait_factor);
                    log::warn!("{} - retrying after {:?}.", $src, duration);
                    sleep(duration)
                }};
            }

            match send_request().await {
                Ok(response) if response.status().is_server_error() => {
                    warn_and_sleep!(format!("{} for {}", response.status(), response.url()))
                }
                Err(error) if error.is_timeout() => warn_and_sleep!(error),
                // If anything else, just return it immediately
                result => return result,
            }
        }

        // On last retry don't handle the error, just propagate all errors.
        send_request().await
    }
}

#[cfg(test)]
mod tests {
    use super::{Retrier, RetryConfig, RetryStrategy};
    use mockito::{mock, server_address};
    use reqwest::{get, Client};
    use std::thread::sleep;
    use std::time::Duration;

    #[tokio::test]
    async fn test_always_retry() {
        let mut handler = Retrier::new(RetryConfig {
            strategy: RetryStrategy::Always,
            max_retry_count: 5,
            base_wait: Duration::from_secs(0),
            backoff_factor: 0.0,
        });

        // Does not attempt to retry on success
        let ok = mock("GET", "/").expect(1).create();
        assert!(
            handler
                .with_retries(|| async { get(&format!("http://{}", server_address())).await })
                .await
                .unwrap()
                .status()
                == 200
        );
        ok.assert();

        // Retries up to N times on timeout.
        for i_retry in 0..10 {
            let err = mock("GET", "/")
                .with_status(500)
                .expect((i_retry + 1).into())
                .create();
            handler.config.max_retry_count = i_retry;
            assert!(
                handler
                    .with_retries(|| async { get(&format!("http://{}", server_address())).await })
                    .await
                    .unwrap()
                    .status()
                    == 500
            );
            err.assert();
        }
    }

    #[tokio::test]
    async fn test_automatic_retry() {
        let mut handler = Retrier::new(RetryConfig {
            strategy: RetryStrategy::Automatic,
            max_retry_count: 5,
            base_wait: Duration::from_secs(0),
            backoff_factor: 0.0,
        });

        // Does not attempt to retry on failure of first request
        let err = mock("GET", "/").with_status(500).expect(1).create();
        assert!(
            handler
                .with_retries(|| async { get(&format!("http://{}", server_address())).await })
                .await
                .unwrap()
                .status()
                == 500
        );
        err.assert();

        // Does not attempt to retry on success
        let ok = mock("GET", "/").expect(1).create();
        assert!(
            handler
                .with_retries(|| async { get(&format!("http://{}", server_address())).await })
                .await
                .unwrap()
                .status()
                == 200
        );
        ok.assert();

        // Retries up to N times on timeout for non-first-requests.
        for i_retry in 0..10 {
            let err = mock("GET", "/")
                .with_status(500)
                .expect((i_retry + 1).into())
                .create();
            handler.config.max_retry_count = i_retry;
            assert!(
                handler
                    .with_retries(|| async { get(&format!("http://{}", server_address())).await })
                    .await
                    .unwrap()
                    .status()
                    == 500
            );
            err.assert();
        }
    }

    #[tokio::test]
    async fn test_timeout_retry() {
        let handler = Retrier::new(RetryConfig {
            strategy: RetryStrategy::Always,
            max_retry_count: 1,
            base_wait: Duration::from_secs(0),
            backoff_factor: 0.0,
        });

        // Should retry on the timeout
        let timeout = mock("GET", "/")
            .with_body_from_fn(|_| {
                sleep(Duration::from_secs_f64(0.2));
                Ok(())
            })
            .expect(2)
            .create();
        let client = Client::new();
        assert!(handler
            .with_retries(|| async {
                let response = client
                    .get(&format!("http://{}", server_address()))
                    .timeout(Duration::from_secs_f64(0.1))
                    .send()
                    .await?;
                // This is a bit of a hack to force a timeout
                let _ = response.text().await?;
                unreachable!()
            })
            .await
            .unwrap_err()
            .is_timeout());
        timeout.assert();
    }
}
