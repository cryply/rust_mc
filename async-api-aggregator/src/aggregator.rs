//! Core aggregator module

use futures::future::join_all;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{info, instrument, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API returned error status {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

// ============================================================================
// API Response Types (using JSONPlaceholder as example)
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Post {
    pub id: u32,
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: u32,
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Comment {
    pub id: u32,
    #[serde(rename = "postId")]
    pub post_id: u32,
    pub name: String,
    pub email: String,
    pub body: String,
}

// ============================================================================
// Aggregated Result
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AggregatedData {
    pub users: Vec<User>,
    pub posts: Vec<Post>,
    pub todos: Vec<Todo>,
    pub comments: Vec<Comment>,
    pub fetch_stats: FetchStats,
}

#[derive(Debug, Serialize)]
pub struct FetchStats {
    pub total_requests: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_duration_ms: u128,
}

// ============================================================================
// API Client
// ============================================================================

pub struct ApiAggregator {
    client: Client,
    base_url: String,
}

impl ApiAggregator {
    /// Create a new API aggregator with configured client
    pub fn new(base_url: &str, timeout_secs: u64) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// Generic fetch function for any deserializable type
    #[instrument(skip(self), fields(url = %url))]
    async fn fetch<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, ApiError> {
        info!("Fetching from {}", url);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(ApiError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        response.json::<T>().await.map_err(ApiError::from)
    }

    /// Fetch users from the API
    pub async fn fetch_users(&self) -> Result<Vec<User>, ApiError> {
        let url = format!("{}/users", self.base_url);
        self.fetch(&url).await
    }

    /// Fetch posts from the API
    pub async fn fetch_posts(&self) -> Result<Vec<Post>, ApiError> {
        let url = format!("{}/posts", self.base_url);
        self.fetch(&url).await
    }

    /// Fetch todos from the API
    pub async fn fetch_todos(&self) -> Result<Vec<Todo>, ApiError> {
        let url = format!("{}/todos", self.base_url);
        self.fetch(&url).await
    }

    /// Fetch comments from the API
    pub async fn fetch_comments(&self) -> Result<Vec<Comment>, ApiError> {
        let url = format!("{}/comments", self.base_url);
        self.fetch(&url).await
    }

    // ========================================================================
    // Aggregation Strategies
    // ========================================================================

    /// Strategy 1: Fetch all concurrently, fail if ANY request fails
    /// Use when all data is required
    #[instrument(skip(self))]
    pub async fn fetch_all_or_nothing(&self) -> Result<AggregatedData, ApiError> {
        let start = std::time::Instant::now();

        // tokio::join! runs all futures concurrently and returns a tuple
        // This is the idiomatic way when futures return different types
        let (users, posts, todos, comments) = tokio::join!(
            self.fetch_users(),
            self.fetch_posts(),
            self.fetch_todos(),
            self.fetch_comments(),
        );

        let duration = start.elapsed();

        // Propagate any errors with ?
        Ok(AggregatedData {
            users: users?,
            posts: posts?,
            todos: todos?,
            comments: comments?,
            fetch_stats: FetchStats {
                total_requests: 4,
                successful: 4,
                failed: 0,
                total_duration_ms: duration.as_millis(),
            },
        })
    }

    /// Strategy 2: Fetch all concurrently, collect partial results
    /// Use when partial data is acceptable
    #[instrument(skip(self))]
    pub async fn fetch_best_effort(&self) -> AggregatedData {
        let start = std::time::Instant::now();

        let (users, posts, todos, comments) = tokio::join!(
            self.fetch_users(),
            self.fetch_posts(),
            self.fetch_todos(),
            self.fetch_comments(),
        );

        let duration = start.elapsed();

        let mut successful = 0;
        let mut failed = 0;

        let users = users
            .inspect(|_| successful += 1)
            .inspect_err(|e| {
                warn!("Failed to fetch users: {}", e);
                failed += 1;
            })
            .unwrap_or_default();

        let posts = posts
            .inspect(|_| successful += 1)
            .inspect_err(|e| {
                warn!("Failed to fetch posts: {}", e);
                failed += 1;
            })
            .unwrap_or_default();

        let todos = todos
            .inspect(|_| successful += 1)
            .inspect_err(|e| {
                warn!("Failed to fetch todos: {}", e);
                failed += 1;
            })
            .unwrap_or_default();

        let comments = comments
            .inspect(|_| successful += 1)
            .inspect_err(|e| {
                warn!("Failed to fetch comments: {}", e);
                failed += 1;
            })
            .unwrap_or_default();

        AggregatedData {
            users,
            posts,
            todos,
            comments,
            fetch_stats: FetchStats {
                total_requests: 4,
                successful,
                failed,
                total_duration_ms: duration.as_millis(),
            },
        }
    }

    /// Strategy 3: Fetch from multiple URLs of the same type concurrently
    /// Useful for paginated APIs or multiple similar endpoints
    #[instrument(skip(self, urls))]
    pub async fn fetch_many<T>(&self, urls: Vec<String>) -> Vec<Result<T, ApiError>>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let futures: Vec<_> = urls.iter().map(|url| self.fetch::<T>(url)).collect();

        join_all(futures).await
    }

    /// Strategy 4: Fetch with controlled concurrency (rate limiting)
    /// Useful when APIs have rate limits
    #[instrument(skip(self, urls))]
    pub async fn fetch_with_concurrency_limit<T>(
        &self,
        urls: Vec<String>,
        max_concurrent: usize,
    ) -> Vec<Result<T, ApiError>>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        stream::iter(urls)
            .map(|url| async move { self.fetch::<T>(&url).await })
            .buffer_unordered(max_concurrent)
            .collect()
            .await
    }
}

// ============================================================================
// Derived Data Processing
// ============================================================================

impl AggregatedData {
    /// Get posts by a specific user
    pub fn posts_by_user(&self, user_id: u32) -> Vec<&Post> {
        self.posts.iter().filter(|p| p.user_id == user_id).collect()
    }

    /// Get completion rate for todos by user
    pub fn todo_completion_rate(&self, user_id: u32) -> f64 {
        let user_todos: Vec<_> = self.todos.iter().filter(|t| t.user_id == user_id).collect();
        if user_todos.is_empty() {
            return 0.0;
        }
        let completed = user_todos.iter().filter(|t| t.completed).count();
        completed as f64 / user_todos.len() as f64
    }

    /// Get comments for a specific post
    pub fn comments_for_post(&self, post_id: u32) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.post_id == post_id)
            .collect()
    }

    /// Summary report
    pub fn summary(&self) -> String {
        format!(
            "Aggregated Data Summary:\n\
             - Users: {}\n\
             - Posts: {}\n\
             - Todos: {} ({} completed)\n\
             - Comments: {}\n\
             - Fetch time: {}ms ({} successful, {} failed)",
            self.users.len(),
            self.posts.len(),
            self.todos.len(),
            self.todos.iter().filter(|t| t.completed).count(),
            self.comments.len(),
            self.fetch_stats.total_duration_ms,
            self.fetch_stats.successful,
            self.fetch_stats.failed,
        )
    }
}
