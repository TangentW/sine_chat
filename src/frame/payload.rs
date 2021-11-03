use std::fmt::Debug;

use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

use super::{Error, Result};

#[derive(Debug)]
pub struct RawPayload {
    pub type_code: u8,
    pub content: Bytes,
}

impl RawPayload {
    pub fn new(type_code: u8, content: Bytes) -> Self {
        Self { type_code, content }
    }

    pub fn from_payload(payload: impl SendablePayload) -> Result<Self> {
        payload.as_raw()
    }

    pub fn into_payload<T>(&self) -> Result<T>
    where
        T: ReceivablePayload,
    {
        T::from_raw(&self)
    }
}

pub trait ReceivablePayload: Sized {
    fn from_raw(raw: &RawPayload) -> Result<Self>;
}

pub trait SendablePayload: Send {
    fn as_raw(&self) -> Result<RawPayload>;
}

impl SendablePayload for Box<dyn SendablePayload> {
    fn as_raw(&self) -> Result<RawPayload> {
        self.as_ref().as_raw()
    }
}

// JSON

pub trait ReceivableJSONPayload: DeserializeOwned + Debug {
    fn type_code() -> u8;
}

impl<T> ReceivablePayload for T
where
    T: ReceivableJSONPayload,
{
    fn from_raw(raw: &RawPayload) -> Result<Self> {
        if Self::type_code() == raw.type_code {
            serde_json::from_slice(&raw.content).map_err(|e| e.into())
        } else {
            Err(Error::TypeMismatch(raw.type_code))
        }
    }
}

pub trait SendableJSONPayload: Serialize + Send {
    fn type_code(&self) -> u8;
}

impl<T> SendablePayload for T
where
    T: SendableJSONPayload,
{
    fn as_raw(&self) -> Result<RawPayload> {
        serde_json::to_vec(self)
            .map(|content| RawPayload::new(self.type_code(), content.into()))
            .map_err(|e| e.into())
    }
}
