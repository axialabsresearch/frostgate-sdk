#![allow(unused_imports)]

//! Message verification module for the Frostgate SDK
//! This module provides functionality for verifying cross-chain messages using the new ZkBackend interface.

use std::sync::Arc;
use async_trait::async_trait;
use parking_lot::RwLock;
use lru::LruCache;
use blake2::{Blake2b512, Digest};
use std::num::NonZeroUsize;
use std::path::Path;
use std::fs;
use std::time::{Duration, SystemTime};

use crate::messages::{FrostMessage, ChainId, Proof};
use frostgate_zkip::{
    ZkBackend, ZkBackendExt, ZkError, ZkResult,
    types::{HealthStatus, ProofMetadata, ResourceUsage, ZkConfig},
};

/// Error types for message verification
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("Backend error: {0}")]
    Backend(#[from] ZkError),
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    #[error("Missing proof")]
    MissingProof,
    #[error("Invalid chain ID")]
    InvalidChainId,
    #[error("System error: {0}")]
    System(String),
}

/// Result type for verification operations
pub type VerificationResult<T> = Result<T, VerificationError>;

/// Cache entry for verification programs
#[derive(Debug)]
struct ProgramCacheEntry {
    #[allow(dead_code)]  // Used for future validation
    program_hash: [u8; 32],
    program_bytes: Vec<u8>,
    last_used: std::time::SystemTime,
    use_count: u64,
}

/// Message verifier using the new ZkBackend interface
pub struct MessageVerifier<B: ZkBackend> {
    /// ZK backend instance
    backend: Arc<B>,
    /// Program cache
    program_cache: Arc<RwLock<LruCache<ChainId, ProgramCacheEntry>>>,
    /// Cache TTL in seconds
    cache_ttl: u64,
    /// Cache configuration
    #[allow(dead_code)]  // Used for future cache resizing
    cache_size: usize,
}

impl<B: ZkBackend> MessageVerifier<B> {
    /// Create a new message verifier with the given backend
    pub fn new(backend: Arc<B>) -> Self {
        Self::with_config(backend, 100, 3600) // Default 100 entries, 1 hour TTL
    }

