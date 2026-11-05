use std::collections::HashMap;
use std::sync::Mutex;

use vaone_tax_sync::models::{TaskProgress, TaskStatus};

#[derive(Default)]
pub struct ProgressStore {
    pub tasks: Mutex<HashMap<String, TaskProgress>>,
}

impl ProgressStore {
    pub fn update(&self, progress: TaskProgress) {
        let key = format!("{}:{}", progress.tax_code, progress.task.label());

        let mut tasks = self.tasks.lock().unwrap();

        if matches!(progress.status, TaskStatus::Running) {
            tasks.insert(key, progress);
        } else {
            // Tác vụ đã kết thúc thì không còn là tác vụ đang chạy.
            tasks.remove(&key);
        }
    }

    pub fn get_running(&self) -> Vec<TaskProgress> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }
}
