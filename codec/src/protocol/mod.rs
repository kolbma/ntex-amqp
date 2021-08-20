use std::fmt;

use chrono::{DateTime, Utc};
use derive_more::From;
use ntex_bytes::{BufMut, ByteString, Bytes, BytesMut};
use uuid::Uuid;

use super::codec::{self, Decode, DecodeFormatted, Encode};
use crate::{error::AmqpParseError, message::Message, types::*, HashMap};

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub(crate) struct CompoundHeader {
    pub size: u32,
    pub count: u32,
}

impl CompoundHeader {
    pub fn empty() -> CompoundHeader {
        CompoundHeader { size: 0, count: 0 }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ProtocolId {
    Amqp = 0,
    AmqpTls = 2,
    AmqpSasl = 3,
}

pub type Map = HashMap<Variant, Variant>;
pub type StringVariantMap = HashMap<Str, Variant>;
pub type Fields = HashMap<Symbol, Variant>;
pub type FilterSet = HashMap<Symbol, Option<ByteString>>;
pub type FieldsVec = VecSymbolMap;
pub type Timestamp = DateTime<Utc>;
pub type Symbols = Multiple<Symbol>;
pub type IetfLanguageTags = Multiple<IetfLanguageTag>;
pub type Annotations = HashMap<Symbol, Variant>;

#[allow(
    clippy::unreadable_literal,
    clippy::match_bool,
    clippy::large_enum_variant
)]
#[cfg(not(tarpaulin_include))]
mod definitions;
#[cfg(not(tarpaulin_include))]
pub use self::definitions::*;

#[derive(Debug, Eq, PartialEq, Clone, From, Display)]
pub enum MessageId {
    #[display(fmt = "{}", _0)]
    Ulong(u64),
    #[display(fmt = "{}", _0)]
    Uuid(Uuid),
    #[display(fmt = "{:?}", _0)]
    Binary(Bytes),
    #[display(fmt = "{}", _0)]
    String(ByteString),
}

impl From<usize> for MessageId {
    fn from(id: usize) -> MessageId {
        MessageId::Ulong(id as u64)
    }
}

impl From<i32> for MessageId {
    fn from(id: i32) -> MessageId {
        MessageId::Ulong(id as u64)
    }
}

impl DecodeFormatted for MessageId {
    fn decode_with_format(input: &[u8], fmt: u8) -> Result<(&[u8], Self), AmqpParseError> {
        match fmt {
            codec::FORMATCODE_SMALLULONG | codec::FORMATCODE_ULONG | codec::FORMATCODE_ULONG_0 => {
                u64::decode_with_format(input, fmt).map(|(i, o)| (i, MessageId::Ulong(o)))
            }
            codec::FORMATCODE_UUID => {
                Uuid::decode_with_format(input, fmt).map(|(i, o)| (i, MessageId::Uuid(o)))
            }
            codec::FORMATCODE_BINARY8 | codec::FORMATCODE_BINARY32 => {
                Bytes::decode_with_format(input, fmt).map(|(i, o)| (i, MessageId::Binary(o)))
            }
            codec::FORMATCODE_STRING8 | codec::FORMATCODE_STRING32 => {
                ByteString::decode_with_format(input, fmt).map(|(i, o)| (i, MessageId::String(o)))
            }
            _ => Err(AmqpParseError::InvalidFormatCode(fmt)),
        }
    }
}

impl Encode for MessageId {
    fn encoded_size(&self) -> usize {
        match *self {
            MessageId::Ulong(v) => v.encoded_size(),
            MessageId::Uuid(ref v) => v.encoded_size(),
            MessageId::Binary(ref v) => v.encoded_size(),
            MessageId::String(ref v) => v.encoded_size(),
        }
    }

    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            MessageId::Ulong(v) => v.encode(buf),
            MessageId::Uuid(ref v) => v.encode(buf),
            MessageId::Binary(ref v) => v.encode(buf),
            MessageId::String(ref v) => v.encode(buf),
        }
    }
}

#[derive(Clone, Debug, PartialEq, From)]
pub enum ErrorCondition {
    AmqpError(AmqpError),
    ConnectionError(ConnectionError),
    SessionError(SessionError),
    LinkError(LinkError),
    Custom(Symbol),
}

impl Default for ErrorCondition {
    fn default() -> ErrorCondition {
        ErrorCondition::Custom(Symbol(Str::from("Unknown")))
    }
}

