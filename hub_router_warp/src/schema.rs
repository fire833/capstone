use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[allow(non_snake_case, unused)]
pub struct NewSessionResponse {
    pub value: NewSessionResponseValue,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case, unused)]
pub struct NewSessionResponseValue {
    pub sessionId: String,
    pub capabilities: Option<NewSessionResponseCapabilities>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case, unused)]
pub struct NewSessionResponseCapabilities {
    pub acceptInsecureCerts: Option<bool>,
    pub browserName: Option<String>,
    pub browserVersion: Option<String>,
    pub platformName: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HubStatusJSONSchema {
    pub value: HubStatusValueJSONSchema,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HubStatusValueJSONSchema {
    pub ready: bool,
    pub message: String,
    pub nodes: Vec<HubStatusNodeJSONSchema>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct HubStatusOSInfoJSONSchema {
    pub arch: String,
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct HubStatusNodeSlotJSONSchema {
    pub id: HubStatusNodeSlotIDJSONSchema,
    pub lastStarted: String,
    pub session: Option<HubStatusNodeSlotSessionJSONSchema>,
    pub stereotype: HubStatusStereotypeJSONSchema,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]

pub struct HubStatusNodeSlotIDJSONSchema {
    pub hostId: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]

pub struct HubStatusNodeSlotSessionJSONSchema {
    pub capabilities: Option<HubStatusNodeSlotSessionCapabilitiesJSONSchema>,
    pub sessionId: String,
    pub start: String,
    pub stereotype: HubStatusStereotypeJSONSchema,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]

pub struct HubStatusNodeSlotSessionCapabilitiesJSONSchema {
    pub acceptInsecureCerts: Option<bool>,
    pub browserName: Option<String>,
    pub browserVersion: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]

pub struct HubStatusStereotypeJSONSchema {
    pub browserName: String,
    pub platformName: String,
}
