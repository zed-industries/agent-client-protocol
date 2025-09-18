use schemars::JsonSchema;
use serde::Serialize;

pub const V0: ProtocolVersion = ProtocolVersion(0);
pub const V1: ProtocolVersion = ProtocolVersion(1);
pub const VERSION: ProtocolVersion = V1;

/// Protocol version identifier.
///
/// This version is only bumped for breaking changes.
/// Non-breaking changes should be introduced via capabilities.
#[derive(Default, Debug, Clone, Serialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion(u16);

impl ProtocolVersion {
    #[cfg(test)]
    #[must_use]
    pub const fn new(version: u16) -> Self {
        Self(version)
    }
}

use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct ProtocolVersionVisitor;

        impl Visitor<'_> for ProtocolVersionVisitor {
            type Value = ProtocolVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a protocol version number or string")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match u16::try_from(value) {
                    Ok(value) => Ok(ProtocolVersion(value)),
                    Err(_) => Err(E::custom(format!("protocol version {value} is too large"))),
                }
            }

            fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Old versions used strings, we consider all of those version 0
                Ok(ProtocolVersion(0))
            }

            fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Old versions used strings, we consider all of those version 0
                Ok(ProtocolVersion(0))
            }
        }

        deserializer.deserialize_any(ProtocolVersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_u64() {
        let json = "1";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(1));
    }

    #[test]
    fn test_deserialize_string() {
        let json = "\"1.0.0\"";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(0));
    }

    #[test]
    fn test_deserialize_large_number() {
        let json = "100000";
        let result: Result<ProtocolVersion, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_zero() {
        let json = "0";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(0));
    }

    #[test]
    fn test_deserialize_max_u16() {
        let json = "65535";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(65535));
    }
}
