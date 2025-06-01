#![allow(async_fn_in_trait)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use frostgate_zkip::zkplug::*;
use async_trait::async_trait;

/// Supported chain identifiers. Extend as needed for more chains.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum ChainId {
    Ethereum,
    Polkadot,
    Solana,
    // Extend as needed (Cosmos, Aptos, etc)
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            ChainId::Ethereum => "Ethereum",
            ChainId::Polkadot => "Polkadot",
            ChainId::Solana => "Solana",
            ChainId::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl std::convert::TryFrom<u64> for ChainId {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ChainId::Ethereum),
            1 => Ok(ChainId::Polkadot),
            2 => Ok(ChainId::Solana),
            _ => Ok(ChainId::Unknown),
        }
    }
}

/// The canonical cross-chain message structure for Frostgate.
///
/// Includes all data necessary for verification and replay protection.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrostMessage {
    /// Unique message ID (UUID v4 for global uniqueness).
    pub id: Uuid,
    /// Source chain identifier.
    pub from_chain: ChainId,
    /// Destination chain identifier.
    pub to_chain: ChainId,
    /// Arbitrary user/application payload (should be encoded as required).
    pub payload: Vec<u8>,
    /// Zero-knowledge proof attached to the message (optional for some flows).
    pub proof: Option<ZkProof<Vec<u8>>>,
    /// Unix timestamp (seconds) for message creation.
    pub timestamp: u64,
    /// Per-sender nonce for replay protection.
    pub nonce: u64,
    /// Optional cryptographic signature (by relayer/operator, not always required).
    pub signature: Option<Vec<u8>>,
    /// Optional relayer or protocol fee (in smallest unit of source chain).
    pub fee: Option<u128>,
    /// Extensible metadata for debugging, audit, or protocol extensions.
    pub metadata: Option<HashMap<String, String>>,
}

impl FrostMessage {
    /// Construct a new unsigned FrostMessage.
    pub fn new(
        from_chain: ChainId,
        to_chain: ChainId,
        payload: Vec<u8>,
        nonce: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_chain,
            to_chain,
            payload,
            proof: None,
            timestamp,
            nonce,
            signature: None,
            fee: None,
            metadata: None,
        }
    }
}

/// Message status for querying relay pipeline progress.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MessageStatus {
    Pending,
    InFlight,
    Confirmed,
    Failed(String),
}

/// Transaction hash or equivalent per chain.
pub type TxHash = Vec<u8>;

/// Message event structure (from source chain).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageEvent {
    pub message: FrostMessage,
    pub tx_hash: Option<TxHash>,
    pub block_number: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frost_message_basic() {
        let msg = FrostMessage::new(
            ChainId::Ethereum,
            ChainId::Solana,
            b"test-payload".to_vec(),
            1,
            1_725_000_000,
        );
        let ser = serde_json::to_string(&msg).unwrap();
        let de: FrostMessage = serde_json::from_str(&ser).unwrap();
        assert_eq!(msg.from_chain, de.from_chain);
        assert_eq!(msg.payload, de.payload);
    }
}