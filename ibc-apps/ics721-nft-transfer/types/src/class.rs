//! Defines Non-Fungible Token Transfer (ICS-721) class types.
use core::fmt::{self, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::{Display, From};
use http::Uri;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_proto::ibc::applications::nft_transfer::v1::ClassTrace as RawClassTrace;

use crate::data::Data;
use crate::error::NftTransferError;
use crate::serializers;

/// Class ID for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassId(String);

impl AsRef<str> for ClassId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ClassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClassId {
    type Err = NftTransferError;

    fn from_str(class_id: &str) -> Result<Self, Self::Err> {
        if class_id.trim().is_empty() {
            Err(NftTransferError::EmptyBaseClassId)
        } else {
            Ok(Self(class_id.to_string()))
        }
    }
}

/// Class prefix, the same as ICS-20 TracePrefix
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TracePrefix {
    port_id: PortId,
    channel_id: ChannelId,
}

impl TracePrefix {
    pub fn new(port_id: PortId, channel_id: ChannelId) -> Self {
        Self {
            port_id,
            channel_id,
        }
    }
}

impl Display for TracePrefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}/{}", self.port_id, self.channel_id)
    }
}

/// Class trace path, the same as ICS-20 TracePath
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, From)]
pub struct TracePath(Vec<TracePrefix>);

impl TracePath {
    /// Returns true iff this path starts with the specified prefix
    pub fn starts_with(&self, prefix: &TracePrefix) -> bool {
        self.0.last().map(|p| p == prefix).unwrap_or(false)
    }

    /// Removes the specified prefix from the path if there is a match, otherwise does nothing.
    pub fn remove_prefix(&mut self, prefix: &TracePrefix) {
        if self.starts_with(prefix) {
            self.0.pop();
        }
    }

    /// Adds the specified prefix to the path.
    pub fn add_prefix(&mut self, prefix: TracePrefix) {
        self.0.push(prefix)
    }

    /// Returns true if the path is empty and false otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> TryFrom<Vec<&'a str>> for TracePath {
    type Error = NftTransferError;

    fn try_from(v: Vec<&'a str>) -> Result<Self, Self::Error> {
        if v.len() % 2 != 0 {
            return Err(NftTransferError::InvalidTraceLength {
                len: v.len() as u64,
            });
        }

        let mut trace = vec![];
        let id_pairs = v.chunks_exact(2).map(|paths| (paths[0], paths[1]));
        for (pos, (port_id, channel_id)) in id_pairs.rev().enumerate() {
            let port_id =
                PortId::from_str(port_id).map_err(|e| NftTransferError::InvalidTracePortId {
                    pos: pos as u64,
                    validation_error: e,
                })?;
            let channel_id = ChannelId::from_str(channel_id).map_err(|e| {
                NftTransferError::InvalidTraceChannelId {
                    pos: pos as u64,
                    validation_error: e,
                }
            })?;
            trace.push(TracePrefix {
                port_id,
                channel_id,
            });
        }

        Ok(trace.into())
    }
}

impl FromStr for TracePath {
    type Err = NftTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 1 && parts[0].trim().is_empty() {
                vec![]
            } else {
                parts
            }
        };
        parts.try_into()
    }
}

impl Display for TracePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let path = self
            .0
            .iter()
            .rev()
            .map(|prefix| prefix.to_string())
            .collect::<Vec<String>>()
            .join("/");
        write!(f, "{path}")
    }
}

/// Prefixed class to trace sources like ICS-20 PrefixedDenom
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct PrefixedClassId {
    /// A series of `{port-id}/{channel-id}`s for tracing the source of the class.
    #[cfg_attr(feature = "serde", serde(with = "serializers"))]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub trace_path: TracePath,
    /// Base class of the relayed non-fungible token.
    pub base_class_id: ClassId,
}

impl PrefixedClassId {
    /// Removes the specified prefix from the trace path if there is a match, otherwise does nothing.
    pub fn remove_trace_prefix(&mut self, prefix: &TracePrefix) {
        self.trace_path.remove_prefix(prefix)
    }

