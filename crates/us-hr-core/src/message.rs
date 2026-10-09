//! Codec for the compact typed-field format used inside USB control transfers.

use thiserror::Error;

/// Maximum control-message size accepted by the device firmware.
pub const MAX_MESSAGE_LEN: usize = 64;

const TYPE_END: u8 = 0;
const TYPE_QUERY: u8 = 1;
const TYPE_U8: u8 = 2;
const TYPE_U16: u8 = 3;
const TYPE_U32: u8 = 4;
const TYPE_BYTES: u8 = 5;

/// One typed field in a device message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Protocol-specific field identifier.
    pub id: u8,
    /// Field payload.
    pub value: FieldValue,
}

/// Supported field payload types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    /// A request for the device to return this field.
    Query,
    /// An unsigned byte.
    U8(u8),
    /// A little-endian unsigned 16-bit integer.
    U16(u16),
    /// A little-endian unsigned 32-bit integer.
    U32(u32),
    /// A length-prefixed byte string.
    Bytes(Vec<u8>),
}

/// A complete vendor-control message.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Message {
    fields: Vec<Field>,
}

impl Message {
    /// Creates an empty message.
    #[must_use]
    pub const fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Returns the decoded fields.
    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// Appends one field after checking the 64-byte device limit.
    ///
    /// # Errors
    ///
    /// Returns [`MessageError::TooLong`] if the encoded message would exceed
    /// the device's control-transfer limit.
    pub fn push(&mut self, field: Field) -> Result<(), MessageError> {
        let new_len = self.encoded_len() + field.encoded_len();
        if new_len > MAX_MESSAGE_LEN {
            return Err(MessageError::TooLong(new_len));
        }
        self.fields.push(field);
        Ok(())
    }

    /// Encodes this message for a USB control transfer.
    ///
    /// # Errors
    ///
    /// Returns an error if a byte field cannot be represented by the protocol.
    pub fn encode(&self) -> Result<Vec<u8>, MessageError> {
        let mut output = Vec::with_capacity(self.encoded_len());
        for field in &self.fields {
            field.encode_into(&mut output)?;
        }
        output.extend_from_slice(&[0, TYPE_END]);
        Ok(output)
    }

    /// Decodes all fields in a received USB control transfer.
    ///
    /// # Errors
    ///
    /// Returns an error for oversized, truncated, unterminated, or unknown
    /// protocol data.
    pub fn decode(input: &[u8]) -> Result<Self, MessageError> {
        if input.len() > MAX_MESSAGE_LEN {
            return Err(MessageError::TooLong(input.len()));
        }

        let mut fields = Vec::new();
        let mut offset = 0;
        while offset < input.len() {
            if input.len() - offset < 2 {
                return Err(MessageError::Truncated { offset });
            }
            let id = input[offset];
            let kind = input[offset + 1];
            offset += 2;
            if id == 0 && kind == TYPE_END {
                return Ok(Self { fields });
            }
            let value = match kind {
                TYPE_QUERY => FieldValue::Query,
                TYPE_U8 => FieldValue::U8(take_array::<1>(input, &mut offset)?[0]),
                TYPE_U16 => FieldValue::U16(u16::from_le_bytes(take_array(input, &mut offset)?)),
                TYPE_U32 => FieldValue::U32(u32::from_le_bytes(take_array(input, &mut offset)?)),
                TYPE_BYTES => {
                    let length = usize::from(take_array::<1>(input, &mut offset)?[0]);
                    if input.len() - offset < length {
                        return Err(MessageError::Truncated { offset });
                    }
                    let bytes = input[offset..offset + length].to_vec();
                    offset += length;
                    FieldValue::Bytes(bytes)
                }
                TYPE_END => return Err(MessageError::InvalidTerminator { id }),
                other => return Err(MessageError::UnknownType(other)),
            };
            fields.push(Field { id, value });
        }
        Err(MessageError::MissingTerminator)
    }

    fn encoded_len(&self) -> usize {
        2 + self.fields.iter().map(Field::encoded_len).sum::<usize>()
    }
}

impl Field {
    fn encoded_len(&self) -> usize {
        2 + match &self.value {
            FieldValue::Query => 0,
            FieldValue::U8(_) => 1,
            FieldValue::U16(_) => 2,
            FieldValue::U32(_) => 4,
            FieldValue::Bytes(bytes) => 1 + bytes.len(),
        }
    }

    fn encode_into(&self, output: &mut Vec<u8>) -> Result<(), MessageError> {
        output.push(self.id);
        match &self.value {
            FieldValue::Query => output.push(TYPE_QUERY),
            FieldValue::U8(value) => {
                output.push(TYPE_U8);
                output.push(*value);
            }
            FieldValue::U16(value) => {
                output.push(TYPE_U16);
                output.extend_from_slice(&value.to_le_bytes());
            }
            FieldValue::U32(value) => {
                output.push(TYPE_U32);
                output.extend_from_slice(&value.to_le_bytes());
            }
            FieldValue::Bytes(bytes) => {
                let length = u8::try_from(bytes.len())
                    .map_err(|_| MessageError::ByteFieldTooLong(bytes.len()))?;
                output.push(TYPE_BYTES);
                output.push(length);
                output.extend_from_slice(bytes);
            }
        }
        Ok(())
    }
}

fn take_array<const N: usize>(input: &[u8], offset: &mut usize) -> Result<[u8; N], MessageError> {
    let end = offset.saturating_add(N);
    let slice = input
        .get(*offset..end)
        .ok_or(MessageError::Truncated { offset: *offset })?;
    *offset = end;
    slice
        .try_into()
        .map_err(|_| MessageError::Truncated { offset: *offset })
}

