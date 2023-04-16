use crate::{HubMap};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, io, sync::RwLock};

/// HubRouterState is an encapsulation of all configurable state within a
/// Hub Router instance. the notable exception to this is the session state
/// table, as that is configured by the system at runtime dynamically and
/// not persisted to disk.
///
/// The most notable data within this structure are the list of registered
/// Hubs to route traffic to (this includes runtime state including the session
/// fullness, but this data is not writeable via the API and not persisted to
/// disk). It also includes values for
#[derive(Serialize, Deserialize, Debug)]
pub struct HubRouterState {
    #[serde(serialize_with = "crate::utils::serialize_dashmap")]
    #[serde(deserialize_with = "crate::utils::deserialize_dashmap")]
    pub hubs: HubMap,

    #[serde(flatten)]
    pub configs: RwLock<HubRouterPrimitiveConfigs>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HubRouterPrimitiveConfigs {
    reaper_thread_interval: u32,
    reaper_thread_duration_max: u32,
    healthcheck_thread_interval: u32,
    bind_port: u16,
    api_bind_port: u16,
}

impl HubRouterState {
    #[allow(unused)]
    pub fn new_from_disk(path: &str) -> Result<Self, io::Error> {
        if let Ok(data) = read_to_string(path) {
            if let Ok(s) = serde_yaml::from_str(&data) {
                return Ok(s);
            }
        }

        // let config = Config::builder().add_source(File::with_name(path));

        Ok(Self {
            hubs: DashMap::new(),
            configs: RwLock::new(HubRouterPrimitiveConfigs {
                reaper_thread_interval: 0,
                reaper_thread_duration_max: 0,
                healthcheck_thread_interval: 0,
                bind_port: 8080,
                api_bind_port: 8081,
            }),
        })
    }

    // pub fn get_hubs(&self) -> Arc<HubMap> {
    //     Arc::new(self.hubs)
    // }

    #[allow(unused)]
    pub fn get_reaper_interval_secs(&self) -> Option<u32> {
        match self.configs.read() {
            Ok(v) => Some(v.reaper_thread_interval),
            Err(_) => None,
        }
    }

    #[allow(unused)]
    pub fn get_reaper_max_session_mins(&self) -> Option<u32> {
        match self.configs.read() {
            Ok(v) => Some(v.reaper_thread_duration_max),
            Err(_) => None,
        }
    }

    #[allow(unused)]
    pub fn get_healthcheck_interval_secs(&self) -> Option<u32> {
        match self.configs.read() {
            Ok(v) => Some(v.reaper_thread_interval),
            Err(_) => None,
        }
    }
}
