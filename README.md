# Frostgate SDK

The Frostgate SDK provides the core types and traits for building cross-chain applications using the Frostgate protocol. It includes message types, chain adapters, and utility functions for interacting with the Frostgate ecosystem.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
frostgate-sdk = { git = "https://github.com/frostgate/frostgate-sdk.git" }
```

## Quick Start

```rust
use frostgate_sdk::{FrostMessage, ChainId};
use frostgate_icap::EthereumAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize adapter
    let config = EthereumConfig::default();
    let adapter = EthereumAdapter::new(config);

    // Create message
    let message = FrostMessage::new(
        ChainId::Ethereum,
        ChainId::Polkadot,
        payload.to_vec(),
        nonce,
        timestamp,
    );

    // Submit message
    let tx_hash = adapter.submit_message(&message).await?;

    // Wait for confirmation
    adapter.wait_for_finality(&tx_hash).await?;

    Ok(())
}
```

## Core Types

### FrostMessage

The canonical cross-chain message format:

```rust
pub struct FrostMessage {
    pub id: Uuid,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub payload: Vec<u8>,
    pub proof: Option<ZkProof>,
    pub timestamp: u64,
    pub nonce: u64,
    pub signature: Option<Vec<u8>>,
    pub fee: Option<u128>,
    pub metadata: Option<HashMap<String, String>>,
}
```

### ChainId

Supported blockchain networks:

```rust
pub enum ChainId {
    Ethereum,
    Polkadot,
    Solana,
    Unknown,
}
```

### MessageStatus

Message processing states:

```rust
pub enum MessageStatus {
    Pending,
    InFlight,
    Confirmed,
    Failed(String),
}
```

## Chain Adapters

The `ChainAdapter` trait defines the interface for blockchain-specific implementations:

```rust
#[async_trait]
pub trait ChainAdapter {
    type Error: std::error::Error;
    type TxId: Clone + Send + Sync;

    async fn submit_message(&self, msg: &FrostMessage) -> Result<Self::TxId, Self::Error>;
    async fn verify_on_chain(&self, msg: &FrostMessage) -> Result<(), Self::Error>;
    async fn listen_for_events(&self) -> Result<Vec<MessageEvent>, Self::Error>;
    async fn message_status(&self, msg_id: &Uuid) -> Result<MessageStatus, Self::Error>;
    async fn wait_for_finality(&self, tx_id: &Self::TxId) -> Result<(), Self::Error>;
    // ...
}
```

## Examples

### Basic Message Submission

```rust
use frostgate_sdk::{FrostMessage, ChainId};

// Create message
let message = FrostMessage::new(
    ChainId::Ethereum,
    ChainId::Polkadot,
    payload,
    nonce,
    timestamp,
);

// Add metadata
message.metadata.insert("version".to_string(), "1.0".to_string());

// Submit message
let tx_hash = adapter.submit_message(&message).await?;
```

### Event Listening

```rust
use frostgate_sdk::MessageEvent;

// Listen for events
let events = adapter.listen_for_events().await?;

for event in events {
    println!("Message ID: {}", event.message.id);
    println!("From: {}", event.message.from_chain);
    println!("To: {}", event.message.to_chain);
}
```

### Message Status Tracking

```rust
use frostgate_sdk::MessageStatus;

match adapter.message_status(&message_id).await? {
    MessageStatus::Pending => println!("Message is pending"),
    MessageStatus::InFlight => println!("Message is in flight"),
    MessageStatus::Confirmed => println!("Message is confirmed"),
    MessageStatus::Failed(error) => println!("Message failed: {}", error),
}
```

## Error Handling

The SDK provides detailed error types:

```rust
pub enum AdapterError {
    Network(String),
    Serialization(String),
    InvalidMessage(String),
    ProofError(String),
    Timeout(String),
    Other(String),
}
```

Example error handling:

```rust
match adapter.submit_message(&message).await {
    Ok(tx_hash) => println!("Message submitted: {}", tx_hash),
    Err(AdapterError::Network(e)) => println!("Network error: {}", e),
    Err(AdapterError::InvalidMessage(e)) => println!("Invalid message: {}", e),
    Err(e) => println!("Other error: {}", e),
}
```

## Configuration

Each chain adapter has its own configuration type:

```rust
// Ethereum configuration
let eth_config = EthereumConfig {
    rpc_url: "https://mainnet.infura.io/v3/YOUR-KEY",
    private_key: "your-private-key",
    confirmations: 12,
    // ...
};

// Polkadot configuration
let dot_config = PolkadotConfig {
    ws_url: "wss://rpc.polkadot.io",
    seed_phrase: "your-seed-phrase",
    // ...
};
```

## Best Practices

1. **Error Handling**
   - Always handle network errors
   - Implement proper retries
   - Log detailed error information

2. **Message Management**
   - Use unique nonces
   - Track message status
   - Handle timeouts properly

3. **Resource Management**
   - Close connections properly
   - Implement rate limiting
   - Monitor resource usage

4. **Security**
   - Secure private keys
   - Validate input data
   - Check message signatures

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details on how to contribute to the SDK.

## License

This project is licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.