/// Message encoding or decoding failure.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MessageError {
    /// The total message exceeds the device's 64-byte limit.
    #[error("message is {0} bytes; the device limit is 64")]
    TooLong(usize),
    /// A byte-string field cannot represent its length in one byte.
    #[error("byte field is too long: {0} bytes")]
    ByteFieldTooLong(usize),
    /// The input ended partway through a field.
    #[error("truncated message at byte offset {offset}")]
    Truncated {
        /// Byte offset where decoding failed.
        offset: usize,
    },
    /// The field type has not been observed or defined.
    #[error("unknown field type {0:#04x}")]
    UnknownType(u8),
    /// Type zero is reserved for the field-zero message terminator.
    #[error("invalid terminator field id {id:#04x}")]
    InvalidTerminator {
        /// Non-zero field identifier paired with type zero.
        id: u8,
    },
    /// A received message did not contain its required zero terminator.
    #[error("message has no terminator")]
    MissingTerminator,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn encodes_known_values_little_endian() -> Result<(), MessageError> {
        let mut message = Message::new();
        message.push(Field {
            id: 0x10,
            value: FieldValue::U16(0x1234),
        })?;
        message.push(Field {
            id: 0x11,
            value: FieldValue::U8(0x7f),
        })?;
        assert_eq!(
            message.encode()?,
            [0x10, TYPE_U16, 0x34, 0x12, 0x11, TYPE_U8, 0x7f, 0, TYPE_END]
        );
        Ok(())
    }

    #[test]
    fn rejects_truncated_fields() {
        assert_eq!(
            Message::decode(&[0x10]),
            Err(MessageError::Truncated { offset: 0 })
        );
        assert_eq!(
            Message::decode(&[0x10, TYPE_U16, 0x34]),
            Err(MessageError::Truncated { offset: 2 })
        );
    }

    #[test]
    fn decodes_padded_device_response() -> Result<(), MessageError> {
        let response = [0x15, TYPE_U8, 1, 0, TYPE_END, 0, 0, 0];
        assert_eq!(
            Message::decode(&response)?,
            Message {
                fields: vec![Field {
                    id: 0x15,
                    value: FieldValue::U8(1),
                }],
            }
        );
        Ok(())
    }

    #[test]
    fn query_has_no_payload() -> Result<(), MessageError> {
        let mut message = Message::new();
        message.push(Field {
            id: 0x14,
            value: FieldValue::Query,
        })?;
        assert_eq!(message.encode()?, [0x14, TYPE_QUERY, 0, TYPE_END]);
        Ok(())
    }

    #[test]
    fn round_trips_all_scalar_field_types() -> Result<(), MessageError> {
        let mut message = Message::new();
        for field in [
            Field {
                id: 1,
                value: FieldValue::Query,
            },
            Field {
                id: 2,
                value: FieldValue::U8(u8::MAX),
            },
            Field {
                id: 3,
                value: FieldValue::U16(u16::MAX),
            },
            Field {
                id: 4,
                value: FieldValue::U32(u32::MAX),
            },
        ] {
            message.push(field)?;
        }

        assert_eq!(Message::decode(&message.encode()?)?, message);
        Ok(())
    }

    #[test]
    fn enforces_message_limit_without_mutating_on_failure() -> Result<(), MessageError> {
        let mut message = Message::new();
        message.push(Field {
            id: 0x42,
            value: FieldValue::Bytes(vec![0; 59]),
        })?;
        assert_eq!(message.encode()?.len(), MAX_MESSAGE_LEN);

        let mut rejected = Message::new();
        assert_eq!(
            rejected.push(Field {
                id: 0x42,
                value: FieldValue::Bytes(vec![0; 60]),
            }),
            Err(MessageError::TooLong(MAX_MESSAGE_LEN + 1))
        );
        assert_eq!(rejected.fields(), []);
        Ok(())
    }

    #[test]
    fn rejects_invalid_message_structure() {
        assert_eq!(
            Message::decode(&[1, TYPE_END]),
            Err(MessageError::InvalidTerminator { id: 1 })
        );
        assert_eq!(
            Message::decode(&[1, 0xff]),
            Err(MessageError::UnknownType(0xff))
        );
        assert_eq!(
            Message::decode(&[1, TYPE_U8, 7]),
            Err(MessageError::MissingTerminator)
        );
        assert_eq!(
            Message::decode(&[1, TYPE_BYTES, 2, 7]),
            Err(MessageError::Truncated { offset: 3 })
        );
        assert_eq!(
            Message::decode(&[0; MAX_MESSAGE_LEN + 1]),
            Err(MessageError::TooLong(MAX_MESSAGE_LEN + 1))
        );
    }

    proptest! {
        #[test]
        fn round_trips_byte_fields(bytes in proptest::collection::vec(any::<u8>(), 0..=59)) {
            let mut message = Message::new();
            let push_result = message.push(Field { id: 0x42, value: FieldValue::Bytes(bytes) });
            prop_assert!(push_result.is_ok());
            let encoded = message.encode().map_err(|error| TestCaseError::fail(error.to_string()))?;
            let decoded = Message::decode(&encoded)
                .map_err(|error| TestCaseError::fail(error.to_string()))?;
            prop_assert_eq!(decoded, message);
        }

        #[test]
        fn arbitrary_inputs_never_decode_past_the_message_limit(
            input in proptest::collection::vec(any::<u8>(), 0..=MAX_MESSAGE_LEN + 8)
        ) {
            let result = Message::decode(&input);
            if input.len() > MAX_MESSAGE_LEN {
                prop_assert_eq!(result, Err(MessageError::TooLong(input.len())));
            }
        }
    }
}
