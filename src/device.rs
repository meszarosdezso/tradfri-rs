use std::sync::Arc;

use anyhow::Result;
use serde::{de, Deserialize, Serialize};
use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::gateway::Gateway;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulbTemperature {
    #[serde(rename = "f5faf6")]
    White,
    #[serde(rename = "f1e0b5")]
    Warm,
    #[serde(rename = "efd275")]
    Glow,
}

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DeviceState {
    On = 1,
    Off = 0,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulbData {
    #[serde(rename = "5850")]
    pub status: DeviceState,
    #[serde(rename = "5706")]
    pub temperature: BulbTemperature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device<'a> {
    #[serde(rename = "9003")]
    pub id: u64,
    #[serde(rename = "9001")]
    pub name: String,
    #[serde(deserialize_with = "flatten_bulb_data")]
    #[serde(rename = "3311")]
    pub data: BulbData,
    #[serde(skip)]
    gateway: Option<Arc<&'a Gateway>>,
}

impl<'a> Device<'a> {
    pub(crate) fn set_gateway(&mut self, gateway: Arc<&'a Gateway>) {
        self.gateway = Some(Arc::clone(&gateway))
    }

    pub fn turn_on(&self) -> Result<()> {
        self.modify(BulbData {
            status: DeviceState::On,
            ..self.data.clone()
        })
    }

    pub fn turn_off(&self) -> Result<()> {
        self.modify(BulbData {
            status: DeviceState::Off,
            ..self.data.clone()
        })
    }

    fn modify(&self, data: BulbData) -> Result<()> {
        let gateway = self.gateway.clone().unwrap();
        let gateway = gateway.as_ref();

        let res = gateway.set_device_state(self.id, json!({ "3311": [data] }));

        res
    }
}

fn flatten_bulb_data<'de, D>(deserializer: D) -> Result<BulbData, D::Error>
where
    D: de::Deserializer<'de>,
{
    let vec: Vec<serde_json::Value> = de::Deserialize::deserialize(deserializer)?;
    let value = vec.into_iter().nth(0).unwrap();
    serde_json::from_value(value).map_err(de::Error::custom)
}
