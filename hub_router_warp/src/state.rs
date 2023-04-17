use crate::HubMap;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, File},
    io::Write,
    net::Ipv4Addr,
    sync::RwLock,
};

#[derive(Debug, Clone)]
enum PersistPath {
    Path(String),
}

impl Into<String> for PersistPath {
    fn into(self) -> String {
        match self {
            PersistPath::Path(s) => s,
        }
    }
}

impl Default for PersistPath {
    fn default() -> Self {
        Self::Path("./config.json".into())
    }
}

/// HubRouterState is an encapsulation of all configurable state within a
/// Hub Router instance. the notable exception to this is the session state
/// table, as that is configured by the system at runtime dynamically and
/// not persisted to disk.
///
/// The most notable data within this structure are the list of registered
/// Hubs to route traffic to (this includes runtime state including the session
/// fullness, but this data is not writeable via the API and not persisted to
/// disk). It also includes values for
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HubRouterState {
    #[serde(serialize_with = "crate::utils::serialize_dashmap")]
    #[serde(deserialize_with = "crate::utils::deserialize_dashmap")]
    pub hubs: HubMap,

    #[serde(flatten)]
    pub configs: RwLock<HubRouterPrimitiveConfigs>,

    #[serde(skip)]
    persist_file: PersistPath,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HubRouterPrimitiveConfigs {
    pub reaper_thread_interval: u64,
    pub reaper_thread_duration_max: u64,
    pub healthcheck_thread_interval: u64,
    pub bind_port: u16,
    pub bind_ip: Ipv4Addr,
    pub api_bind_port: u16,
    pub api_bind_ip: Ipv4Addr,
}

impl Default for HubRouterPrimitiveConfigs {
    fn default() -> Self {
        HubRouterPrimitiveConfigs {
            reaper_thread_interval: 60,
            reaper_thread_duration_max: 30,
            healthcheck_thread_interval: 1,
            bind_port: 6543,
            bind_ip: Ipv4Addr::UNSPECIFIED,
            api_bind_port: 8080,
            api_bind_ip: Ipv4Addr::UNSPECIFIED,
        }
    }
}

impl HubRouterState {
    #[allow(unused)]
    pub fn new_from_disk(path: &str) -> Self {
        let state = match read_to_string(path) {
            Ok(data) => match serde_json::from_str::<HubRouterState>(&data) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Error deserializing state: {}, falling back to default", e);
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Could not config file: {} - falling back to default", e);
                Self::default()
            }
        };

        state.hubs.alter_all(|_, v| v.clone_from_meta());
        println!("Serialized hubs: {:#?}", state.hubs);
        warn!("Unable to fetch config from disk - falling back to default");
        state
    }

    #[allow(unused)]
    pub fn get_reaper_interval_secs(&self) -> Option<u64> {
        match self.configs.read() {
            Ok(v) => Some(v.reaper_thread_interval),
            Err(_) => None,
        }
    }

    #[allow(unused)]
    pub fn get_reaper_max_session_mins(&self) -> Option<u64> {
        match self.configs.read() {
            Ok(v) => Some(v.reaper_thread_duration_max),
            Err(_) => None,
        }
    }

    #[allow(unused)]
    pub fn get_healthcheck_interval_secs(&self) -> Option<u64> {
        match self.configs.read() {
            Ok(v) => Some(v.reaper_thread_interval),
            Err(_) => None,
        }
    }

    pub fn persist(&self) -> Result<(), String> {
        let serialized = match serde_json::to_string_pretty(self) {
            Ok(str) => str,
            Err(e) => return Err(format!("Error serializing state: {}", e.to_string())),
        };

        let mut config_file = match File::create::<String>(self.persist_file.clone().into()) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "Error opening config file for serialization: {}",
                    e
                ))
            }
        };

        let bytes = serialized.as_bytes();
        match config_file.write(bytes) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error writing serialized hubs {}", e);
                return Err(format!("Error writing serialized hubs {}", e));
            }
        };

        return Ok(());
    }
}