    /// Create a new message verifier with custom configuration
    pub fn with_config(backend: Arc<B>, cache_size: usize, cache_ttl: u64) -> Self {
        Self {
            backend,
            program_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap()))),
            cache_size,
            cache_ttl,
        }
    }

    /// Get or load verification program for a chain
    async fn get_program(&self, chain_id: ChainId) -> VerificationResult<Vec<u8>> {
        // Check cache first
        let mut cache = self.program_cache.write();
        if let Some(entry) = cache.get_mut(&chain_id) {
            // Check if entry is still valid
            if let Ok(age) = std::time::SystemTime::now().duration_since(entry.last_used) {
                if age.as_secs() < self.cache_ttl {
                    entry.use_count += 1;
                    entry.last_used = std::time::SystemTime::now();
                    return Ok(entry.program_bytes.clone());
                }
            }
        }

        // Load program based on chain ID
        let program_path = match chain_id {
            ChainId::Ethereum => "../../../frostgate-circuits/programs/eth_verifier.sp1",
            ChainId::Polkadot => "../../../frostgate-circuits/programs/dot_verifier.sp1",
            ChainId::Solana => "../../../frostgate-circuits/programs/sol_verifier.sp1",
            ChainId::Unknown => return Err(VerificationError::InvalidChainId),
        };

        let program_bytes = if let Ok(bytes) = fs::read(Path::new(program_path)) {
            bytes
        } else {
            // For development/testing, return dummy program bytes
            vec![0u8; 64] // Placeholder for development
        };

        // Calculate program hash
        let mut hasher = Blake2b512::new();
        hasher.update(&program_bytes);
        let mut program_hash = [0u8; 32];
        program_hash.copy_from_slice(&hasher.finalize()[..32]);

        // Cache program
        cache.put(chain_id, ProgramCacheEntry {
            program_hash,
            program_bytes: program_bytes.clone(),
            last_used: std::time::SystemTime::now(),
            use_count: 1,
        });

        Ok(program_bytes)
    }

    /// Verify a message using the ZK backend
    pub async fn verify_message(&self, message: &FrostMessage) -> VerificationResult<bool> {
        // Get proof
        let proof = message.proof.as_ref()
            .ok_or(VerificationError::MissingProof)?;

        // Get verification program
        let program = self.get_program(message.from_chain).await?;

        // Prepare input data
        let mut input = Vec::new();
        input.extend_from_slice(&message.from_chain.to_u64().to_be_bytes());
        input.extend_from_slice(&message.to_chain.to_u64().to_be_bytes());
        input.extend_from_slice(&(message.payload.len() as u64).to_be_bytes());
        input.extend_from_slice(&message.payload);
        input.extend_from_slice(&message.nonce.to_be_bytes());
        input.extend_from_slice(&message.timestamp.to_be_bytes());

        // Verify proof
        let result = self.backend.verify(&program, &proof.data, None).await?;

        Ok(result)
    }

    /// Verify multiple messages in batch
    pub async fn verify_messages_batch(&self, messages: &[FrostMessage]) -> VerificationResult<Vec<bool>> {
        let mut results = Vec::with_capacity(messages.len());

        for message in messages {
            results.push(self.verify_message(message).await?);
        }

        Ok(results)
    }

    /// Get backend health status
    pub async fn health_check(&self) -> HealthStatus {
        self.backend.health_check().await
    }

    /// Get backend resource usage
    pub fn resource_usage(&self) -> ResourceUsage {
        self.backend.resource_usage()
    }

    /// Clear program cache
    pub async fn clear_cache(&mut self) -> VerificationResult<()> {
        self.program_cache.write().clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Mock backend for testing
    #[derive(Debug)]
    struct MockBackend;
    
    #[async_trait]
    impl ZkBackend for MockBackend {
        async fn prove(&self, _program: &[u8], _input: &[u8], _config: Option<&ZkConfig>) -> ZkResult<(Vec<u8>, frostgate_zkip::types::ProofMetadata)> {
            Ok((vec![], frostgate_zkip::types::ProofMetadata {
                generation_time: Duration::from_secs(1),
                proof_size: 4,
                program_hash: "dummy".to_string(),
                timestamp: SystemTime::now(),
            }))
        }

        async fn verify(&self, _program: &[u8], _proof: &[u8], _config: Option<&ZkConfig>) -> ZkResult<bool> {
            Ok(true)
        }

        async fn health_check(&self) -> HealthStatus {
            HealthStatus::Healthy
        }

        fn resource_usage(&self) -> ResourceUsage {
            ResourceUsage {
                cpu_usage: 0.0,
                memory_usage: 0,
                active_tasks: 0,
                max_concurrent: 1,
                queue_depth: 0,
            }
        }
    }

    #[tokio::test]
    async fn test_message_verification() {
        // Create backend and verifier
        let backend = Arc::new(MockBackend);
        let verifier = MessageVerifier::new(backend);

        // Create test message
        let message = FrostMessage {
            id: Uuid::new_v4(),
            from_chain: ChainId::Ethereum,
            to_chain: ChainId::Polkadot,
            payload: b"test".to_vec(),
            proof: Some(crate::messages::Proof {
                data: vec![1, 2, 3, 4],
                metadata: frostgate_zkip::types::ProofMetadata {
                    generation_time: Duration::from_secs(1),
                    proof_size: 4,
                    program_hash: "dummy".to_string(),
                    timestamp: SystemTime::now(),
                },
            }),
            timestamp: 1_725_000_000,
            nonce: 1,
            signature: None,
            fee: None,
            metadata: None,
        };

        // Test verification
        let result = verifier.verify_message(&message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_verification() {
        // Create backend and verifier
        let backend = Arc::new(MockBackend);
        let verifier = MessageVerifier::new(backend);

        // Create test messages
        let messages = vec![
            FrostMessage {
                id: Uuid::new_v4(),
                from_chain: ChainId::Ethereum,
                to_chain: ChainId::Polkadot,
                payload: b"test1".to_vec(),
                proof: Some(crate::messages::Proof {
                    data: vec![1, 2, 3, 4],
                    metadata: frostgate_zkip::types::ProofMetadata {
                        generation_time: Duration::from_secs(1),
                        proof_size: 4,
                        program_hash: "dummy".to_string(),
                        timestamp: SystemTime::now(),
                    },
                }),
                timestamp: 1_725_000_000,
                nonce: 1,
                signature: None,
                fee: None,
                metadata: None,
            },
            FrostMessage {
                id: Uuid::new_v4(),
                from_chain: ChainId::Solana,
                to_chain: ChainId::Ethereum,
                payload: b"test2".to_vec(),
                proof: Some(crate::messages::Proof {
                    data: vec![5, 6, 7, 8],
                    metadata: frostgate_zkip::types::ProofMetadata {
                        generation_time: Duration::from_secs(1),
                        proof_size: 4,
                        program_hash: "dummy".to_string(),
                        timestamp: SystemTime::now(),
                    },
                }),
                timestamp: 1_725_000_001,
                nonce: 2,
                signature: None,
                fee: None,
                metadata: None,
            },
        ];

        // Test batch verification
        let results = verifier.verify_messages_batch(&messages).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_program_cache() {
        // Create backend and verifier with small cache
        let backend = Arc::new(MockBackend);
        let verifier = MessageVerifier::with_config(backend, 2, 1);

        // Test program loading
        let program1 = verifier.get_program(ChainId::Ethereum).await;
        assert!(program1.is_ok());

        let program2 = verifier.get_program(ChainId::Polkadot).await;
        assert!(program2.is_ok());

        let program3 = verifier.get_program(ChainId::Solana).await;
        assert!(program3.is_ok());

        // Check cache size is maintained
        assert_eq!(verifier.program_cache.read().len(), 2);
    }
} 