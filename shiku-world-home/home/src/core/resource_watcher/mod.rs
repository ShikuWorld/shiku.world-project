use std::sync::mpsc::Receiver;
use std::{path::Path, time::Duration};

use log::error;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

use crate::core::get_out_dir;

pub struct ResourceWatcher {
    pub watch_path: String,
    pub receiver: Receiver<DebounceEventResult>,
    pub debouncer: Debouncer<RecommendedWatcher>,
}

impl ResourceWatcher {
    pub fn new(watch_path: &str) -> ResourceWatcher {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut debouncer = new_debouncer(Duration::from_millis(500), None, sender).unwrap();

        if let Err(err) = debouncer.watcher().watch(
            get_out_dir().join(&Path::new(watch_path.into())).as_path(),
            RecursiveMode::Recursive,
        ) {
            error!("{}", err);
        }

        ResourceWatcher {
            watch_path: watch_path.into(),
            receiver,
            debouncer,
        }
    }
}
