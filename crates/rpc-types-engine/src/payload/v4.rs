//! Optimism execution payload envelope V3.

use alloc::vec::Vec;
use alloy_consensus::Block;
use alloy_eips::Decodable2718;
use alloy_primitives::{Bytes, B256, U256};
use alloy_rpc_types_engine::{BlobsBundleV1, ExecutionPayloadV3, PayloadError};

/// The Opstack execution payload for `newPayloadV4` of the engine API introduced with isthmus.
/// See also <https://specs.optimism.io/protocol/isthmus/exec-engine.html#engine_newpayloadv4-api>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct OpExecutionPayloadV4 {
    /// L1 execution payload
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub payload_inner: ExecutionPayloadV3,
    /// OP-Stack Isthmus specific field:
    /// instead of computing the root from a withdrawals list, set it directly.
    /// The "withdrawals" list attribute must be non-nil but empty.
    pub withdrawals_root: B256,
}

impl OpExecutionPayloadV4 {
    /// Converts [`ExecutionPayloadV3`] to [`OpExecutionPayloadV4`] using the given L2 withdrawals
    /// root.
    ///
    /// See also [`ExecutionPayloadV3::from_block_unchecked`].
    pub const fn from_v3_with_withdrawals_root(
        payload: ExecutionPayloadV3,
        withdrawals_root: B256,
    ) -> Self {
        Self { withdrawals_root, payload_inner: payload }
    }

    /// Converts [`OpExecutionPayloadV4`] to [`Block`].
    ///
    /// This performs the same conversion as the underlying V3 payload, but inserts the L2
    /// withdrawals root.
    ///
    /// See also [`ExecutionPayloadV3::try_into_block`].
    pub fn try_into_block<T: Decodable2718>(self) -> Result<Block<T>, PayloadError> {
        let mut base_block = self.payload_inner.try_into_block()?;

        // overwrite l1 withdrawals root with l2 withdrawals root
        base_block.header.withdrawals_root = Some(self.withdrawals_root);

        Ok(base_block)
    }
}

/// This structure maps for the return value of `engine_getPayload` of the beacon chain spec, for
/// V4.
///
/// See also:
/// [Optimism execution payload envelope v4] <https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/exec-engine.md#engine_getpayloadv4>
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct OpExecutionPayloadEnvelopeV4 {
    /// Execution payload V4
    pub execution_payload: ExecutionPayloadV3,
    /// The expected value to be received by the feeRecipient in wei
    pub block_value: U256,
    /// The blobs, commitments, and proofs associated with the executed payload.
    pub blobs_bundle: BlobsBundleV1,
    /// Introduced in V3, this represents a suggestion from the execution layer if the payload
    /// should be used instead of an externally provided one.
    pub should_override_builder: bool,
    /// Ecotone parent beacon block root
    pub parent_beacon_block_root: B256,
    /// A list of opaque [EIP-7685][eip7685] requests.
    ///
    /// [eip7685]: https://eips.ethereum.org/EIPS/eip-7685
    pub execution_requests: Vec<Bytes>,
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_execution_payload_envelope_v4() {
        // modified execution payload envelope v3 with empty deposit, withdrawal, and consolidation
        // requests.
        let response = r#"{"executionPayload":{"parentHash":"0xe927a1448525fb5d32cb50ee1408461a945ba6c39bd5cf5621407d500ecc8de9","feeRecipient":"0x0000000000000000000000000000000000000000","stateRoot":"0x10f8a0830000e8edef6d00cc727ff833f064b1950afd591ae41357f97e543119","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","prevRandao":"0xe0d8b4521a7da1582a713244ffb6a86aa1726932087386e2dc7973f43fc6cb24","blockNumber":"0x1","gasLimit":"0x2ffbd2","gasUsed":"0x0","timestamp":"0x1235","extraData":"0xd883010d00846765746888676f312e32312e30856c696e7578","baseFeePerGas":"0x342770c0","blockHash":"0x44d0fa5f2f73a938ebb96a2a21679eb8dea3e7b7dd8fd9f35aa756dda8bf0a8a","transactions":[],"withdrawals":[],"blobGasUsed":"0x0","excessBlobGas":"0x0"},"blockValue":"0x0","blobsBundle":{"commitments":[],"proofs":[],"blobs":[]},"shouldOverrideBuilder":false,"parentBeaconBlockRoot":"0xdead00000000000000000000000000000000000000000000000000000000beef","executionRequests":["0xdeadbeef"]}"#;
        let envelope: OpExecutionPayloadEnvelopeV4 = serde_json::from_str(response).unwrap();
        assert_eq!(serde_json::to_string(&envelope).unwrap(), response);
    }
}
