use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

// Basic select — first future to complete wins, others are cancelled
async fn basic_select() {
    tokio::select! {
        _ = time::sleep(Duration::from_millis(100)) => {
            println!("100ms timer fired first");
        }
        _ = time::sleep(Duration::from_millis(200)) => {
            println!("200ms timer fired first"); // never reached
        }
    }
}

// select on channels — common pattern for multiplexing inputs
async fn channel_select() {
    let (tx1, mut rx1) = mpsc::channel::<&str>(1);
    let (tx2, mut rx2) = mpsc::channel::<&str>(1);

    tokio::spawn(async move {
        time::sleep(Duration::from_millis(50)).await;
        tx1.send("from channel 1").await.unwrap();
    });

    tokio::spawn(async move {
        time::sleep(Duration::from_millis(30)).await;
        tx2.send("from channel 2").await.unwrap();
    });

    // Whichever channel delivers first
    tokio::select! {
        Some(msg) = rx1.recv() => println!("rx1: {}", msg),
        Some(msg) = rx2.recv() => println!("rx2: {}", msg),
    }
}

// select in a loop — typical event loop pattern
async fn loop_select() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);

    tokio::spawn(async move {
        for i in 0..3 {
            tx.send(i).await.unwrap();
            time::sleep(Duration::from_millis(30)).await;
        }
        // tx drops here, closing the channel
    });

    let deadline = time::sleep(Duration::from_millis(200));
    // pin! is required because select! takes &mut references to futures,
    // and sleep is used across multiple loop iterations
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            Some(val) = rx.recv() => {
                println!("loop got: {}", val);
            }
            // Use &mut to avoid consuming the future on first iteration
            _ = &mut deadline => {
                println!("deadline reached, exiting loop");
                break;
            }
        }
    }
}

// biased select — checks branches in order instead of randomly.
// Useful when you want to prioritize one source (e.g., drain a channel before checking shutdown).
async fn biased_select() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);
    for i in 0..3 {
        tx.send(i).await.unwrap();
    }
    drop(tx);

    let mut count = 0;
    loop {
        tokio::select! {
            // biased; makes select check branches top-to-bottom
            // Without it, branches are polled in random order each iteration
            biased;

            Some(val) = rx.recv() => {
                println!("biased got: {}", val);
                count += 1;
            }
            _ = time::sleep(Duration::from_millis(10)) => {
                println!("channel drained after {} msgs", count);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    basic_select().await;
    channel_select().await;
    loop_select().await;
    biased_select().await;
}
