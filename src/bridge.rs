use std::sync::mpsc;
use tokio::runtime::Runtime;

use crate::state::TaskResult;

pub struct AsyncBridge {
    runtime: Runtime,
    tx: mpsc::Sender<TaskResult>,
    rx: mpsc::Receiver<TaskResult>,
}

impl AsyncBridge {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");
        let (tx, rx) = mpsc::channel();
        Self { runtime, tx, rx }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = TaskResult> + Send + 'static,
    {
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let result = future.await;
            let _ = tx.send(result);
        });
    }

    pub fn poll_results(&self) -> Vec<TaskResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.rx.try_recv() {
            results.push(result);
        }
        results
    }
}
