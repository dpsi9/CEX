use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::CexError;

pub fn to_json<T: Serialize>(v: &T) -> Result<String, CexError> {
    serde_json::to_string(v).map_err(CexError::from)
}

pub fn from_json<T: DeserializeOwned>(s: &str) -> Result<T, CexError> {
    serde_json::from_str(s).map_err(CexError::from)
}
