use serde::{de::DeserializeOwned, Serialize};
use synchole_core::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecId {
    Bincode,
    Postcard,
    Rkyv,
    Custom(String),
}

impl CodecId {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bincode => "bincode",
            Self::Postcard => "postcard",
            Self::Rkyv => "rkyv",
            Self::Custom(value) => value.as_str(),
        }
    }
}

pub trait BinaryCodec: Send + Sync {
    fn id(&self) -> CodecId;
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;
    fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;
}
