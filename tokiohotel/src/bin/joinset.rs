use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time;

// Basic JoinSet — spawn tasks, collect results in completion order
async fn basic_joinset() {
    let mut set = JoinSet::new();

    for i in 0..5 {
        set.spawn(async move {
            // Different sleep times — results arrive out of order
            time::sleep(Duration::from_millis(50 * (5 - i))).await;
            i * 10
        });
    }

    // join_next() returns results in completion order (fastest first)
    while let Some(result) = set.join_next().await {
        println!("completed: {}", result.unwrap());
    }

}

// Abort all — dropping JoinSet cancels remaining tasks
async fn abort_demo() {
    let mut set = JoinSet::new();

    for i in 0..5 {
        set.spawn(async move {
            time::sleep(Duration::from_millis(100 * i)).await;
            println!("task {} finished", i);
            i
        });
    }

    // Take just the first 2 results, then abort the rest
    for _ in 0..2 {
        let result = set.join_next().await.unwrap().unwrap();
        println!("got: {}", result);
    }

    // abort_all() cancels remaining tasks
    set.abort_all();
    println!("aborted remaining {} tasks", set.len());

    // Drain to clean up — aborted tasks return JoinError
    while let Some(result) = set.join_next().await {
        match result {
            Ok(val) => println!("late finish: {}", val),
            Err(_) => println!("task was cancelled"),
        }
    }
}

// Bounded concurrency — JoinSet + Semaphore
// Common pattern: process 100 items but max 3 in flight
async fn bounded_concurrency() {
    let sem = Arc::new(Semaphore::new(3));
    let mut set = JoinSet::new();

    for i in 0..10 {
        let sem = Arc::clone(&sem);
        set.spawn(async move {
            // Acquire permit before doing work
            let _permit = sem.acquire().await.unwrap();
            println!("processing item {} (permits left: {})", i, sem.available_permits());
            time::sleep(Duration::from_millis(50)).await;
            i
        });
    }

    while let Some(result) = set.join_next().await {
        println!("done: {}", result.unwrap());
    }
}

// JoinSet with spawn_on — pin task to a specific runtime handle
// JoinSet with join_all (1.83+) — await all at once instead of looping
async fn join_all_demo() {
    let mut set = JoinSet::new();

    for i in 0..3 {
        set.spawn(async move {
            time::sleep(Duration::from_millis(10)).await;
            i
        });
    }

    // join_all() waits for everything and returns Vec of results
    let results: Vec<i32> = set.join_all().await;
    println!("all results: {:?}", results);
}

#[tokio::main]
async fn main() {
    println!("--- basic joinset ---");
    basic_joinset().await;

    println!("\n--- abort demo ---");
    abort_demo().await;

    println!("\n--- bounded concurrency ---");
    bounded_concurrency().await;

    println!("\n--- join_all ---");
    join_all_demo().await;
}
