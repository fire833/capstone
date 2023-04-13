use std::{fs::read_to_string, io, sync::Mutex};

use config::{Config, File};
use dashmap::DashMap;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::{conf, hub::Hub, hub::HubExternal};
use url::Url;

/// HubRouterState is an encapsulation of all configurable state within a
/// Hub Router instance. the notable exception to this is the session state
/// table, as that is configured by the system at runtime dynamically and
/// not persisted to disk.
///
/// The most notable data within this structure are the list of registered
/// Hubs to route traffic to (this includes runtime state including the session
/// fullness, but this data is not writeable via the API and not persisted to
/// disk). It also includes values for
pub struct HubRouterState {
    hubs: DashMap<Url, Hub>,

    // internal mapping for API callers to get Hub by Url.
    id_to_url: DashMap<String, Url>,

    reaper_thread_interval: u32,
    reaper_thread_duration_max: u32,
    healthcheck_thread_interval: u32,
    bind_port: u16,
    api_bind_port: u16,
}

impl Serialize for HubRouterState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.reaper_thread_interval as i32)?;
        serializer.serialize_i32(self.reaper_thread_duration_max as i32)?;
        serializer.serialize_i32(self.healthcheck_thread_interval as i32)?;
        serializer.serialize_i16(self.bind_port as i16)?;
        serializer.serialize_i16(self.api_bind_port as i16)?;

        let mut map = serializer.serialize_map(Some(self.hubs.len()))?;
        for (k, v) in self.hubs {
            map.serialize_entry(&v.name, &v)?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for HubRouterState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
    }
}

impl HubRouterState {
    pub fn new_from_disk(path: &str) -> Result<Self, io::Error> {
        if let Ok(data) = read_to_string(path) {
            if let Ok(s) = serde_yaml::from_str(&data) {
                return Ok(s);
            }
        }

        // let config = Config::builder().add_source(File::with_name(path));

        Ok(Self {
            hubs: DashMap::new(),
            id_to_url: DashMap::new(),
            reaper_thread_interval: 0,
            reaper_thread_duration_max: 0,
            healthcheck_thread_interval: 0,
            bind_port: 6543,
            api_bind_port: 8080,
        })
    }

    pub fn get_reaper_interval_secs(&self) -> u32 {
        self.reaper_thread_interval
    }

    pub fn get_reaper_max_session_mins(&self) -> u32 {
        self.reaper_thread_duration_max
    }

    pub fn get_healthcheck_interval_secs(&self) -> u32 {
        self.healthcheck_thread_interval
    }

    pub fn get_hub_by_name(&self, id: String) -> Option<Hub> {
        if let Some(url) = self.id_to_url.get(&id) {
            if let Some(hub) = self.hubs.get(&url) {
                Some(hub.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn remove_hub_by_name(&self, id: String) {
        if let Some(url) = self.id_to_url.get(&id) {
            self.hubs.remove(&url);
            self.id_to_url.remove(&id);
        }
    }
}
