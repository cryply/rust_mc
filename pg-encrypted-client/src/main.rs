//! PostgreSQL Encrypted Client
//!
//! Implements "Always Encrypted" pattern for PostgreSQL in Rust.
//! Based on the architecture shown in the lecture slides:
//!
//! 1. Key Hierarchy (Slide 1):
//!    - Master Key (like DPAPI/Service Master Key)
//!    - Column Encryption Key (like Database Encryption Key)
//!
//! 2. Always Encrypted (Slide 2):
//!    - Client-side encryption before INSERT
//!    - Client-side decryption after SELECT
//!    - Database never sees plaintext
//!
//! 3. Data in Use (Slide 3):
//!    - Enhanced Client Driver handles crypto
//!    - Plaintext ↔ Ciphertext conversion at client

mod crypto;
mod repository;

use crate::crypto::{ColumnEncryptionKey, EncryptedClientDriver, MasterKey};
use crate::repository::{CreateUserInput, UpdateUserInput, UserRepository};
use clap::{Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Parser)]
#[command(name = "pg-encrypted-client")]
#[command(about = "PostgreSQL client with Always Encrypted support")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database schema
    Init,

    /// Create a new user
    Create {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        email: String,
        #[arg(long)]
        ssn: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        address: Option<String>,
    },

    /// Get user by ID
    Get {
        #[arg(short, long)]
        id: i32,
    },

    /// List all users
    List,

    /// Update a user
    Update {
        #[arg(short, long)]
        id: i32,
        #[arg(short, long)]
        email: Option<String>,
        #[arg(long)]
        ssn: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        address: Option<String>,
    },

    /// Delete a user
    Delete {
        #[arg(short, long)]
        id: i32,
    },

    /// Show raw encrypted data for a user (demonstration)
    ShowEncrypted {
        #[arg(short, long)]
        id: i32,
    },

    /// Demo: Create user and show encryption
    Demo,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pg_encrypted_client=debug".parse()?),
        )
        .init();

    // Load environment
    dotenvy::dotenv().ok();

    // Get configuration
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/encrypted_demo".to_string());

    let master_password = env::var("MASTER_KEY_PASSWORD")
        .unwrap_or_else(|_| "demo-master-password-change-in-production".to_string());

    let master_salt = env::var("MASTER_KEY_SALT")
        .unwrap_or_else(|_| "saltsaltsaltsalt".to_string());

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    // Initialize encryption 
    // In production, load master key from secure storage (AWS KMS, HashiCorp Vault, etc.)
    let master_key = MasterKey::from_password(&master_password, master_salt.as_bytes())?;

    // Derive Column Encryption Key for the 'users' table sensitive columns
    let cek = ColumnEncryptionKey::derive(&master_key, "users.sensitive_columns")?;

    // Create the Enhanced Client Driver (from slide 3)
    let driver = EncryptedClientDriver::new(&cek);

    // Create repository
    let repo = UserRepository::new(pool, driver);

    // Parse CLI and execute
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            repo.initialize().await?;
            println!("✓ Database initialized successfully");
        }

        Commands::Create {
            username,
            email,
            ssn,
            phone,
            address,
        } => {
            let user = repo
                .create(CreateUserInput {
                    username,
                    email,
                    ssn,
                    phone,
                    address,
                })
                .await?;

            println!("✓ User created successfully:");
            println!("{}", serde_json::to_string_pretty(&user)?);
        }

        Commands::Get { id } => {
            let user = repo.get_by_id(id).await?;
            println!("{}", serde_json::to_string_pretty(&user)?);
        }

        Commands::List => {
            let users = repo.list_all().await?;
            println!("{}", serde_json::to_string_pretty(&users)?);
        }

        Commands::Update {
            id,
            email,
            ssn,
            phone,
            address,
        } => {
            let user = repo
                .update(
                    id,
                    UpdateUserInput {
                        email,
                        ssn,
                        phone,
                        address,
                    },
                )
                .await?;

            println!("✓ User updated successfully:");
            println!("{}", serde_json::to_string_pretty(&user)?);
        }

        Commands::Delete { id } => {
            let deleted = repo.delete(id).await?;
            if deleted {
                println!("✓ User {} deleted", id);
            } else {
                println!("✗ User {} not found", id);
            }
        }

        Commands::ShowEncrypted { id } => {
            let raw = repo.get_raw_encrypted(id).await?;
            println!("Raw encrypted data stored in PostgreSQL:");
            println!("─────────────────────────────────────────");
            println!("ID:              {}", raw.id);
            println!("Username:        {}", raw.username);
            println!("Encrypted Email: {}", raw.encrypted_email);
            println!(
                "Encrypted SSN:   {}",
                raw.encrypted_ssn.as_deref().unwrap_or("NULL")
            );
            println!(
                "Encrypted Phone: {}",
                raw.encrypted_phone.as_deref().unwrap_or("NULL")
            );
            println!(
                "Encrypted Addr:  {}",
                raw.encrypted_address.as_deref().unwrap_or("NULL")
            );
            println!("Created At:      {}", raw.created_at);
        }

        Commands::Demo => {
            println!("╔═══════════════════════════════════════════════════════════════╗");
            println!("║     PostgreSQL Always Encrypted Demo (Rust Implementation)    ║");
            println!("╚═══════════════════════════════════════════════════════════════╝");
            println!();

            // Initialize
            println!("1️⃣  Initializing database schema...");
            repo.initialize().await?;
            println!("   ✓ Schema created\n");

            // Create user
            println!("2️⃣  Creating user with sensitive data...");
            println!("   Plaintext input:");
            println!("   - Email:   john.doe@example.com");
            println!("   - SSN:     123-45-6789");
            println!("   - Phone:   +1-555-0123");
            println!("   - Address: 123 Main St, City, ST 12345");

            let user = repo
                .create(CreateUserInput {
                    username: format!("demo_user_{}", chrono::Utc::now().timestamp()),
                    email: "john.doe@example.com".to_string(),
                    ssn: Some("123-45-6789".to_string()),
                    phone: Some("+1-555-0123".to_string()),
                    address: Some("123 Main St, City, ST 12345".to_string()),
                })
                .await?;

            println!("   ✓ User created with ID: {}\n", user.id);

            // Show encrypted data
            println!("3️⃣  Raw data stored in PostgreSQL (what DB admin sees):");
            let raw = repo.get_raw_encrypted(user.id).await?;
            println!("   ┌─────────────────────────────────────────────────────────────┐");
            println!("   │ encrypted_email: {}...", &raw.encrypted_email[..40]);
            println!(
                "   │ encrypted_ssn:   {}...",
                &raw.encrypted_ssn.as_ref().unwrap()[..40]
            );
            println!(
                "   │ encrypted_phone: {}...",
                &raw.encrypted_phone.as_ref().unwrap()[..40]
            );
            println!("   └─────────────────────────────────────────────────────────────┘");
            println!("   ⚠️  Database only stores ciphertext - no plaintext visible!\n");

            // Show decrypted data
            println!("4️⃣  Decrypted data (what client application sees):");
            let decrypted = repo.get_by_id(user.id).await?;
            println!("   ┌─────────────────────────────────────────────────────────────┐");
            println!("   │ email:   {}", decrypted.email);
            println!("   │ ssn:     {}", decrypted.ssn.as_deref().unwrap_or("N/A"));
            println!(
                "   │ phone:   {}",
                decrypted.phone.as_deref().unwrap_or("N/A")
            );
            println!(
                "   │ address: {}",
                decrypted.address.as_deref().unwrap_or("N/A")
            );
            println!("   └─────────────────────────────────────────────────────────────┘\n");

            // Architecture summary
            println!("═══════════════════════════════════════════════════════════════════");
            println!("Architecture:");
            println!();
            println!("  [Client App]  ←→  [Enhanced Client Driver]  ←→  [PostgreSQL]");
            println!("   plaintext         encrypt/decrypt              ciphertext");
            println!();
            println!("Key Hierarchy:");
            println!("  Master Key (from password/KMS) → Column Encryption Key → AES-256-GCM");
            println!("═══════════════════════════════════════════════════════════════════");

            // Cleanup
            repo.delete(user.id).await?;
            println!("\n✓ Demo user cleaned up");
        }
    }

    Ok(())
}
