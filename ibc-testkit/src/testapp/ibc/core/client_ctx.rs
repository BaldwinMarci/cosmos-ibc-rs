use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use ibc::clients::tendermint::context::{
    CommonContext as TmCommonContext, ValidationContext as TmValidationContext,
};
use ibc::core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, PortId};
use ibc::core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc::core::host::ValidationContext;
use ibc::core::primitives::Timestamp;

use crate::testapp::ibc::clients::mock::client_state::MockClientContext;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use crate::testapp::ibc::core::types::MockContext;

pub type PortChannelIdMap<V> = BTreeMap<PortId, BTreeMap<ChannelId, V>>;

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord {
    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<AnyClientState>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: BTreeMap<Height, AnyConsensusState>,
}

impl MockClientContext for MockContext {
    type ConversionError = &'static str;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        ValidationContext::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }
}

impl TmCommonContext for MockContext {
    type ConversionError = &'static str;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        ValidationContext::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }

    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let ibc_store = self.ibc_store.lock();
        let client_record =
            ibc_store
                .clients
                .get(client_id)
                .ok_or_else(|| ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                })?;

        let heights = client_record.consensus_states.keys().cloned().collect();

        Ok(heights)
    }
}

impl TmValidationContext for MockContext {
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let ibc_store = self.ibc_store.lock();
        let client_record =
            ibc_store
                .clients
                .get(client_id)
                .ok_or_else(|| ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                })?;

        // Get the consensus state heights and sort them in ascending order.
        let mut heights: Vec<Height> = client_record.consensus_states.keys().cloned().collect();
        heights.sort();

        // Search for next state.
        for h in heights {
            if h > *height {
                // unwrap should never happen, as the consensus state for h must exist
                return Ok(Some(
                    client_record
                        .consensus_states
                        .get(&h)
                        .expect("Never fails")
                        .clone(),
                ));
            }
        }
        Ok(None)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let ibc_store = self.ibc_store.lock();
        let client_record =
            ibc_store
                .clients
                .get(client_id)
                .ok_or_else(|| ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                })?;

        // Get the consensus state heights and sort them in descending order.
        let mut heights: Vec<Height> = client_record.consensus_states.keys().cloned().collect();
        heights.sort_by(|a, b| b.cmp(a));

        // Search for previous state.
        for h in heights {
            if h < *height {
                // unwrap should never happen, as the consensus state for h must exist
                return Ok(Some(
                    client_record
                        .consensus_states
                        .get(&h)
                        .expect("Never fails")
                        .clone(),
                ));
            }
        }
        Ok(None)
    }
}

impl ClientValidationContext for MockContext {
    fn client_update_meta(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<(Timestamp, Height), ContextError> {
        let key = (client_id.clone(), height);
        (|| {
            let ibc_store = self.ibc_store.lock();
            let time = ibc_store.client_processed_times.get(&key)?;
            let height = ibc_store.client_processed_heights.get(&key)?;
            Some((*time, *height))
        })()
        .ok_or(ClientError::ProcessedTimeNotFound {
            client_id: key.0,
            height,
        })
        .map_err(ContextError::from)
    }
}

impl ClientExecutionContext for MockContext {
    type V = Self;
    type AnyClientState = AnyClientState;
    type AnyConsensusState = AnyConsensusState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        let mut ibc_store = self.ibc_store.lock();

        let client_id = client_state_path.0;
        let client_record = ibc_store
            .clients
            .entry(client_id)
            .or_insert(MockClientRecord {
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        client_record.client_state = Some(client_state);

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        let mut ibc_store = self.ibc_store.lock();

        let client_record = ibc_store
            .clients
            .entry(consensus_state_path.client_id)
            .or_insert(MockClientRecord {
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        let height = Height::new(
            consensus_state_path.revision_number,
            consensus_state_path.revision_height,
        )
        .expect("Never fails");
        client_record
            .consensus_states
            .insert(height, consensus_state);

        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        let mut ibc_store = self.ibc_store.lock();

        let client_record = ibc_store
            .clients
            .entry(consensus_state_path.client_id)
            .or_insert(MockClientRecord {
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        let height = Height::new(
            consensus_state_path.revision_number,
            consensus_state_path.revision_height,
        )
        .expect("Never fails");

        client_record.consensus_states.remove(&height);

        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        let key = (client_id.clone(), height);
        let mut ibc_store = self.ibc_store.lock();
        ibc_store.client_processed_times.remove(&key);
        ibc_store.client_processed_heights.remove(&key);
        Ok(())
    }

    fn store_update_meta(
        &mut self,
        client_id: &ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        let mut ibc_store = self.ibc_store.lock();
        ibc_store
            .client_processed_times
            .insert((client_id.clone(), height), host_timestamp);
        ibc_store
            .client_processed_heights
            .insert((client_id.clone(), height), host_height);
        Ok(())
    }
}
