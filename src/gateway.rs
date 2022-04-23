use anyhow::Result;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::coap::{CoapClient, Method, RequestOptions};
use crate::device::Device;
use crate::endpoints;

#[derive(Clone, Debug)]
pub struct Gateway {
    addr: Ipv4Addr,
    security_code: String,
    preshared_key: Option<String>,
    user: Option<String>,
}

impl Gateway {
    pub fn new(addr: Ipv4Addr, security_code: &str) -> Self {
        Self {
            addr,
            security_code: security_code.to_string(),
            preshared_key: None,
            user: None,
        }
    }

    pub fn authenticate(&mut self, user: &str) -> Result<()> {
        dotenv::dotenv().ok();

        if let Ok(key) = env::var("PRESHARED_KEY") {
            println!("Preshared key loaded from environment");
            self.preshared_key = Some(key);
            self.user = Some(user.to_string());
            return Ok(());
        }

        let options = RequestOptions::build()
            .method(Method::POST)
            .user("Client_identity")
            .key(&self.security_code)
            .payload(format!("{{\"9090\":\"{user}\"}}").as_str());

        let endpoint = format!("coaps://{}:5684/{}", self.addr, &endpoints::AUTHENTICATE);

        let response = CoapClient::request(&endpoint, options)?;

        if response.is_ok() {
            let data = response.data();
            let preshared_key = data.get("9091").unwrap();
            self.preshared_key = Some(preshared_key.to_string());
            self.user = Some(user.to_string());
            fs::write(".env", format!("PRESHARED_KEY={}", &preshared_key))?;
            println!("New preshared key saved: {preshared_key}");
            Ok(())
        } else {
            Err(response.into())
        }
    }

    pub fn get_device_by_id<'a>(&'a self, id: u64) -> Result<Device> {
        let options = RequestOptions::build()
            .method(Method::GET)
            .user(self.user.clone().unwrap())
            .key(&self.preshared_key.clone().unwrap());

        let endpoint = format!("coaps://{}:5684/{}/{}", self.addr, &endpoints::DEVICES, &id);

        let response = CoapClient::request(&endpoint, options)?;

        if response.is_ok() {
            let data = response.data();
            let mut device = serde_json::from_value::<Device>(data.to_owned())?;
            let arc = Arc::new(self);
            device.set_gateway(arc);
            Ok(device)
        } else {
            Err(response.into())
        }
    }

    pub fn get_device_ids(&self) -> Result<Vec<u64>> {
        let options = RequestOptions::build()
            .method(Method::GET)
            .user(self.user.clone().unwrap())
            .key(&self.preshared_key.clone().unwrap());

        let endpoint = format!("coaps://{}:5684/{}", self.addr, &endpoints::DEVICES);

        let response = CoapClient::request(&endpoint, options)?;

        if response.is_ok() {
            let data = response.data();
            let device_ids = data
                .as_array()
                .unwrap()
                .into_iter()
                .map(|v| v.as_u64())
                .flatten()
                .collect::<Vec<u64>>();

            Ok(device_ids)
        } else {
            Err(response.into())
        }
    }

    pub fn set_device_state(&self, device_id: u64, payload: serde_json::Value) -> Result<()> {
        let payload = serde_json::to_string(&payload)?;
        println!("Payload: {payload}");

        let options = RequestOptions::build()
            .method(Method::PUT)
            .user(self.user.clone().unwrap())
            .key(&self.preshared_key.clone().unwrap())
            .payload(payload);

        let endpoint = format!(
            "coaps://{}:5684/{}/{}",
            self.addr,
            &endpoints::DEVICES,
            device_id
        );

        let response = CoapClient::request(&endpoint, options)?;

        if response.is_ok() {
            Ok(())
        } else {
            Err(response.into())
        }
    }
}
