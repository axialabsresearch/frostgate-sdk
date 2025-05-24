//! # Frostgate SDK: Core Types and Traits for Cross-Chain ZK Messaging
//!
//! The Frostgate SDK provides the foundational types, traits, and abstractions for building
//! cross-chain, zero-knowledge-enabled messaging and interoperability protocols.
//!
//! ## Overview
//!
//! This crate is the canonical interface layer for the Frostgate protocol. It defines:
//! - **Message formats** for cross-chain communication (`FrostMessage`)
//! - **Proof encapsulation** for ZK backends (`ProofData`)
//! - **Chain identifiers** for multi-chain support (`ChainId`)
//! - **Adapter traits** for blockchain integration (`ChainAdapter`)
//! - **Pluggable ZK proof engine traits** (`Prover`)
//! - **Standard error types** for robust error handling
//!
//! All relayers, provers, verifiers, and chain adapters in the Frostgate ecosystem depend on these abstractions.
//!
//! ## Features
//!
//! - **ZK-agnostic:** Supports any ZK proof system or VM via generic traits and types.
//! - **Cross-chain:** Unified message and adapter interfaces for EVM, Solana, Substrate, and more.
//! - **Extensible:** Easily add new chains, proof systems, or message types.
//! - **Async-ready:** All core traits are async for compatibility with modern Rust and networked backends.
//! - **Production-grade error handling:** Rich error enums for adapters and provers.
//! - **Testable:** Includes basic tests and is designed for easy mocking and extension.
//!
//! ## Key Types
//!
//! - [`FrostMessage`]: Canonical cross-chain message structure, including payload, proof, and metadata.
//! - [`ProofData`]: ZK proof blob and optional metadata for zk-agnostic workflows.
//! - [`ChainId`]: Enum of supported blockchain networks.
//! - [`ChainAdapter`]: Async trait for interacting with any supported chain.
//! - [`Prover`]: Trait for pluggable ZK proof engines (e.g., SP1, Groth16, Plonky2).
//!
//! ## Example
//!
//! ```rust
//! use frostgate_sdk::{FrostMessage, ChainId};
//!
//! let msg = FrostMessage::new(
//!     ChainId::Ethereum,
//!     ChainId::Solana,
//!     b"hello-world".to_vec(),
//!     42,
//!     1_725_000_000,
//! );
//! ```
//!
//! ## Extending
//!
//! - To add a new chain, extend [`ChainId`] and implement [`ChainAdapter`].
//! - To support a new ZK backend, implement [`Prover`] or [`ZkPlug`] (see `zkplug.rs`).
//! 
//! Cheers!

#![allow(async_fn_in_trait)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use async_trait::async_trait;

// ----------- ChainId ------------

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

// ----------- ProofData ------------

/// ZK proof data and optional metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProofData {
    /// The proof as a byte vector.
    pub proof: Vec<u8>,
    /// Optionally, the type of proof or circuit used (for zk-agnostic backends).
    pub proof_type: Option<String>,
    /// Optional public inputs (serialized).
    pub public_inputs: Option<Vec<u8>>,
    /// Optional recursive or extra metadata.
    pub metadata: Option<HashMap<String, String>>,
}

impl ProofData {
    pub fn new(proof: Vec<u8>) -> Self {
        Self {
            proof,
            proof_type: None,
            public_inputs: None,
            metadata: None,
        }
    }
}

// ----------- FrostMessage ------------

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
    pub proof: Option<ProofData>,
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

// ----------- AdapterError ------------

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
    Timeout,

    /// Chain not supported by this SDK version
    #[error("Chain not supported")]
    ChainNotSupported,

    /// Configuration or initialization error
    #[error("Configuration error")]
    Configuration,

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

// ----------- Prover ------------

/// Result of a successful ZK proof operation.
pub type ProverResult = Result<ProofData, ProverError>;

/// Errors for the prover.
#[derive(thiserror::Error, Debug)]
pub enum ProverError {
    #[error("Proof generation failed: {0}")]
    Failure(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Unsupported circuit/proof type")]
    Unsupported,
    #[error("Timeout")]
    Timeout,
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

/// Trait for pluggable ZK proof engines (e.g., SP1, Groth16, Plonky2, etc).
pub trait Prover: Send + Sync {
    /// Prove given input data, returning a proof blob.
    fn prove(&self, input_data: &[u8]) -> ProverResult;
}

// ----------- ChainAdapter ------------

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

    #[test]
    fn proof_data_basic() {
        let data = ProofData::new(vec![1, 2, 3, 4]);
        assert_eq!(data.proof, vec![1, 2, 3, 4]);
    }
}