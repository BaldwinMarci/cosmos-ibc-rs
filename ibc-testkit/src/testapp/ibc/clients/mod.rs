pub mod mock;

use alloc::fmt::Debug;

use basecoin_store::context::ProvableStore;
use derive_more::{From, TryInto};
use ibc::clients::tendermint::client_state::ClientState as TmClientState;
use ibc::clients::tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::tendermint::types::{
    TENDERMINT_CLIENT_STATE_TYPE_URL, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc::core::client::types::error::ClientError;
use ibc::core::primitives::prelude::*;
use ibc::derive::{ClientState, ConsensusState};
use ibc::primitives::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::client_state::{
    MockClientState, MOCK_CLIENT_STATE_TYPE_URL,
};
use crate::testapp::ibc::clients::mock::consensus_state::{
    MockConsensusState, MOCK_CONSENSUS_STATE_TYPE_URL,
};
use crate::testapp::ibc::core::types::MockGenericContext;

#[derive(Debug, Clone, From, PartialEq, ClientState)]
#[validation(MockGenericContext<S: ProvableStore + Debug>)]
#[execution(MockGenericContext<S: ProvableStore + Debug>)]
pub enum AnyClientState {
    Tendermint(TmClientState),
    Mock(MockClientState),
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if raw.type_url == TENDERMINT_CLIENT_STATE_TYPE_URL {
            Ok(TmClientState::try_from(raw)?.into())
        } else if raw.type_url == MOCK_CLIENT_STATE_TYPE_URL {
            MockClientState::try_from(raw).map(Into::into)
        } else {
            Err(ClientError::Other {
                description: "failed to deserialize message".to_string(),
            })
        }
    }
}

impl From<AnyClientState> for Any {
    fn from(host_client_state: AnyClientState) -> Self {
        match host_client_state {
            AnyClientState::Tendermint(cs) => cs.into(),
            AnyClientState::Mock(cs) => cs.into(),
        }
    }
}

#[derive(Debug, Clone, From, TryInto, PartialEq, ConsensusState)]
pub enum AnyConsensusState {
    Tendermint(TmConsensusState),
    Mock(MockConsensusState),
}

impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if raw.type_url == TENDERMINT_CONSENSUS_STATE_TYPE_URL {
            Ok(TmConsensusState::try_from(raw)?.into())
        } else if raw.type_url == MOCK_CONSENSUS_STATE_TYPE_URL {
            MockConsensusState::try_from(raw).map(Into::into)
        } else {
            Err(ClientError::Other {
                description: "failed to deserialize message".to_string(),
            })
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(host_consensus_state: AnyConsensusState) -> Self {
        match host_consensus_state {
            AnyConsensusState::Tendermint(cs) => cs.into(),
            AnyConsensusState::Mock(cs) => cs.into(),
        }
    }
}
