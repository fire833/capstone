//! Serde struct definitions for JSON schemae which we expect from various
//! API endpoints on the Selenium hubs

use serde::{Deserialize, Serialize};

use crate::routing::Endpoint;

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case, unused)]
pub struct NewSessionResponse {
    pub value: NewSessionResponseValue,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case, unused)]
pub struct NewSessionResponseValue {
    pub sessionId: String,
    pub capabilities: NewSessionResponseCapabilities,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case, unused)]
pub struct NewSessionResponseCapabilities {
    pub acceptInsecureCerts: Option<bool>,
    pub browserName: Option<String>,
    pub browserVersion: Option<String>,
    pub platformName: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HubStatusJSONSchema {
    pub value: HubStatusValueJSONSchema,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HubStatusValueJSONSchema {
    pub ready: bool,
    pub message: String,
    pub nodes: Vec<HubStatusNodeJSONSchema>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct HubStatusNodeJSONSchema {
    pub id: String,
    pub uri: String,
    pub maxSessions: u32,
    pub osInfo: HubStatusOSInfoJSONSchema,
    pub availability: String,
    pub version: String,
    pub slots: Vec<HubStatusNodeSlotJSONSchema>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HubStatusOSInfoJSONSchema {
    pub arch: String,
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct HubStatusNodeSlotJSONSchema {
    pub id: HubStatusNodeSlotIDJSONSchema,
    pub lastStarted: String,
    pub session: Option<HubStatusNodeSlotSessionJSONSchema>,
    pub stereotype: HubStatusStereotypeJSONSchema,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]

pub struct HubStatusNodeSlotIDJSONSchema {
    pub hostId: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]

pub struct HubStatusNodeSlotSessionJSONSchema {
    pub capabilities: Option<HubStatusNodeSlotSessionCapabilitiesJSONSchema>,
    pub sessionId: String,
    pub start: String,
    pub stereotype: HubStatusStereotypeJSONSchema,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]

pub struct HubStatusNodeSlotSessionCapabilitiesJSONSchema {
    pub acceptInsecureCerts: Option<bool>,
    pub browserName: Option<String>,
    pub browserVersion: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq)]
#[allow(non_snake_case)]

pub struct HubStatusStereotypeJSONSchema {
    pub browserName: String,
    pub platformName: String,
}

impl PartialEq for HubStatusStereotypeJSONSchema {
    fn eq(&self, other: &Self) -> bool {
        self.browserName.eq_ignore_ascii_case(&other.browserName)
            && self.platformName.eq_ignore_ascii_case(&other.platformName)
    }
}

impl Into<NewSessionRequestCapability> for HubStatusStereotypeJSONSchema {
    fn into(self) -> NewSessionRequestCapability {
        NewSessionRequestCapability {
            browserName: Some(self.browserName),
            platformName: Some(self.platformName),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_snake_case)]

pub struct NewSessionRequestBody {
    pub capabilities: NewSessionRequestCapabilities,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_snake_case)]

pub struct NewSessionRequestCapabilities {
    pub alwaysMatch: NewSessionRequestCapability,
    pub firstMatch: Vec<NewSessionRequestCapability>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[allow(non_snake_case)]

pub struct NewSessionRequestCapability {
    pub browserName: Option<String>,
    pub platformName: Option<String>,
}

impl NewSessionRequestCapability {
    pub fn satisfied_by(&self, other: &NewSessionRequestCapability) -> bool {
        if self.platformName.is_some() && self.platformName != other.platformName {
            return false;
        }

        if self.browserName.is_some() && self.browserName != other.browserName {
            return false;
        }

        return true;
    }
}

/// Session is the object for serializing internal session data for
/// consumption by the external API.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    id: String,

    #[serde(serialize_with = "crate::utils::serialize_url")]
    #[serde(deserialize_with = "crate::utils::deserialize_url")]
    endpoint: Endpoint,
}

impl Session {
    pub fn new(id: &String, endpoint: &Endpoint) -> Self {
        Self {
            id: id.clone(),
            endpoint: endpoint.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HubGraphQLQuery {
    pub query: String,
}
