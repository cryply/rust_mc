# PostgreSQL Encrypted Client (Rust)

A Rust implementation of the **"Always Encrypted"** pattern for PostgreSQL, providing client-side encryption for sensitive data.

## Architecture

This implementation follows the encryption architecture :

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           KEY HIERARCHY (Slide 1)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────┐                                                       │
│   │   Master Key    │  ← Derived from password (like DPAPI)                 │
│   │   (CMK)         │    In production: AWS KMS, HashiCorp Vault            │
│   └────────┬────────┘                                                       │
│            │                                                                │
│            │ encrypts                                                       │
│            ▼                                                                │
│   ┌─────────────────┐                                                       │
│   │ Column          │  ← One per table/column group                         │
│   │ Encryption Key  │    Derived deterministically from master key          │
│   │ (CEK)           │                                                       │
│   └────────┬────────┘                                                       │
│            │                                                                │
│            │ used by                                                        │
│            ▼                                                                │
│   ┌─────────────────┐                                                       │
│   │ Enhanced Client │  ← AES-256-GCM encryption                             │
│   │ Driver          │    Handles all encrypt/decrypt operations             │
│   └─────────────────┘                                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                    DATA FLOW (Slides 2 & 3)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  [Application]                                                              │
│        │                                                                    │
│        │ plaintext: "john@example.com"                                      │
│        ▼                                                                    │
│  ┌─────────────────┐                                                        │
│  │ Enhanced Client │                                                        │
│  │ Driver          │  ← Encrypts BEFORE sending to DB                       │
│  └────────┬────────┘                                                        │
│           │                                                                 │
│           │ ciphertext: "A3f8x9Kp2mN..."                                    │
│           ▼                                                                 │
│  ┌─────────────────┐                                                        │
│  │   PostgreSQL    │  ← Only ever sees encrypted data!                      │
│  │   Database      │    SELECT shows ciphertext                             │
│  └────────┬────────┘                                                        │
│           │                                                                 │
│           │ ciphertext: "A3f8x9Kp2mN..."                                    │
│           ▼                                                                 │
│  ┌─────────────────┐                                                        │
│  │ Enhanced Client │                                                        │
│  │ Driver          │  ← Decrypts AFTER receiving from DB                    │
│  └────────┬────────┘                                                        │
│           │                                                                 │
│           │ plaintext: "john@example.com"                                   │
│           ▼                                                                 │
│  [Application]                                                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Encryption Modes Covered

| Mode | Description | Implementation |
|------|-------------|----------------|
| **Data at Rest** | Encrypted when stored on disk | AES-256-GCM with random nonce |
| **Data in Transit** | Protected during network transfer | TLS (PostgreSQL SSL) + encrypted payload |
| **Data in Use** | Protected in application memory | Keys zeroed after use (Rust ownership) |

## Quick Start

### 1. Start PostgreSQL

```bash
# Using Docker
make db-start

# Or use existing PostgreSQL
export DATABASE_URL="postgres://user:pass@localhost/mydb"
```

### 2. Configure Encryption Keys

```bash
cp .env.example .env
# Edit .env with your master password
```

### 3. Run Demo

```bash
make demo
```

## Usage Examples

### Initialize Database
```bash
cargo run -- init
```

### Create User with Encrypted Data
```bash
cargo run -- create \
  --username john_doe \
  --email "john@example.com" \
  --ssn "123-45-6789" \
  --phone "+1-555-0123" \
  --address "123 Main St, City, ST 12345"
```

### Retrieve User (Auto-Decrypted)
```bash
cargo run -- get --id 1
```

### View Raw Encrypted Data
```bash
cargo run -- show-encrypted --id 1
```

## Security Features

### What's Encrypted
- Email addresses
- Social Security Numbers (SSN)
- Phone numbers
- Physical addresses

### What's NOT Encrypted (Searchable)
- Username (needed for lookups)
- ID (primary key)
- Timestamps

### Encryption Details
- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: Argon2id (memory-hard, resistant to GPU attacks)
- **Nonce**: 12 bytes, randomly generated per encryption
- **Storage Format**: Base64(nonce || ciphertext || auth_tag)

## Production Considerations

### Key Management
Replace password-based key derivation with a proper KMS:

```rust
// AWS KMS example
use aws_sdk_kms::Client;

async fn get_master_key(kms: &Client, key_id: &str) -> MasterKey {
    let response = kms
        .generate_data_key()
        .key_id(key_id)
        .key_spec(DataKeySpec::Aes256)
        .send()
        .await?;
    
    MasterKey::from_bytes(&response.plaintext.unwrap())
}
```

### Key Rotation
1. Decrypt all data with old key
2. Re-encrypt with new key
3. Update key reference in secure storage

### Audit Logging
Add logging for all encryption/decryption operations:

```rust
tracing::info!(
    user_id = %id,
    operation = "decrypt",
    columns = "email,ssn,phone,address",
    "Sensitive data accessed"
);
```

## Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Project Structure

```
pg-encrypted-client/
├── Cargo.toml
├── Makefile
├── .env.example
├── README.md
└── src/
    ├── main.rs        # CLI application
    ├── lib.rs         # Library exports
    ├── crypto.rs      # Encryption service (Enhanced Client Driver)
    └── repository.rs  # Database operations with encryption
```

## Comparison with SQL Server Always Encrypted

| Feature | SQL Server | This Implementation |
|---------|------------|---------------------|
| Key Hierarchy | DPAPI → CMK → CEK | Password → Master Key → CEK |
| Encryption Location | Client driver | Rust application |
| Algorithm | AES-256 | AES-256-GCM |
| Secure Enclave | Yes (optional) | No (future: SGX support) |
| Query on Encrypted | Deterministic encryption | Not supported |

## License

MIT
