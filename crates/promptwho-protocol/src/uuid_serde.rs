use serde::{
    Deserializer,
    de::{self, SeqAccess, Visitor},
};
use std::fmt;
use uuid::Uuid;

pub fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    struct UuidVisitor;

    impl<'de> Visitor<'de> for UuidVisitor {
        type Value = Uuid;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a UUID string or 16-byte UUID value")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Uuid::parse_str(value).map_err(E::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }

        /// When MsgPack encodes a UUID as a 16-byte binary value, it will be deserialized as a byte
        /// slice or byte buffer.
        fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Uuid::from_slice(value).map_err(E::custom)
        }

        /// When MsgPack encodes a UUID as a 16-byte binary value, it will be deserialized as a byte
        /// slice or byte buffer.
        fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_bytes(&value)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut bytes = [0u8; 16];

            for (index, byte) in bytes.iter_mut().enumerate() {
                *byte = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(index, &self))?;
            }

            if seq.next_element::<u8>()?.is_some() {
                return Err(de::Error::invalid_length(17, &self));
            }

            Ok(Uuid::from_bytes(bytes))
        }
    }

    deserializer.deserialize_any(UuidVisitor)
}
