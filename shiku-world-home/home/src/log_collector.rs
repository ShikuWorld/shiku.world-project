use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use ts_rs::TS;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct LogInfo(String, String, String, String);

pub struct LogCollector {
    pub new_logs: Arc<Mutex<Vec<LogInfo>>>,
    pub log_archive: Arc<Mutex<Vec<LogInfo>>>,
}

impl LogCollector {
    pub fn new() -> Self {
        Self {
            new_logs: Arc::new(Mutex::new(Vec::new())),
            log_archive: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn log(&self, time: String, log_level: String, location: String, message: String) {
        if let Ok(mut logs) = self.log_archive.lock() {
            logs.push(LogInfo(time, log_level, location, message));
        }
    }

    pub fn get_new_logs_and_archive(&self, log_buff: &mut Vec<LogInfo>) {
        if let Ok(ref mut new_logs) = self.new_logs.lock() {
            log_buff.clone_from(new_logs);
            if let Ok(ref mut log_archive) = self.log_archive.lock() {
                log_archive.append(new_logs);
                // Only preserve the last 100 logs in the archive
                let mut index = log_archive.len();
                log_archive.retain(|_| {
                    index -= 1;
                    index < 100
                });
            }
        }
    }

    pub fn get_log_archive(&self) -> Vec<LogInfo> {
        self.log_archive
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }
}