    /// Adds the specified prefix to the trace path.
    pub fn add_trace_prefix(&mut self, prefix: TracePrefix) {
        self.trace_path.add_prefix(prefix)
    }
}

/// Returns true if the class ID originally came from the sender chain and false otherwise.
pub fn is_sender_chain_source(
    source_port: PortId,
    source_channel: ChannelId,
    class_id: &PrefixedClassId,
) -> bool {
    !is_receiver_chain_source(source_port, source_channel, class_id)
}

/// Returns true if the class ID originally came from the receiving chain and false otherwise.
pub fn is_receiver_chain_source(
    source_port: PortId,
    source_channel: ChannelId,
    class_id: &PrefixedClassId,
) -> bool {
    // For example, let
    // A: sender chain in this transfer, port "transfer" and channel "c2b" (to B)
    // B: receiver chain in this transfer, port "transfer" and channel "c2a" (to A)
    //
    // If B had originally sent the token in a previous transfer, then A would have stored the token as
    // "transfer/c2b/{token_denom}". Now, A is sending to B, so to check if B is the source of the token,
    // we need to check if the token starts with "transfer/c2b".
    let prefix = TracePrefix::new(source_port, source_channel);
    class_id.trace_path.starts_with(&prefix)
}

impl FromStr for PrefixedClassId {
    type Err = NftTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts: Vec<&str> = s.split('/').collect();
        let last_part = parts.pop().expect("split() returned an empty iterator");

        let (base_class_id, trace_path) = {
            if last_part == s {
                (ClassId::from_str(s)?, TracePath::default())
            } else {
                let base_class_id = ClassId::from_str(last_part)?;
                let trace_path = TracePath::try_from(parts)?;
                (base_class_id, trace_path)
            }
        };

        Ok(Self {
            trace_path,
            base_class_id,
        })
    }
}

impl TryFrom<RawClassTrace> for PrefixedClassId {
    type Error = NftTransferError;

    fn try_from(value: RawClassTrace) -> Result<Self, Self::Error> {
        let base_class_id = ClassId::from_str(&value.base_class_id)?;
        let trace_path = TracePath::from_str(&value.path)?;
        Ok(Self {
            trace_path,
            base_class_id,
        })
    }
}

impl From<PrefixedClassId> for RawClassTrace {
    fn from(value: PrefixedClassId) -> Self {
        Self {
            path: value.trace_path.to_string(),
            base_class_id: value.base_class_id.to_string(),
        }
    }
}

impl From<ClassId> for PrefixedClassId {
    fn from(class_id: ClassId) -> Self {
        Self {
            trace_path: Default::default(),
            base_class_id: class_id,
        }
    }
}

impl Display for PrefixedClassId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        if self.trace_path.0.is_empty() {
            write!(f, "{}", self.base_class_id)
        } else {
            write!(f, "{}/{}", self.trace_path, self.base_class_id)
        }
    }
}

/// Class URI for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub struct ClassUri(String);

impl AsRef<str> for ClassUri {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for ClassUri {
    type Err = NftTransferError;

    fn from_str(class_uri: &str) -> Result<Self, Self::Err> {
        match Uri::from_str(class_uri) {
            Ok(_) => Ok(Self(class_uri.to_string())),
            Err(err) => Err(NftTransferError::InvalidUri {
                uri: class_uri.to_string(),
                validation_error: err,
            }),
        }
    }
}

/// Class data for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassData(Data);

impl Display for ClassData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClassData {
    type Err = NftTransferError;

