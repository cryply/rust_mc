//! Async API Aggregator - Binary Entry Point
//!
//! Demonstrates concurrent fetching from multiple REST APIs.

use async_api_aggregator::{ApiAggregator, ApiError, User};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting API Aggregator");

    // Create aggregator pointing to JSONPlaceholder (free test API)
    let aggregator = ApiAggregator::new("https://jsonplaceholder.typicode.com", 30)?;

    // Demo 1: Fetch all or nothing
    info!("=== Strategy 1: All or Nothing ===");
    match aggregator.fetch_all_or_nothing().await {
        Ok(data) => {
            println!("{}", data.summary());
            println!("\nFirst user: {:?}", data.users.first());
        }
        Err(e) => eprintln!("Failed to fetch all data: {}", e),
    }

    println!("\n{}", "=".repeat(60));

    // Demo 2: Best effort (partial results)
    info!("=== Strategy 2: Best Effort ===");
    let data = aggregator.fetch_best_effort().await;
    println!("{}", data.summary());

    // Demo derived data
    if let Some(user) = data.users.first() {
        let user_posts = data.posts_by_user(user.id);
        let completion_rate = data.todo_completion_rate(user.id);
        println!(
            "\nUser '{}' has {} posts and {:.1}% todo completion rate",
            user.name,
            user_posts.len(),
            completion_rate * 100.0
        );
    }

    println!("\n{}", "=".repeat(60));

    // Demo 3: Fetch multiple URLs with concurrency limit
    info!("=== Strategy 3: Rate-Limited Concurrent Fetch ===");
    let user_urls: Vec<String> = (1..=5)
        .map(|id| format!("https://jsonplaceholder.typicode.com/users/{}", id))
        .collect();

    let results: Vec<Result<User, ApiError>> = aggregator
        .fetch_with_concurrency_limit(user_urls, 2) // Max 2 concurrent
        .await;

    println!("Fetched {} individual users:", results.len());
    for result in results {
        match result {
            Ok(user) => println!("  ✓ {} ({})", user.name, user.email),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    }

    Ok(())
}
