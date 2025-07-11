#![allow(unused_imports)]

//! Core types
//! 
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::messages::ChainId;

/// A finalized block with proof of finality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizedBlock<T> {
    /// The block identifier
    pub block: T,
    /// Optional proof of finality
    pub finality_proof: Option<Vec<u8>>,
    /// When the block was finalized
    pub finalized_at: SystemTime,
    /// Number of confirmations (if applicable)
    pub confirmations: Option<u32>,
}

/// Chain-specific message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionDetails {
    /// Raw transaction bytes
    Raw(Vec<u8>),
    /// Parsed transaction data
    Parsed(ParsedTransaction),
    /// Zero-knowledge proof data
    Proof(ProofData),
}

/// Common transaction fields across chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub hash: Vec<u8>,
    pub from: Option<Vec<u8>>,
    pub to: Option<Vec<u8>>,
    pub value: u128,
    pub data: Vec<u8>,
    pub status: TransactionStatus,
    pub metadata: HashMap<String, String>,
}

/// Transaction execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed(String),
    Dropped,
}

/// Zero-knowledge proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    pub proof_type: String,
    pub proof: Vec<u8>,
    pub public_inputs: Vec<Vec<u8>>,
    pub verification_key: Option<Vec<u8>>,
}

/// Chain capabilities and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainCapabilities {
    /// Whether the chain supports smart contracts
    pub supports_smart_contracts: bool,
    /// Whether the chain has native token support
    pub supports_native_tokens: bool,
    /// Whether the chain supports on-chain verification
    pub supports_onchain_verification: bool,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Supported proof types
    pub proof_types: Vec<String>,
    /// Finality mechanism
    pub finality_type: FinalityType,
    /// Maximum proof size (if applicable)
    pub max_proof_size: Option<usize>,
    /// Whether parallel execution is supported
    pub supports_parallel_execution: bool,
    /// Chain-specific features
    pub features: HashMap<String, String>,
}

/// Types of finality mechanisms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinalityType {
    /// Probabilistic finality (e.g. Bitcoin)
    Probabilistic {
        /// Required confirmations
        confirmations: u32,
    },
    /// Deterministic finality (e.g. Tendermint)
    Deterministic,
    /// Instant finality (e.g. some L2s)
    Instant,
}

/// Chain adapter error categories
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Finality error: {0}")]
    Finality(#[from] FinalityError),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Proof error: {0}")]
    Proof(#[from] ProofError),

    #[error("Message format error: {0}")]
    MessageFormat(String),

    #[error("Chain capability error: {0}")]
    Capability(String),

    #[error("Transaction error: {0}")]
    Transaction(#[from] SubmissionError),

    #[error("Rate limit error: {0}")]
    RateLimit(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Event error: {0}")]
    Event(#[from] EventError),

    #[error("Fee estimation error: {0}")]
    FeeEstimation(#[from] FeeEstimationError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Finality-related errors
#[derive(Debug, thiserror::Error)]
pub enum FinalityError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    
    #[error("Finality timeout after {0} seconds")]
    Timeout(u64),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid finality proof: {0}")]
    InvalidProof(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Proof-related errors
#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    #[error("Failed to generate proof: {0}")]
    Generation(String),
    
    #[error("Failed to verify proof: {0}")]
    Verification(String),
    
    #[error("Invalid proof format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing verification key")]
    MissingVerificationKey,
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Transaction submission errors
#[derive(Debug, thiserror::Error)]
pub enum SubmissionError {
    #[error("Transaction failed: {0}")]
    Failed(String),
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Fee estimation errors
#[derive(Debug, thiserror::Error)]
pub enum FeeEstimationError {
    #[error("Estimation failed: {0}")]
    Failed(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Event-related errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Failed to subscribe: {0}")]
    SubscriptionFailed(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid event data: {0}")]
    InvalidData(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Chain health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Last successful operation timestamp
    pub last_successful: Option<SystemTime>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Total operations performed
    pub total_operations: u64,
    /// Failed operations count
    pub failed_operations: u64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Current connection status
    pub connection_status: ConnectionStatus,
    /// Latest block number seen
    pub latest_block: Option<u64>,
    /// Chain-specific metrics
    pub custom_metrics: HashMap<String, String>,
}

/// Connection health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Unknown,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Unknown
    }
}

/// Message submission options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionOptions {
    /// Maximum gas price willing to pay
    pub max_gas_price: Option<u128>,
    /// Transaction priority (if supported)
    pub priority: Option<u8>,
    /// Whether to wait for finality
    pub wait_for_finality: bool,
    /// Maximum time to wait for finality
    pub finality_timeout: Option<Duration>,
    /// Chain-specific options
    pub custom_options: HashMap<String, String>,
}

impl Default for SubmissionOptions {
    fn default() -> Self {
        Self {
            max_gas_price: None,
            priority: None,
            wait_for_finality: true,
            finality_timeout: Some(Duration::from_secs(300)), // 5 minutes default
            custom_options: HashMap::new(),
        }
    }
} 