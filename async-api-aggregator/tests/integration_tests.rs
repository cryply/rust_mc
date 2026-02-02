//! Integration tests for async-api-aggregator
//!
//! These tests hit the live JSONPlaceholder API.
//! Run with: cargo test -- --nocapture

use async_api_aggregator::*;

// ============================================================================
// API Fetch Tests
// ============================================================================

#[tokio::test]
async fn test_fetch_users() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let users = aggregator.fetch_users().await.unwrap();

    assert!(!users.is_empty());
    assert_eq!(users.len(), 10); // JSONPlaceholder has 10 users
}

#[tokio::test]
async fn test_fetch_posts() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let posts = aggregator.fetch_posts().await.unwrap();

    assert!(!posts.is_empty());
    assert_eq!(posts.len(), 100); // JSONPlaceholder has 100 posts
}

#[tokio::test]
async fn test_fetch_todos() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let todos = aggregator.fetch_todos().await.unwrap();

    assert!(!todos.is_empty());
    assert_eq!(todos.len(), 200); // JSONPlaceholder has 200 todos
}

#[tokio::test]
async fn test_fetch_comments() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let comments = aggregator.fetch_comments().await.unwrap();

    assert!(!comments.is_empty());
    assert_eq!(comments.len(), 500); // JSONPlaceholder has 500 comments
}

// ============================================================================
// Aggregation Strategy Tests
// ============================================================================

#[tokio::test]
async fn test_fetch_all_or_nothing() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let data = aggregator.fetch_all_or_nothing().await.unwrap();

    assert!(!data.users.is_empty());
    assert!(!data.posts.is_empty());
    assert!(!data.todos.is_empty());
    assert!(!data.comments.is_empty());
    assert_eq!(data.fetch_stats.successful, 4);
    assert_eq!(data.fetch_stats.failed, 0);
}

#[tokio::test]
async fn test_fetch_best_effort() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let data = aggregator.fetch_best_effort().await;

    assert!(!data.users.is_empty());
    assert!(!data.posts.is_empty());
    assert!(data.fetch_stats.successful > 0);
}

#[tokio::test]
async fn test_fetch_with_concurrency_limit() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();

    let urls: Vec<String> = (1..=5)
        .map(|id| format!("https://jsonplaceholder.typicode.com/users/{}", id))
        .collect();

    let results: Vec<Result<User, ApiError>> = aggregator
        .fetch_with_concurrency_limit(urls, 2)
        .await;

    assert_eq!(results.len(), 5);

    let successful: Vec<_> = results.into_iter().filter_map(Result::ok).collect();
    assert_eq!(successful.len(), 5);
}

// ============================================================================
// Derived Data Tests
// ============================================================================

#[tokio::test]
async fn test_posts_by_user() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let data = aggregator.fetch_best_effort().await;

    // User 1 should have posts
    let user_1_posts = data.posts_by_user(1);
    assert!(!user_1_posts.is_empty());
    assert!(user_1_posts.iter().all(|p| p.user_id == 1));
}

#[tokio::test]
async fn test_todo_completion_rate() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let data = aggregator.fetch_best_effort().await;

    // Check completion rate is between 0 and 1
    let rate = data.todo_completion_rate(1);
    assert!((0.0..=1.0).contains(&rate));
}

#[tokio::test]
async fn test_comments_for_post() {
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30).unwrap();
    let data = aggregator.fetch_best_effort().await;

    // Post 1 should have comments
    let comments = data.comments_for_post(1);
    assert!(!comments.is_empty());
    assert!(comments.iter().all(|c| c.post_id == 1));
}

// ============================================================================
// Unit Tests (no network)
// ============================================================================

#[test]
fn test_aggregated_data_summary() {
    let data = AggregatedData {
        users: vec![User {
            id: 1,
            name: "Test User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
        }],
        posts: vec![Post {
            id: 1,
            user_id: 1,
            title: "Test Post".into(),
            body: "Content".into(),
        }],
        todos: vec![
            Todo {
                id: 1,
                user_id: 1,
                title: "Done task".into(),
                completed: true,
            },
            Todo {
                id: 2,
                user_id: 1,
                title: "Pending task".into(),
                completed: false,
            },
        ],
        comments: vec![],
        fetch_stats: FetchStats {
            total_requests: 4,
            successful: 4,
            failed: 0,
            total_duration_ms: 100,
        },
    };

    let summary = data.summary();
    assert!(summary.contains("Users: 1"));
    assert!(summary.contains("Posts: 1"));
    assert!(summary.contains("Todos: 2 (1 completed)"));
    assert!(summary.contains("100ms"));
}

#[test]
fn test_empty_user_completion_rate() {
    let data = AggregatedData {
        users: vec![],
        posts: vec![],
        todos: vec![],
        comments: vec![],
        fetch_stats: FetchStats {
            total_requests: 0,
            successful: 0,
            failed: 0,
            total_duration_ms: 0,
        },
    };

    // Non-existent user should return 0.0
    let rate = data.todo_completion_rate(999);
    assert_eq!(rate, 0.0);
}

#[test]
fn test_api_aggregator_creation() {
    let aggregator = ApiAggregator::new("https://example.com", 30);
    assert!(aggregator.is_ok());
}
