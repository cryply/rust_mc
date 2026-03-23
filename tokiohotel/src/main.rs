// #[tokio::main] expands roughly to:
//   fn main() { tokio::runtime::Runtime::new().unwrap().block_on(async { ... }) }
// It creates a multi-threaded runtime by default.
// Use #[tokio::main(flavor = "current_thread")] for single-threaded.

#[tokio::main]
async fn main() {
    // spawn returns a JoinHandle — a future that resolves to the task's return value.
    // Tasks are scheduled on the runtime's thread pool (not 1:1 with OS threads).
    let handle = tokio::spawn(async {
        // This runs on a worker thread, concurrently with main
        42
    });

    // .await the JoinHandle to get the result.
    // Returns Result<T, JoinError> — JoinError if the task panicked or was cancelled.
    let result = handle.await.unwrap();
    println!("spawned task returned: {}", result);

    // spawn multiple tasks and collect results
    let handles: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                // async move captures i by value — same as closures
                println!("task {} running on {:?}", i, std::thread::current().id());
                i * 10
            })
        })
        .collect();

    for h in handles {
        let val = h.await.unwrap();
        println!("got: {}", val);
    }

    // yield_now — cooperatively gives up the time slice to other tasks
    tokio::task::yield_now().await;

    // spawn_blocking — runs a closure on a dedicated thread pool for blocking work.
    // Use this for CPU-heavy or synchronous I/O that would block the async runtime.
    let blocking_result = tokio::task::spawn_blocking(|| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        "done with blocking work"
    })
    .await
    .unwrap();
    println!("{}", blocking_result);
}
