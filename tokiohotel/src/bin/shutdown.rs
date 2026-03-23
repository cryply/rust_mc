use std::time::Duration;
use tokio::time;
use tokio_util::sync::CancellationToken;

// Worker that does periodic work until cancelled
async fn worker(id: u32, token: CancellationToken) {
    let mut interval = time::interval(Duration::from_millis(100));
    let mut count = 0;

    loop {
        tokio::select! {
            // cancelled() is cancel-safe — can be used in select! across iterations
            _ = token.cancelled() => {
                println!("worker {}: shutting down after {} ticks", id, count);
                // Do cleanup here (flush buffers, close connections, etc.)
                return;
            }
            _ = interval.tick() => {
                count += 1;
                println!("worker {}: tick {}", id, count);
            }
        }
    }
}

// Child tokens — cancelling parent cancels children, but not vice versa.
// Useful for scoped task groups within a larger system.
async fn child_token_demo() {
    let parent = CancellationToken::new();
    let child = parent.child_token();

    let h1 = tokio::spawn({
        let token = parent.clone();
        async move {
            token.cancelled().await;
            println!("parent-listener: cancelled");
        }
    });

    let h2 = tokio::spawn({
        let token = child.clone();
        async move {
            token.cancelled().await;
            println!("child-listener: cancelled");
        }
    });

    // Cancelling parent cascades to child
    parent.cancel();
    h1.await.unwrap();
    h2.await.unwrap();
    println!("child is_cancelled: {}", child.is_cancelled()); // true
}

// drop_guard — automatically cancels token when dropped.
// Ensures cleanup even if the owning task panics.
async fn drop_guard_demo() {
    let token = CancellationToken::new();

    let h = tokio::spawn({
        let token = token.clone();
        async move {
            token.cancelled().await;
            println!("drop_guard: got cancelled");
        }
    });

    {
        // guard cancels the token when it goes out of scope
        let _guard = token.drop_guard();
        println!("drop_guard: guard in scope");
    } // _guard dropped here → token cancelled

    h.await.unwrap();
}

#[tokio::main]
async fn main() {
    println!("--- basic shutdown ---");
    let token = CancellationToken::new();

    // Start workers
    let h1 = tokio::spawn(worker(1, token.clone()));
    let h2 = tokio::spawn(worker(2, token.clone()));

    // Let them run briefly, then signal shutdown
    time::sleep(Duration::from_millis(350)).await;
    println!("main: sending shutdown signal");
    token.cancel();

    // Wait for workers to finish cleanup
    h1.await.unwrap();
    h2.await.unwrap();

    println!("\n--- child tokens ---");
    child_token_demo().await;

    println!("\n--- drop guard ---");
    drop_guard_demo().await;
}
