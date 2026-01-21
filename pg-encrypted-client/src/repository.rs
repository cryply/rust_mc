//! Database models and repository
//!
//! This layer handles database operations while ensuring all sensitive
//! data is encrypted before storage and decrypted after retrieval.

use crate::crypto::{CryptoError, EncryptedClientDriver};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;

/// Repository errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Encryption error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("User not found: {0}")]
    NotFound(i32),
}

/// Raw database row - contains encrypted data
/// This is what's actually stored in PostgreSQL
#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: i32,
    pub username: String,
    pub encrypted_email: String,
    pub encrypted_ssn: Option<String>,
    pub encrypted_phone: Option<String>,
    pub encrypted_address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Decrypted user data - what the application works with
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub ssn: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Input for creating a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
    pub ssn: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

/// Input for updating user
#[derive(Debug, Deserialize)]
pub struct UpdateUserInput {
    pub email: Option<String>,
    pub ssn: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

/// User Repository - handles all database operations with encryption
///
/// This implements the "Always Encrypted" pattern from slide 2:
/// - Client encrypts data BEFORE sending to database
/// - Database only ever sees ciphertext
/// - Client decrypts data AFTER receiving from database
pub struct UserRepository {
    pool: PgPool,
    driver: EncryptedClientDriver,
}

impl UserRepository {
    pub fn new(pool: PgPool, driver: EncryptedClientDriver) -> Self {
        Self { pool, driver }
    }

    /// Initialize the database schema
    pub async fn initialize(&self) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                encrypted_email TEXT NOT NULL,
                encrypted_ssn TEXT,
                encrypted_phone TEXT,
                encrypted_address TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index on username (the only searchable field)
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)
            "#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Database schema initialized");
        Ok(())
    }

    /// Create a new user - encrypts all sensitive fields client-side
    pub async fn create(&self, input: CreateUserInput) -> Result<User, RepositoryError> {
        tracing::debug!("Creating user: {}", input.username);

        // Encrypt all sensitive fields BEFORE sending to database
        let encrypted_email = self.driver.encrypt(&input.email)?;
        let encrypted_ssn = self.driver.encrypt_optional(input.ssn.as_deref())?;
        let encrypted_phone = self.driver.encrypt_optional(input.phone.as_deref())?;
        let encrypted_address = self.driver.encrypt_optional(input.address.as_deref())?;

        let row: UserRow = sqlx::query_as(
            r#"
            INSERT INTO users (username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address, created_at
            "#,
        )
        .bind(&input.username)
        .bind(&encrypted_email)
        .bind(&encrypted_ssn)
        .bind(&encrypted_phone)
        .bind(&encrypted_address)
        .fetch_one(&self.pool)
        .await?;

        // Decrypt for return
        self.decrypt_row(row)
    }

    /// Get user by ID - decrypts all sensitive fields client-side
    pub async fn get_by_id(&self, id: i32) -> Result<User, RepositoryError> {
        let row: UserRow = sqlx::query_as(
            r#"
            SELECT id, username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound(id))?;

        self.decrypt_row(row)
    }

    /// Get user by username
    pub async fn get_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(self.decrypt_row(r)?)),
            None => Ok(None),
        }
    }

    /// List all users (with decryption)
    pub async fn list_all(&self) -> Result<Vec<User>, RepositoryError> {
        let rows: Vec<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address, created_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| self.decrypt_row(r)).collect()
    }

    /// Update user - re-encrypts any changed sensitive fields
    pub async fn update(&self, id: i32, input: UpdateUserInput) -> Result<User, RepositoryError> {
        // First get current row
        let current = self.get_by_id(id).await?;

        // Determine new values, encrypting as needed
        let new_email = match input.email {
            Some(email) => self.driver.encrypt(&email)?,
            None => self.driver.encrypt(&current.email)?,
        };

        let new_ssn = match input.ssn {
            Some(ssn) => self.driver.encrypt_optional(Some(&ssn))?,
            None => self.driver.encrypt_optional(current.ssn.as_deref())?,
        };

        let new_phone = match input.phone {
            Some(phone) => self.driver.encrypt_optional(Some(&phone))?,
            None => self.driver.encrypt_optional(current.phone.as_deref())?,
        };

        let new_address = match input.address {
            Some(address) => self.driver.encrypt_optional(Some(&address))?,
            None => self.driver.encrypt_optional(current.address.as_deref())?,
        };

        let row: UserRow = sqlx::query_as(
            r#"
            UPDATE users
            SET encrypted_email = $2,
                encrypted_ssn = $3,
                encrypted_phone = $4,
                encrypted_address = $5
            WHERE id = $1
            RETURNING id, username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address, created_at
            "#,
        )
        .bind(id)
        .bind(&new_email)
        .bind(&new_ssn)
        .bind(&new_phone)
        .bind(&new_address)
        .fetch_one(&self.pool)
        .await?;

        self.decrypt_row(row)
    }

    /// Delete user by ID
    pub async fn delete(&self, id: i32) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Helper to decrypt a row
    fn decrypt_row(&self, row: UserRow) -> Result<User, RepositoryError> {
        Ok(User {
            id: row.id,
            username: row.username,
            email: self.driver.decrypt(&row.encrypted_email)?,
            ssn: self.driver.decrypt_optional(row.encrypted_ssn.as_deref())?,
            phone: self.driver.decrypt_optional(row.encrypted_phone.as_deref())?,
            address: self.driver.decrypt_optional(row.encrypted_address.as_deref())?,
            created_at: row.created_at,
        })
    }

    /// Get raw encrypted data (for demonstration/debugging)
    pub async fn get_raw_encrypted(&self, id: i32) -> Result<UserRow, RepositoryError> {
        sqlx::query_as(
            r#"
            SELECT id, username, encrypted_email, encrypted_ssn, encrypted_phone, encrypted_address, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound(id))
    }
}
