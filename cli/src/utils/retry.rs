use anyhow::Result;
use reqwest::StatusCode;
use std::time::Duration;

pub fn retry_request<F>(mut request_fn: F) -> Result<reqwest::blocking::Response>
where
    F: FnMut() -> Result<reqwest::blocking::Response>,
{
    let max_retries = 3;
    let base_wait = Duration::from_secs(5);
    let backoff_factor: f64 = 2.0;

    for attempt in 0..max_retries {
        match request_fn() {
            Ok(response) => {
                let status = response.status();
                if should_retry_status(status) && attempt < max_retries - 1 {
                    let wait_factor = backoff_factor.powi(attempt);
                    let duration = base_wait.mul_f64(wait_factor);
                    log::warn!(
                        "{} for {} - retrying after {:?}.",
                        status,
                        response.url(),
                        duration
                    );
                    std::thread::sleep(duration);
                    continue;
                }

                return Ok(response);
            }
            Err(e) => {
                // Check if the error is a reqwest::Error we should retry
                let should_retry = e
                    .downcast_ref::<reqwest::Error>()
                    .map(should_retry_error)
                    .unwrap_or(false);

                if should_retry && attempt < max_retries - 1 {
                    let wait_factor = backoff_factor.powi(attempt);
                    let duration = base_wait.mul_f64(wait_factor);
                    log::warn!("{e} - retrying after {duration:?}.");
                    std::thread::sleep(duration);
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Final attempt (matches legacy client behavior)
    request_fn()
}

/// Determine if an HTTP status code should trigger a retry
fn should_retry_status(status: StatusCode) -> bool {
    status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
}

/// Determine if a reqwest error should trigger a retry
fn should_retry_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request()
}
