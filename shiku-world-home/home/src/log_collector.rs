use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use ts_rs::TS;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct LogInfo(u32, String, String, String, String);

pub struct LogCollector {
    pub new_logs: Arc<Mutex<Vec<LogInfo>>>,
    pub log_archive: Arc<Mutex<Vec<LogInfo>>>,
    pub log_count: u32,
}

impl LogCollector {
    pub fn new() -> Self {
        Self {
            new_logs: Arc::new(Mutex::new(Vec::new())),
            log_archive: Arc::new(Mutex::new(Vec::new())),
            log_count: 0,
        }
    }

    pub fn log(&mut self, time: String, log_level: String, location: String, message: String) {
        if let Ok(ref mut new_logs) = self.new_logs.try_lock() {
            self.log_count += 1;
            let new_log = LogInfo(self.log_count, time, log_level, location, message);
            new_logs.push(new_log.clone());
            if let Ok(ref mut log_archive) = self.log_archive.try_lock() {
                log_archive.push(new_log);
                // Only preserve the last 100 logs in the archive
                let mut index = log_archive.len();
                log_archive.retain(|_| {
                    index -= 1;
                    index < 100
                });
            }
        }
    }

    pub fn get_new_logs(&self, log_buff: &mut Vec<LogInfo>) {
        if let Ok(ref mut new_logs) = self.new_logs.try_lock() {
            log_buff.append(new_logs);
        }
    }

    pub fn get_log_archive(&self) -> Vec<LogInfo> {
        self.log_archive
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }
}