use crate::context::ValidationContext as TmValidationContext;

use ibc_client_tendermint_types::client_state::check_header_trusted_next_validator_set;
use ibc_client_tendermint_types::consensus_state::ConsensusState as TmConsensusState;
use ibc_client_tendermint_types::error::{Error, IntoResult};
use ibc_client_tendermint_types::header::Header as TmHeader;
use ibc_client_tendermint_types::misbehaviour::Misbehaviour as TmMisbehaviour;

use ibc_core_client_types::error::ClientError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_core_host_types::path::ClientConsensusStatePath;
use ibc_primitives::{prelude::*, Timestamp};

use super::ClientState;

use tendermint_light_client_verifier::Verifier;

impl ClientState {
    // verify_misbehaviour determines whether or not two conflicting headers at
    // the same height would have convinced the light client.
    pub fn verify_misbehaviour<ClientValidationContext>(
        &self,
        ctx: &ClientValidationContext,
        client_id: &ClientId,
        misbehaviour: TmMisbehaviour,
    ) -> Result<(), ClientError>
    where
        ClientValidationContext: TmValidationContext,
    {
        misbehaviour.validate_basic()?;

        let header_1 = misbehaviour.header1();
        let trusted_consensus_state_1 = {
            let consensus_state_path =
                ClientConsensusStatePath::new(client_id.clone(), header_1.trusted_height.revision_number(), header_1.trusted_height.revision_height());
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            consensus_state
                .try_into()
                .map_err(|err| ClientError::Other {
                    description: err.to_string(),
                })?
        };

        let header_2 = misbehaviour.header2();
        let trusted_consensus_state_2 = {
            let consensus_state_path =
                ClientConsensusStatePath::new(client_id.clone(), header_2.trusted_height.revision_number(), header_2.trusted_height.revision_height());
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            consensus_state
                .try_into()
                .map_err(|err| ClientError::Other {
                    description: err.to_string(),
                })?
        };

        let current_timestamp = ctx.host_timestamp()?;
        self.verify_misbehaviour_header(
            header_1,
            &trusted_consensus_state_1,
            current_timestamp,
        )?;
        self.verify_misbehaviour_header(header_2, &trusted_consensus_state_2, current_timestamp)
    }

    pub fn verify_misbehaviour_header(
        &self,
        header: &TmHeader,
        trusted_consensus_state: &TmConsensusState,
        current_timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        // ensure correctness of the trusted next validator set provided by the relayer
        check_header_trusted_next_validator_set(header, trusted_consensus_state)?;

        // ensure trusted consensus state is within trusting period
        {
            let duration_since_consensus_state = current_timestamp
                .duration_since(&(trusted_consensus_state.timestamp().into()))
                .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
                    time1: trusted_consensus_state.timestamp().into(),
                    time2: current_timestamp,
                })?;

            if duration_since_consensus_state >= self.0.trusting_period {
                return Err(Error::ConsensusStateTimestampGteTrustingPeriod {
                    duration_since_consensus_state,
                    trusting_period: self.0.trusting_period,
                }
                .into());
            }
        }

        // main header verification, delegated to the tendermint-light-client crate.
        let untrusted_state = header.as_untrusted_block_state();

        let chain_id =
            self
                .0
                .chain_id
                .to_string()
                .try_into()
                .map_err(|e| ClientError::Other {
                    description: format!("failed to parse chain id: {}", e),
                })?;
        let trusted_state = header.as_trusted_block_state(trusted_consensus_state, &chain_id)?;

        let options = self.0.as_light_client_options()?;
        let current_timestamp = current_timestamp.into_tm_time().ok_or(ClientError::Other {
            description: "host timestamp must not be zero".to_string(),
        })?;

        self
            .0
            .verifier
            .verify_misbehaviour_header(untrusted_state, trusted_state, &options, current_timestamp)
            .into_result()?;

        Ok(())
    }

    pub fn check_for_misbehaviour_misbehavior(
        &self,
        misbehaviour: &TmMisbehaviour,
    ) -> Result<bool, ClientError> {
        let header_1 = misbehaviour.header1();
        let header_2 = misbehaviour.header2();

        if header_1.height() == header_2.height() {
            // when the height of the 2 headers are equal, we only have evidence
            // of misbehaviour in the case where the headers are different
            // (otherwise, the same header was added twice in the message,
            // and this is evidence of nothing)
            Ok(header_1.signed_header.commit.block_id.hash
                != header_2.signed_header.commit.block_id.hash)
        } else {
            // header_1 is at greater height than header_2, therefore
            // header_1 time must be less than or equal to
            // header_2 time in order to be valid misbehaviour (violation of
            // monotonic time).
            Ok(header_1.signed_header.header.time <= header_2.signed_header.header.time)
        }
    }
}
