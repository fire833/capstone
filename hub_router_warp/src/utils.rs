use dashmap::DashMap;
use serde::{de::Visitor, ser::SerializeSeq, Deserializer, Serializer};
use url::Url;

use crate::{hub::Hub, HubMap};

struct UuidVisitor;

impl<'de> Visitor<'de> for UuidVisitor {
    type Value = uuid::Uuid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid uuid")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match uuid::Builder::from_slice(v.as_bytes()) {
            Ok(uuid) => Ok(uuid.into_uuid()),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

pub fn serialize_uuid<S>(uuid: &uuid::Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

pub fn deserialize_uuid<'de, D>(deserializer: D) -> Result<uuid::Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(UuidVisitor)
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = url::Url;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid url")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match url::Url::parse(v.as_str()) {
            Ok(url) => Ok(url),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

pub fn serialize_url<S>(url: &url::Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&url.to_string())
}

pub fn deserialize_url<'de, D>(deserializer: D) -> Result<url::Url, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(UrlVisitor)
}

struct DashMapVisitor;

impl<'de> Visitor<'de> for DashMapVisitor {
    type Value = HubMap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid map")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut map = DashMap::new();

        while let Some(hub) = seq.next_element::<Hub>()? {
            map.insert(hub.meta.uuid.clone(), hub);
        }

        Ok(map)
    }
}

pub fn serialize_dashmap<S>(map: &HubMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(map.len()))?;

    map.iter().for_each(|h| {
        seq.serialize_element(h.value());
    });

    seq.end()
}

pub fn deserialize_dashmap<'de, D>(deserializer: D) -> Result<HubMap, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(DashMapVisitor)
}