use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use tokio_util::sync::CancellationToken;

pub struct GlobalRequestQueue {
    // Khóa chung: đảm bảo các request không chạy đồng thời.
    lock: Mutex<()>,

    // Khoảng cách tối thiểu giữa hai request toàn hệ thống.
    interval: Duration,

    // Thời điểm request gần nhất.
    last_request: Mutex<Option<Instant>>,

    // Cooldown riêng cho từng mã số thuế.
    tax_cool_downs: Mutex<HashMap<String, Instant>>,

    // Tổng số request đang chờ.
    waiting_requests: AtomicUsize,
}

impl GlobalRequestQueue {
    pub fn new(interval: Duration) -> Arc<Self> {
        Arc::new(Self {
            lock: Mutex::new(()),
            interval,
            last_request: Mutex::new(None),
            tax_cool_downs: Mutex::new(HashMap::new()),
            waiting_requests: AtomicUsize::new(0),
        })
    }

    /// Lấy thời gian cooldown còn lại của một MST.
    async fn tax_code_cooldown_remaining(&self, tax_code: &str) -> Option<Duration> {
        let mut cooldowns = self.tax_cool_downs.lock().await;

        match cooldowns.get(tax_code).copied() {
            Some(until) => {
                let now = Instant::now();

                if now < until {
                    Some(until - now)
                } else {
                    cooldowns.remove(tax_code);
                    None
                }
            }
            None => None,
        }
    }

    /// Chờ cooldown riêng của MST.
    async fn wait_for_tax_code(
        &self,
        tax_code: &str,
        cancel: &CancellationToken,
    ) -> anyhow::Result<()> {
        loop {
            if cancel.is_cancelled() {
                anyhow::bail!("Request cancelled");
            }

            let remaining = self.tax_code_cooldown_remaining(tax_code).await;

            match remaining {
                Some(duration) => {
                    println!(
                        "[request-queue][{}] Đang cooldown, còn {:.1} giây",
                        tax_code,
                        duration.as_secs_f64()
                    );

                    tokio::select! {
                        _ = cancel.cancelled() => {
                            anyhow::bail!("Request cancelled");
                        }
                        _ = sleep(duration) => {}
                    }
                }

                None => return Ok(()),
            }
        }
    }

    /// Thực hiện request theo giới hạn toàn cục và cooldown riêng MST.
    pub async fn execute<F, Fut, T>(
        &self,
        tax_code: &str,
        cancel: &CancellationToken,
        request: F,
    ) -> anyhow::Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
    {
        // Tăng số request đang chờ.
        self.waiting_requests.fetch_add(1, Ordering::Relaxed);

        // FnOnce chỉ được gọi một lần khi thực sự gửi request.
        let mut request = Some(request);

        loop {
            if cancel.is_cancelled() {
                self.waiting_requests.fetch_sub(1, Ordering::Relaxed);
                anyhow::bail!("Request cancelled");
            }

            // 1. Chờ cooldown riêng của MST trước khi lấy khóa toàn cục.
            if let Err(err) = self.wait_for_tax_code(tax_code, cancel).await {
                self.waiting_requests.fetch_sub(1, Ordering::Relaxed);
                return Err(err);
            }

            // 2. Lấy khóa request toàn cục.
            let guard = tokio::select! {
                _ = cancel.cancelled() => {
                    self.waiting_requests.fetch_sub(1, Ordering::Relaxed);
                    anyhow::bail!("Request cancelled");
                }
                guard = self.lock.lock() => guard,
            };

            if cancel.is_cancelled() {
                drop(guard);

                self.waiting_requests.fetch_sub(1, Ordering::Relaxed);

                anyhow::bail!("Request cancelled");
            }

            // 3. Kiểm tra lại cooldown sau khi lấy được khóa.
            // Tránh trường hợp MST vừa bị 429 trong lúc đang chờ khóa.
            if let Some(remaining) = self.tax_code_cooldown_remaining(tax_code).await {
                // Quan trọng: giải phóng khóa toàn cục trước khi chờ.
                drop(guard);

                tokio::select! {
                    _ = cancel.cancelled() => {
                        self.waiting_requests.fetch_sub(1, Ordering::Relaxed);
                        anyhow::bail!("Request cancelled");
                    }
                    _ = sleep(remaining) => {}
                }

                continue;
            }

            // 4. Đảm bảo khoảng cách tối thiểu giữa hai request.
            if let Some(last) = *self.last_request.lock().await {
                let elapsed = last.elapsed();

                if elapsed < self.interval {
                    let remaining = self.interval - elapsed;

                    tokio::select! {
                        _ = cancel.cancelled() => {
                            drop(guard);

                            self.waiting_requests.fetch_sub(1, Ordering::Relaxed);

                            anyhow::bail!("Request cancelled");
                        }
                        _ = sleep(remaining) => {}
                    }
                }
            }

            if cancel.is_cancelled() {
                drop(guard);

                self.waiting_requests.fetch_sub(1, Ordering::Relaxed);

                anyhow::bail!("Request cancelled");
            }

            // 5. Kiểm tra lại cooldown trước khi gửi request.
            if let Some(remaining) = self.tax_code_cooldown_remaining(tax_code).await {
                drop(guard);

                tokio::select! {
                    _ = cancel.cancelled() => {
                        self.waiting_requests.fetch_sub(1, Ordering::Relaxed);
                        anyhow::bail!("Request cancelled");
                    }
                    _ = sleep(remaining) => {}
                }

                continue;
            }

            // 6. Ghi nhận thời điểm bắt đầu request.
            *self.last_request.lock().await = Some(Instant::now());

            self.waiting_requests.fetch_sub(1, Ordering::Relaxed);

            println!("[request-queue][{}] Bắt đầu request", tax_code);

            // 7. Giữ khóa toàn cục đến khi request hoàn tất.
            let request_fn = request.take().unwrap();

            let result = tokio::select! {
                _ = cancel.cancelled() => {
                    anyhow::bail!("Request cancelled");
                }
                result = request_fn() => result,
            };

            drop(guard);

            return result;
        }
    }

    /// Đặt cooldown riêng cho một mã số thuế.
    pub async fn set_tax_code_cooldown(&self, tax_code: &str, duration: Duration) {
        let new_until = Instant::now() + duration;

        let mut cooldowns = self.tax_cool_downs.lock().await;

        let until = cooldowns.entry(tax_code.to_string()).or_insert(new_until);

        // Nếu đã có cooldown dài hơn thì giữ thời gian dài hơn.
        if new_until > *until {
            *until = new_until;
        }

        println!(
            "[request-queue][{}] Bị giới hạn 429, cooldown {} giây",
            tax_code,
            duration.as_secs()
        );
    }

    /// Số request hiện đang chờ.
    pub fn waiting_count(&self) -> usize {
        self.waiting_requests.load(Ordering::Relaxed)
    }
}