    fn from_str(class_data: &str) -> Result<Self, Self::Err> {
        // validate the data
        let data = Data::from_str(class_data)?;
        Ok(Self(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denom_validation() -> Result<(), NftTransferError> {
        assert!(ClassId::from_str("").is_err(), "empty base class ID");
        assert!(ClassId::from_str("myclass").is_ok(), "valid base class ID");
        assert!(PrefixedClassId::from_str("").is_err(), "empty class trace");
        assert!(
            PrefixedClassId::from_str("transfer/channel-0/").is_err(),
            "empty base class ID with trace"
        );
        assert!(
            PrefixedClassId::from_str("/myclass").is_err(),
            "empty prefix"
        );
        assert!(PrefixedClassId::from_str("//myclass").is_err(), "empty ids");
        assert!(
            PrefixedClassId::from_str("transfer/").is_err(),
            "single trace"
        );
        assert!(
            PrefixedClassId::from_str("transfer/myclass").is_err(),
            "single trace with base class ID"
        );
        assert!(
            PrefixedClassId::from_str("transfer/channel-0/myclass").is_ok(),
            "valid single trace info"
        );
        assert!(
            PrefixedClassId::from_str("transfer/channel-0/transfer/channel-1/myclass").is_ok(),
            "valid multiple trace info"
        );
        assert!(
            PrefixedClassId::from_str("(transfer)/channel-0/myclass").is_err(),
            "invalid port"
        );
        assert!(
            PrefixedClassId::from_str("transfer/(channel-0)/myclass").is_err(),
            "invalid channel"
        );

        Ok(())
    }

    #[test]
    fn test_denom_trace() -> Result<(), NftTransferError> {
        assert_eq!(
            PrefixedClassId::from_str("transfer/channel-0/myclass")?,
            PrefixedClassId {
                trace_path: "transfer/channel-0".parse()?,
                base_class_id: "myclass".parse()?
            },
            "valid single trace info"
        );
        assert_eq!(
            PrefixedClassId::from_str("transfer/channel-0/transfer/channel-1/myclass")?,
            PrefixedClassId {
                trace_path: "transfer/channel-0/transfer/channel-1".parse()?,
                base_class_id: "myclass".parse()?
            },
            "valid multiple trace info"
        );

        Ok(())
    }

    #[test]
    fn test_denom_serde() -> Result<(), NftTransferError> {
        let dt_str = "transfer/channel-0/myclass";
        let dt = PrefixedClassId::from_str(dt_str)?;
        assert_eq!(dt.to_string(), dt_str, "valid single trace info");

        let dt_str = "transfer/channel-0/transfer/channel-1/myclass";
        let dt = PrefixedClassId::from_str(dt_str)?;
        assert_eq!(dt.to_string(), dt_str, "valid multiple trace info");

        Ok(())
    }

    #[test]
    fn test_trace_path() -> Result<(), NftTransferError> {
        assert!(TracePath::from_str("").is_ok(), "empty trace path");
        assert!(
            TracePath::from_str("transfer/myclass").is_err(),
            "invalid trace path: bad ChannelId"
        );
        assert!(
            TracePath::from_str("transfer//myclass").is_err(),
            "malformed trace path: missing ChannelId"
        );
        assert!(
            TracePath::from_str("transfer/channel-0/").is_err(),
            "malformed trace path: trailing delimiter"
        );

        let prefix_1 = TracePrefix::new("transfer".parse().unwrap(), "channel-1".parse().unwrap());
        let prefix_2 = TracePrefix::new("transfer".parse().unwrap(), "channel-0".parse().unwrap());
        let mut trace_path = TracePath(vec![prefix_1.clone()]);

        trace_path.add_prefix(prefix_2.clone());
        assert_eq!(
            TracePath::from_str("transfer/channel-0/transfer/channel-1")?,
            trace_path
        );
        assert_eq!(
            TracePath(vec![prefix_1.clone(), prefix_2.clone()]),
            trace_path
        );

        trace_path.remove_prefix(&prefix_2);
        assert_eq!(TracePath::from_str("transfer/channel-1")?, trace_path);
        assert_eq!(TracePath(vec![prefix_1.clone()]), trace_path);

        trace_path.remove_prefix(&prefix_1);
        assert!(trace_path.is_empty());

        Ok(())
    }
}
