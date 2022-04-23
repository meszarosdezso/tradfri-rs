use std::str::FromStr;
use std::{fmt::Debug, process::Command};

use anyhow::{Error, Result};
use serde_json::json;

pub enum Method {
    GET,
    POST,
    PUT,
}

pub(crate) enum CoapResponse {
    Success(serde_json::Value),
    Error(String),
}

impl CoapResponse {
    pub fn data(self) -> serde_json::Value {
        match self {
            CoapResponse::Error { .. } => json!({}),
            CoapResponse::Success(data) => data,
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            CoapResponse::Error { .. } => false,
            _ => true,
        }
    }
}

impl Debug for CoapResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(value) => f.debug_tuple("Success").field(value).finish(),
            Self::Error(message) => f.debug_tuple("Error").field(message).finish(),
        }
    }
}

impl Into<Error> for CoapResponse {
    fn into(self) -> Error {
        match self {
            CoapResponse::Error(message) => Error::msg(message),
            _ => panic!("Response was successful."),
        }
    }
}

pub(crate) struct CoapClient {}

impl CoapClient {
    pub fn request(endpoint: &str, options: RequestOptions) -> Result<CoapResponse> {
        let mut cmd = Command::new("coap-client");
        for (key, value) in options.into_iter() {
            cmd.args([&format!("-{key}"), &value]);
        }

        cmd.arg(endpoint);

        let output = cmd.output()?;

        let (error, success) = (output.stderr, output.stdout);

        if success.len() > 0 {
            let success = String::from_utf8(success)?;
            let success = serde_json::Value::from_str(&success)?;
            Ok(CoapResponse::Success(success))
        } else {
            let error = String::from_utf8(error)?;
            let parts = error.split("\n").collect::<Vec<&str>>();

            if parts.len() > 2 {
                let status_code = parts.iter().nth(1).unwrap_or(&"4.00").replace(".", "");

                let message = String::from(match status_code.as_str() {
                    "4.00" => "Bad request.",
                    _ => "Bad request.",
                });

                Ok(CoapResponse::Error(message))
            } else {
                Ok(CoapResponse::Success(json!({})))
            }
        }
    }
}

pub struct RequestOptions {
    method: Option<String>,
    user: Option<String>,
    key: Option<String>,
    payload: Option<String>,
}

impl RequestOptions {
    pub fn new(method: &str, user: &str, key: &str, payload: &str) -> Self {
        Self {
            method: Some(method.to_string()),
            user: Some(user.to_string()),
            key: Some(key.to_string()),
            payload: Some(payload.to_string()),
        }
    }

    pub fn build() -> Self {
        Self {
            method: None,
            user: None,
            key: None,
            payload: None,
        }
    }

    pub fn user<S: ToString>(self, user: S) -> Self {
        Self {
            user: Some(user.to_string()),
            ..self
        }
    }

    pub fn key<S: ToString>(self, key: S) -> Self {
        Self {
            key: Some(key.to_string()),
            ..self
        }
    }

    pub fn method(self, method: Method) -> Self {
        let method = match method {
            Method::GET => "get",
            Method::POST => "post",
            Method::PUT => "put",
        };

        Self {
            method: Some(method.to_string()),
            ..self
        }
    }

    pub fn payload<S: ToString>(self, payload: S) -> Self {
        Self {
            payload: Some(payload.to_string()),
            ..self
        }
    }
}

impl Iterator for RequestOptions {
    type Item = (char, String);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(method) = self.method.clone() {
            self.method = None;
            return Some(('m', method));
        }

        if let Some(user) = self.user.clone() {
            self.user = None;
            return Some(('u', user));
        }

        if let Some(key) = self.key.clone() {
            self.key = None;
            return Some(('k', key));
        }

        if let Some(payload) = self.payload.clone() {
            self.payload = None;
            return Some(('e', payload));
        }

        None
    }
}
