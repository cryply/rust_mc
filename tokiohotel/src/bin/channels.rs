use tokio::sync::{broadcast, mpsc, oneshot, watch};

// mpsc — multiple producer, single consumer (most common)
async fn mpsc_demo() {
    // Bounded channel — backpressure when buffer is full (sender.send().await blocks)
    let (tx, mut rx) = mpsc::channel::<String>(32);

    for i in 0..3 {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(format!("msg {}", i)).await.unwrap();
        });
    }
    drop(tx); // channel closes when all senders dropped

    // recv() returns None when channel is closed
    while let Some(msg) = rx.recv().await {
        println!("mpsc got: {}", msg);
    }
}

// oneshot — single value, single use. Perfect for request/response.
async fn oneshot_demo() {
    let (tx, rx) = oneshot::channel::<String>();

    tokio::spawn(async move {
        // Simulate some work, then send result back
        tx.send("here's your answer".to_string()).unwrap();
    });

    // rx.await consumes the receiver — can only receive once
    let result = rx.await.unwrap();
    println!("oneshot got: {}", result);
}

// broadcast — every subscriber gets every message
async fn broadcast_demo() {
    let (tx, _) = broadcast::channel::<String>(16);

    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();

    tx.send("hello everyone".to_string()).unwrap();
    // Both receivers get the same message
    println!("rx1: {}", rx1.recv().await.unwrap());
    println!("rx2: {}", rx2.recv().await.unwrap());

    // If a receiver is slow, messages are dropped with RecvError::Lagged(n)
}

// watch — only keeps the latest value. Receivers see the most recent state.
async fn watch_demo() {
    let (tx, mut rx) = watch::channel("initial");

    tokio::spawn(async move {
        tx.send("updated").unwrap();
        tx.send("final").unwrap();
    });

    // changed() waits until the value differs from what we last saw
    rx.changed().await.unwrap();
    // borrow() gives a Ref to the current value
    println!("watch current value: {}", *rx.borrow());
}

#[tokio::main]
async fn main() {
    mpsc_demo().await;
    oneshot_demo().await;
    broadcast_demo().await;
    watch_demo().await;
}
