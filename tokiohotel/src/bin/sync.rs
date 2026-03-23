use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Barrier, Mutex, Notify, RwLock, Semaphore};
use tokio::time;

// Mutex — use when lock must be held across .await points.
// For short critical sections without .await, prefer std::sync::Mutex (less overhead).
async fn mutex_demo() {
    let data = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let data = Arc::clone(&data);
            tokio::spawn(async move {
                // .lock().await — yields if locked, doesn't block the thread
                let mut vec = data.lock().await;
                vec.push(i);
                // lock is held across this await — std::Mutex would deadlock here
                time::sleep(Duration::from_millis(1)).await;
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }
    println!("mutex result: {:?}", *data.lock().await);
}

// RwLock — multiple concurrent readers, exclusive writer
async fn rwlock_demo() {
    let data = Arc::new(RwLock::new(0));

    // Spawn readers — all can run concurrently
    let readers: Vec<_> = (0..3)
        .map(|i| {
            let data = Arc::clone(&data);
            tokio::spawn(async move {
                let val = data.read().await;
                println!("reader {} sees: {}", i, *val);
            })
        })
        .collect();

    // Writer — blocks until all readers release
    {
        let mut val = data.write().await;
        *val = 42;
        println!("writer set value to {}", *val);
    }

    for r in readers {
        r.await.unwrap();
    }
}

// Semaphore — limit concurrency. Common for rate limiting, connection pools.
async fn semaphore_demo() {
    // Only 2 tasks can run concurrently
    let sem = Arc::new(Semaphore::new(2));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let sem = Arc::clone(&sem);
            tokio::spawn(async move {
                // acquire() returns a permit — dropped when guard goes out of scope
                let _permit = sem.acquire().await.unwrap();
                println!("task {} got permit (available: {})", i, sem.available_permits());
                time::sleep(Duration::from_millis(50)).await;
                // permit released here
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }
}

// Notify — wake waiting tasks. Like a condvar but for async.
async fn notify_demo() {
    let notify = Arc::new(Notify::new());

    let notify2 = Arc::clone(&notify);
    let waiter = tokio::spawn(async move {
        println!("waiter: waiting for notification...");
        notify2.notified().await;
        println!("waiter: got notified!");
    });

    time::sleep(Duration::from_millis(50)).await;
    println!("main: sending notification");
    notify.notify_one(); // notify_waiters() wakes ALL

    waiter.await.unwrap();
}

// Barrier — all tasks must arrive before any can proceed.
// Useful for phased computation where all workers must finish step N before starting step N+1.
async fn barrier_demo() {
    let barrier = Arc::new(Barrier::new(3));

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
                println!("task {} doing prep work", i);
                time::sleep(Duration::from_millis(i as u64 * 30)).await;

                println!("task {} waiting at barrier", i);
                let result = barrier.wait().await;
                // One task is designated the "leader" — useful for single-run cleanup
                if result.is_leader() {
                    println!("task {} is the leader", i);
                }
                println!("task {} passed barrier", i);
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    mutex_demo().await;
    rwlock_demo().await;
    semaphore_demo().await;
    notify_demo().await;
    barrier_demo().await;
}