impl DecodeFormatted for ErrorCondition {
    #[inline]
    fn decode_with_format(input: &[u8], format: u8) -> Result<(&[u8], Self), AmqpParseError> {
        let (input, result) = Symbol::decode_with_format(input, format)?;
        if let Ok(r) = AmqpError::try_from(&result) {
            return Ok((input, ErrorCondition::AmqpError(r)));
        }
        if let Ok(r) = ConnectionError::try_from(&result) {
            return Ok((input, ErrorCondition::ConnectionError(r)));
        }
        if let Ok(r) = SessionError::try_from(&result) {
            return Ok((input, ErrorCondition::SessionError(r)));
        }
        if let Ok(r) = LinkError::try_from(&result) {
            return Ok((input, ErrorCondition::LinkError(r)));
        }
        Ok((input, ErrorCondition::Custom(result)))
    }
}

impl Encode for ErrorCondition {
    fn encoded_size(&self) -> usize {
        match *self {
            ErrorCondition::AmqpError(ref v) => v.encoded_size(),
            ErrorCondition::ConnectionError(ref v) => v.encoded_size(),
            ErrorCondition::SessionError(ref v) => v.encoded_size(),
            ErrorCondition::LinkError(ref v) => v.encoded_size(),
            ErrorCondition::Custom(ref v) => v.encoded_size(),
        }
    }

    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            ErrorCondition::AmqpError(ref v) => v.encode(buf),
            ErrorCondition::ConnectionError(ref v) => v.encode(buf),
            ErrorCondition::SessionError(ref v) => v.encode(buf),
            ErrorCondition::LinkError(ref v) => v.encode(buf),
            ErrorCondition::Custom(ref v) => v.encode(buf),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DistributionMode {
    Move,
    Copy,
    Custom(Symbol),
}

impl DecodeFormatted for DistributionMode {
    fn decode_with_format(input: &[u8], format: u8) -> Result<(&[u8], Self), AmqpParseError> {
        let (input, result) = Symbol::decode_with_format(input, format)?;
        let result = match result.as_str() {
            "move" => DistributionMode::Move,
            "copy" => DistributionMode::Copy,
            _ => DistributionMode::Custom(result),
        };
        Ok((input, result))
    }
}

impl Encode for DistributionMode {
    fn encoded_size(&self) -> usize {
        match *self {
            DistributionMode::Move => 6,
            DistributionMode::Copy => 6,
            DistributionMode::Custom(ref v) => v.encoded_size(),
        }
    }

    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            DistributionMode::Move => Symbol::from("move").encode(buf),
            DistributionMode::Copy => Symbol::from("copy").encode(buf),
            DistributionMode::Custom(ref v) => v.encode(buf),
        }
    }
}

impl SaslInit {
    pub fn prepare_response(authz_id: &str, authn_id: &str, password: &str) -> Bytes {
        Bytes::from(format!("{}\x00{}\x00{}", authz_id, authn_id, password))
    }
}

impl Default for Properties {
    fn default() -> Properties {
        Properties {
            message_id: None,
            user_id: None,
            to: None,
            subject: None,
            reply_to: None,
            correlation_id: None,
            content_type: None,
            content_encoding: None,
            absolute_expiry_time: None,
            creation_time: None,
            group_id: None,
            group_sequence: None,
            reply_to_group_id: None,
        }
    }
}

#[derive(Debug, Clone, From, PartialEq)]
pub enum TransferBody {
    Data(Bytes),
    Message(Message),
}

impl TransferBody {
    #[inline]
    pub fn len(&self) -> usize {
        self.encoded_size()
    }

    #[inline]
    pub fn message_format(&self) -> Option<MessageFormat> {
        match self {
            TransferBody::Data(_) => None,
            TransferBody::Message(ref data) => data.0.message_format,
        }
    }
}

impl Encode for TransferBody {
    #[inline]
    fn encoded_size(&self) -> usize {
        match self {
            TransferBody::Data(ref data) => data.len(),
            TransferBody::Message(ref data) => data.encoded_size(),
        }
    }
    #[inline]
    fn encode(&self, dst: &mut BytesMut) {
        match *self {
            TransferBody::Data(ref data) => dst.put_slice(data),
            TransferBody::Message(ref data) => data.encode(dst),
        }
    }
}

impl Transfer {
    #[inline]
    pub fn get_body(&self) -> Option<&Bytes> {
        match self.body() {
            Some(TransferBody::Data(ref b)) => Some(b),
            _ => None,
        }
    }

    #[inline]
    pub fn load_message<T: Decode>(&self) -> Result<T, AmqpParseError> {
        if let Some(TransferBody::Data(ref b)) = self.body() {
            Ok(T::decode(b)?.1)
        } else {
            Err(AmqpParseError::UnexpectedType("body"))
        }
    }
}

impl Default for Role {
    fn default() -> Role {
        Role::Sender
    }
}

impl Default for SenderSettleMode {
    fn default() -> SenderSettleMode {
        SenderSettleMode::Mixed
    }
}

impl Default for ReceiverSettleMode {
    fn default() -> ReceiverSettleMode {
        ReceiverSettleMode::First
    }
}
