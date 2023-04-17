use std::{fs::File as FsFile, path::Path, io::Write, sync::Arc};

use config::{Config, ConfigError, File};

use crate::{HubMap};

pub const REAPER_THREAD_INTERVAL_SECS: &str = "reaper_thread_interval";
pub const REAPER_MAX_SESSION_LIFETIME_MINS: &str = "reaper_max_session_mins";
pub const HEALTHCHECK_THREAD_INTERVAL_SECS: &str = "healthcheck_thread_interval";
pub const API_BIND_IP: &str = "api_bind_ip";
pub const API_BIND_PORT: &str = "api_bind_port";
pub const PROXY_BIND_IP: &str = "proxy_bind_ip";
pub const PROXY_BIND_PORT: &str = "proxy_bind_port";
pub const HUBS_FILE_PATH: &str = "hubs_file_path";

pub fn load_in_config(source: &String) -> Result<Config, ConfigError> {
    let mut builder = Config::builder();
    if Path::new(source).exists() {
        builder = builder.add_source(File::with_name(source));
    } else {
        eprintln!("Warning: given configuration file {} does not exist - falling back to default configuration", source);
    }

    builder = builder.set_default(REAPER_THREAD_INTERVAL_SECS, 60)?;
    builder = builder.set_default(REAPER_MAX_SESSION_LIFETIME_MINS, 30)?;
    builder = builder.set_default(API_BIND_IP, "0.0.0.0")?;
    builder = builder.set_default(API_BIND_PORT, 8080)?;
    builder = builder.set_default(PROXY_BIND_IP, "0.0.0.0")?;
    builder = builder.set_default(PROXY_BIND_PORT, 6543)?;
    builder = builder.set_default(HEALTHCHECK_THREAD_INTERVAL_SECS, 1)?;
    builder = builder.set_default(HUBS_FILE_PATH, "./hubs.ser")?;

    builder.build()
}

pub fn update_hub_file(hubs: Arc<HubMap>, hub_file_path: String) -> Result<(), String> {
    let serialized =
        match serde_json::to_string(&hubs.iter().map(|e| e.value().clone().meta).collect::<Vec<_>>()) {
            Ok(str) => str,
            Err(e) => return Err(format!("Error serializing hubs: {}", e.to_string())),
        };

    let mut hub_file = match FsFile::create(hub_file_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Error opening hub file: {}", e)),
    };

    let bytes = serialized.as_bytes();
    match hub_file.write(bytes) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error writing serialized hubs {}", e)
        },
    };

    return Ok(());
}
