use std::{sync::RwLock, time::SystemTime};

use log::{warn, Level, Metadata, Record};
use serde::{Deserialize, Serialize};

const SEVERE_LOG_BUFFER_SIZE: usize = 15;

static LOGGER: HubRouterLogger = HubRouterLogger;
pub static SEVERE_LOG_STORE: RwLock<SevereLogStore> = RwLock::new(SevereLogStore {
    logs: Vec::new(),
    log_buffer_size: SEVERE_LOG_BUFFER_SIZE,
    log_buffer_idx: 0,
});

#[derive(Serialize, Deserialize)]
pub struct SevereLogStore {
    logs: Vec<SevereLog>,
    log_buffer_size: usize,
    log_buffer_idx: usize,
}

impl SevereLogStore {
    pub fn save_log(&mut self, log: SevereLog) {
        // First, see if any previously stored log has the exact
        // same message. If it does, just update the time on that one.
        for saved_log in &mut self.logs {
            if saved_log.log == log.log {
                saved_log.time = log.time;
                return;
            }
        }

        // Otherwise, save the log
        if self.logs.len() < self.log_buffer_size {
            self.logs.push(log);
        } else {
            self.logs[self.log_buffer_idx] = log;
        }

        self.log_buffer_idx += 1;
        self.log_buffer_idx %= self.log_buffer_size;
    }
}

pub struct HubRouterLogger;

#[derive(Serialize, Deserialize)]
pub struct SevereLog {
    log: String,
    time: SystemTime,
}

impl HubRouterLogger {
    pub fn init() {
        match log::set_logger(&LOGGER) {
            Ok(()) => log::set_max_level(log::LevelFilter::Info),
            Err(e) => {
                warn!("Unable to load logger: {} - no logs will be captured", e);
            }
        }
    }
}

impl log::Log for HubRouterLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());

            if matches!(record.metadata().level(), Level::Warn | Level::Error) {
                match SEVERE_LOG_STORE.write() {
                    Ok(mut handle) => {
                        handle.save_log(SevereLog {
                            log: record.args().to_string(),
                            time: SystemTime::now(),
                        });
                    }
                    Err(e) => {
                        eprintln!(
                            "Unable to acquire write lock to add severe lock to store: {}",
                            e
                        );
                    }
                }
            }
        }
    }

    fn flush(&self) {}
}
