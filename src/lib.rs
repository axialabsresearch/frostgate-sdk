//! # Frostgate SDK
//!
//! The Frostgate SDK provides a high-level interface for developers to interact with
//! zero-knowledge proof systems. It abstracts away the complexity of working directly
//! with ZK circuits and provides a developer-friendly API.
//!
//! ## Core Features
//!
//! - **Message Processing**: Easy-to-use message handling system
//! - **Type Safety**: Strong typing for ZK operations
//! - **Verification Tools**: Simplified proof verification
//! - **Extensible Traits**: Flexible trait system for custom implementations
//!
//! ## Module Structure
//!
//! - [`messages`]: Message handling and processing
//! - [`types`]: Core type definitions
//! - [`traits`]: Extensible trait system
//! - [`verification`]: Proof verification utilities
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use frostgate_sdk::{
//!     messages::Message,
//!     verification::Verifier,
//!     types::ProofRequest,
//! };
//!
//! async fn verify_message(message: Message, verifier: &impl Verifier) -> bool {
//!     let request = ProofRequest::new(message);
//!     verifier.verify(request).await.is_ok()
//! }
//! ```
//!
//! ## Usage Examples
//!
//! ### Message Processing
//!
//! ```rust,no_run
//! use frostgate_sdk::messages::Message;
//!
//! let message = Message::new("Hello, World!");
//! ```
//!
//! ### Verification
//!
//! ```rust,no_run
//! use frostgate_sdk::verification::Verifier;
//!
//! async fn verify<V: Verifier>(verifier: &V, proof: &[u8]) {
//!     verifier.verify_proof(proof).await;
//! }
//! ```
//!
//! ## Error Handling
//!
//! The SDK uses custom error types for each module, all implementing standard
//! error traits for consistency and interoperability.
//!
//! ## Performance Considerations
//!
//! - Use batch operations when processing multiple messages
//! - Enable caching for repeated operations
//! - Consider using async operations for better throughput
//!
//! ## Feature Flags
//!
//! The SDK provides several feature flags for customizing functionality:
//! - `async`: Enable async support (default)
//! - `std`: Enable standard library features (default)

pub mod messages;
pub mod types;
pub mod traits;
pub mod verification;