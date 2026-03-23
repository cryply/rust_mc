use std::time::{Duration, Instant};
use tokio::time;

async fn run_behavior(
    label: &str,
    behavior: time::MissedTickBehavior,
    slow_work_ms: u64,
) {
    println!("\n=== {} ===", label);

    let start = Instant::now();
    let mut interval = time::interval(Duration::from_millis(50));
    interval.set_missed_tick_behavior(behavior);
    interval.tick().await; // consume first immediate tick

    for i in 0..5 {
        interval.tick().await;
        let elapsed = start.elapsed().as_millis();
        println!("  tick {} at t={}ms", i, elapsed);

        // Simulate slow work on first iteration only
        if i == 0 {
            println!("  [slow work starting for {}ms]", slow_work_ms);
            time::sleep(Duration::from_millis(slow_work_ms)).await;
            println!("  [slow work done at t={}ms]", start.elapsed().as_millis());
        }
    }
}

#[tokio::main]
async fn main() {
    run_behavior("BURST  (catch up)", time::MissedTickBehavior::Burst, 120).await;
    run_behavior("DELAY  (reset from now)", time::MissedTickBehavior::Delay, 120).await;
    run_behavior("SKIP   (keep original grid)", time::MissedTickBehavior::Skip, 120).await;
}
