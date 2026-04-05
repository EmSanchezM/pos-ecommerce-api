use std::future::Future;

use crate::InventoryError;

/// Default maximum number of retry attempts for optimistic lock conflicts.
pub const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Default base delay in milliseconds for exponential backoff between retries.
pub const DEFAULT_BASE_DELAY_MS: u64 = 10;

/// Retries an async operation on optimistic lock conflicts with exponential backoff.
///
/// Only retries when the operation returns `InventoryError::OptimisticLockError`.
/// All other errors are returned immediately.
///
/// The closure should re-read the entity from the database on each invocation
/// to obtain the latest version.
///
/// # Arguments
/// * `max_attempts` - Maximum number of attempts (including the first one)
/// * `base_delay_ms` - Base delay in milliseconds (doubles each retry: 10, 20, 40, ...)
/// * `operation` - Async closure that performs the operation
pub async fn retry_on_conflict<F, Fut, T>(
    max_attempts: u32,
    base_delay_ms: u64,
    mut operation: F,
) -> Result<T, InventoryError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, InventoryError>>,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match operation().await {
            Ok(value) => return Ok(value),
            Err(InventoryError::OptimisticLockError) if attempt < max_attempts => {
                let delay = base_delay_ms * (1 << (attempt - 1));
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_success_on_first_attempt() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let result = retry_on_conflict(3, 1, move || {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Ok::<_, InventoryError>(42)
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_success_after_retries() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let result = retry_on_conflict(3, 1, move || {
            let cc = cc.clone();
            async move {
                let count = cc.fetch_add(1, Ordering::SeqCst) + 1;
                if count < 3 {
                    Err(InventoryError::OptimisticLockError)
                } else {
                    Ok(99)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 99);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_exhausts_all_attempts() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let result: Result<i32, _> = retry_on_conflict(3, 1, move || {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Err(InventoryError::OptimisticLockError)
            }
        })
        .await;

        assert!(matches!(result, Err(InventoryError::OptimisticLockError)));
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_non_retryable_error_returns_immediately() {
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let result: Result<i32, _> = retry_on_conflict(3, 1, move || {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Err(InventoryError::NegativeStock)
            }
        })
        .await;

        assert!(matches!(result, Err(InventoryError::NegativeStock)));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}
