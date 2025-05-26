// Chain Abstraction Layer and Messaging

#![allow(async_fn_in_trait)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::zkplug::*;
use async_trait::async_trait;

/// Supported chain identifiers. Extend as needed for more chains.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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

/// Errors that can occur in adapters or core SDK logic.
#[derive(thiserror::Error, Debug)]
pub enum AdapterError {
    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid message format or content
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Proof verification failure
    #[error("Proof verification failed: {0}")]
    ProofError(String),

    /// Chain interaction error (e.g., RPC failure)
    #[error("Timeout")]
    Timeout(String),

    /// Chain not supported by this SDK version
    #[error("Chain not supported")]
    ChainNotSupported,

    /// Configuration or initialization error
    #[error("Configuration error")]
    Configuration(String),

    /// Serialization or deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Invalid input data (e.g., missing fields)
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Message not found in the system (e.g., querying a non-existent message)
    #[error("Message not foungd: {0}")]
    MessageNotFound(String),

    /// Unsupported operation or feature
    #[error("Internal error: {0}")]
    Internal(String),

    /// Other errors not covered by specific cases
    #[error("Other: {0}")]
    Other(String),

    /// Anyhow error for general-purpose error handling
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
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

/// Core trait for chain adapters (EVM, Substrate, Solana, etc).
///
/// All methods are synchronous for trait object safety; async versions can be provided via `async-trait`.
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    /// Type for block identifiers (u64, hash, etc).
    type BlockId: Clone + std::fmt::Debug + Send + Sync + 'static;
    /// Type for transaction identifiers.
    type TxId: Clone + std::fmt::Debug + Send + Sync + 'static;
    /// Adapter-specific error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get latest finalized block.
    async fn latest_block(&self) -> Result<Self::BlockId, Self::Error>;

    /// Fetch transaction details by ID.
    async fn get_transaction(&self, tx: &Self::TxId) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Wait until the given block is finalized.
    async fn wait_for_finality(&self, block: &Self::BlockId) -> Result<(), Self::Error>;

    /// Submit a message or proof to the chain (returns TxId).
    async fn submit_message(&self, msg: &FrostMessage) -> Result<Self::TxId, Self::Error>;

    /// Listen for incoming message events (returns all new events).
    async fn listen_for_events(&self) -> Result<Vec<MessageEvent>, Self::Error>;

    /// Optionally verify a message/proof on chain (e.g. via contract call).
    async fn verify_on_chain(&self, msg: &FrostMessage) -> Result<(), Self::Error>;

    /// Estimate native transaction fee for submitting a message.
    async fn estimate_fee(&self, msg: &FrostMessage) -> Result<u128, Self::Error>;

    /// Query message status (pending, confirmed, failed, etc).
    async fn message_status(&self, id: &Uuid) -> Result<MessageStatus, Self::Error>;

    /// Health check (for monitoring).
    async fn health_check(&self) -> Result<(), Self::Error>;
}

// Optionally, provide default/mock implementations for testing if needed.

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