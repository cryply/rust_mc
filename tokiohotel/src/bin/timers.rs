use std::time::Duration;
use tokio::time::{self, Instant};

// sleep — non-blocking delay. The task yields, freeing the thread for other work.
async fn sleep_demo() {
    println!("sleeping 100ms...");
    let start = Instant::now();
    time::sleep(Duration::from_millis(100)).await;
    println!("woke up after {:?}", start.elapsed());
}

// interval — fires at fixed rate. If your work takes longer than the period,
// the next tick fires immediately (burst mode) to catch up.
// Use MissedTickBehavior to change this.
async fn interval_demo() {
    let mut interval = time::interval(Duration::from_millis(50));

    // First tick completes immediately
    for i in 0..4 {
        interval.tick().await;
        println!("tick {} at {:?}", i, Instant::now());
    }

    // MissedTickBehavior options:
    //   Burst (default) — catch up immediately
    //   Delay — reset the interval after slow work
    //   Skip — drop missed ticks
    let mut interval = time::interval(Duration::from_millis(50));
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
    interval.tick().await; // consume first immediate tick

    for i in 0..3 {
        interval.tick().await;
        println!("delay-mode tick {}", i);
        // Simulate slow work on first iteration
        if i == 0 {
            time::sleep(Duration::from_millis(120)).await;
        }
    }
}

// timeout — wraps any future with a deadline
async fn timeout_demo() {
    // Fast task — completes in time
    let result = time::timeout(Duration::from_millis(100), async {
        time::sleep(Duration::from_millis(10)).await;
        "fast"
    })
    .await;
    println!("fast task: {:?}", result); // Ok("fast")

    // Slow task — exceeds deadline
    let result = time::timeout(Duration::from_millis(50), async {
        time::sleep(Duration::from_millis(200)).await;
        "slow"
    })
    .await;
    println!("slow task: {:?}", result); // Err(Elapsed)

    // Pattern: handle timeout gracefully
    match time::timeout(Duration::from_millis(50), some_work()).await {
        Ok(val) => println!("got: {}", val),
        Err(_) => println!("timed out, using fallback"),
    }
}

async fn some_work() -> &'static str {
    time::sleep(Duration::from_millis(200)).await;
    "result"
}

#[tokio::main]
async fn main() {
    sleep_demo().await;
    interval_demo().await;
    timeout_demo().await;
}
