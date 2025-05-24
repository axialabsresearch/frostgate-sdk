//! # Frostgate SDK: Core Types and Traits for Cross-Chain ZK Messaging
//!
//! The Frostgate SDK provides the foundational types, traits, and abstractions for building
//! cross-chain, zero-knowledge-enabled messaging and interoperability protocols.
//!
//! ## Overview
//!
//! This crate is the canonical interface layer for the Frostgate protocol. It defines:
//! - **Message formats** for cross-chain communication ([`FrostMessage`])
//! - **Generic ZK proof encapsulation** for pluggable backends ([`ZkProof`], [`ProofMetadata`])
//! - **Chain identifiers** for multi-chain support ([`ChainId`])
//! - **Adapter traits** for blockchain integration ([`ChainAdapter`])
//! - **Pluggable ZK proof engine traits** ([`Prover`], [`ZkPlug`])
//! - **Standard error types** for robust error handling
//!
//! All relayers, provers, verifiers, and chain adapters in the Frostgate ecosystem depend on these abstractions.
//!
//! ## Features
//!
//! - **ZK-agnostic:** Supports any ZK proof system or VM via generic traits and types ([`ZkPlug`], [`ZkProof`]).
//! - **Cross-chain:** Unified message and adapter interfaces for EVM, Solana, Substrate, and more.
//! - **Extensible:** Easily add new chains, proof systems, or message types.
//! - **Async-ready:** All core traits are async for compatibility with modern Rust and networked backends.
//! - **Production-grade error handling:** Rich error enums for adapters and provers.
//! - **Testable:** Includes basic tests and is designed for easy mocking and extension.
//!
//! ## Key Types
//!
//! - [`FrostMessage`]: Canonical cross-chain message structure, including payload, proof, and metadata.
//! - [`ZkProof`], [`ProofMetadata`]: Generic ZK proof wrapper and rich metadata for zk-agnostic workflows.
//! - [`ChainId`]: Enum of supported blockchain networks.
//! - [`ChainAdapter`]: Async trait for interacting with any supported chain.
//! - [`Prover`], [`ZkPlug`]: Traits for pluggable ZK proof engines (e.g., SP1, Groth16, Plonky2).
//!
//! ## Example
//!
//! ```rust
//! use frostgate_sdk::chainadapter::{FrostMessage, ChainId};
//! use frostgate_sdk::zkplug::{ZkProof, ProofMetadata};
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
//! - To add a new chain, extend [`ChainId`] and implement [`ChainAdapter`] (see `chaainadapter.rs`).
//! - To support a new ZK backend, implement [`ZkPlug`] (see `zkplug.rs`).
//!
//! Cheers!

pub mod zkplug;
pub mod chainadapter;