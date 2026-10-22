use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use vaone_tax_sync::TaxSyncWorker;

pub struct TaxSyncWorkerManager {
    worker: Arc<TaxSyncWorker>,
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    running: Arc<AtomicBool>,
}

impl TaxSyncWorkerManager {
    pub fn new(worker: Arc<TaxSyncWorker>) -> Self {
        Self {
            worker,
            task: Mutex::new(None),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let mut task = self.task.lock().map_err(|e| e.to_string())?;

        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let worker = Arc::clone(&self.worker);
        let running = Arc::clone(&self.running);

        worker.clone().start();

        *task = Some(tauri::async_runtime::spawn(async move {
            worker.run_workers().await;
            running.store(false, Ordering::SeqCst);
        }));

        Ok(())
    }

    pub fn stop(&self) {
        self.worker.stop();
    }
}
