use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio_util::sync::CancellationToken;
use vaone_tax_sync::TaxSyncWorker;

pub struct TaxSyncWorkerManager {
    worker: Arc<TaxSyncWorker>,
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    cancel: Mutex<Option<CancellationToken>>,
    running: Arc<AtomicBool>,
}

impl TaxSyncWorkerManager {
    pub fn new(worker: Arc<TaxSyncWorker>) -> Self {
        Self {
            worker,
            task: Mutex::new(None),
            cancel: Mutex::new(None),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let mut task = self.task.lock().map_err(|e| e.to_string())?;

        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let cancel = CancellationToken::new();
        let worker = Arc::clone(&self.worker);
        let running = Arc::clone(&self.running);

        *self.cancel.lock().map_err(|e| e.to_string())? = Some(cancel.clone());

        self.running.store(true, Ordering::SeqCst);

        *task = Some(tauri::async_runtime::spawn(async move {
            worker.run_workers(cancel).await;
            running.store(false, Ordering::SeqCst);
        }));

        Ok(())
    }

    /// Chỉ gửi tín hiệu dừng, không chờ worker kết thúc.
    pub fn stop(&self) -> Result<(), String> {
        if let Some(cancel) = self.cancel.lock().map_err(|e| e.to_string())?.as_ref() {
            cancel.cancel();
        }

        Ok(())
    }

    /// Gửi tín hiệu dừng và chờ tất cả vòng lặp kết thúc.
    pub async fn stop_and_wait(&self) -> Result<(), String> {
        self.stop()?;

        let task = self.task.lock().map_err(|e| e.to_string())?.take();

        if let Some(task) = task {
            task.await
                .map_err(|e| format!("Worker task kết thúc bất thường: {e}"))?;
        }

        self.running.store(false, Ordering::SeqCst);
        *self.cancel.lock().map_err(|e| e.to_string())? = None;

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn notify_upload(&self) {
        self.worker.notify_upload();
    }

    pub fn notify_update(&self) {
        self.worker.notify_update();
    }

    pub fn notify_all(&self) {
        self.worker.notify_update();
        self.worker.notify_upload();
    }
}
