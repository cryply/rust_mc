#![feature(mpmc_channel)]
use std::sync::Mutex;

fn threads_demo() {
    use std::thread;

    let handle = thread::spawn(|| {
        println!("Dis is von thread:");
    });

    handle.join().unwrap();
}

fn mpsc_demo() {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let msg = String::from("Polo");
        tx.send(msg).unwrap();
    });

    let received = rx.recv().unwrap();
    println!("Marco: {}", received);
}

fn mutex_demo() {
    use std::sync::Mutex;

    let data = Mutex::new(99);
    {
        let mut data_access = data.lock().unwrap();
        *data_access += 1;
    }

    println!("data: {:?}", *data.lock().unwrap());
}

fn arc_demo() {
    use std::sync::Arc;
    use std::thread;

    let data = Arc::new(5);

    // Ar cprovides RO access. Use Arc<Mutex> for internal mutability, See below.

    for _ in 0..5 {
        let data_shared = data.clone();
        thread::spawn(move || {
            println!("{:?}", data_shared);
        });
    }

    use std::sync::atomic::{AtomicI32, Ordering};

    let counter = Arc::new(AtomicI32::new(0));
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let counter_clone = Arc::clone(&counter);

            thread::spawn(move || {
                for _ in 0..100 {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "Atomic I32 Final value: {}",
        counter.load(Ordering::Relaxed)
    );

    let counter = Arc::new(Mutex::new(0));

    let counter_clone = Arc::clone(&counter);
    {
        let mut num = counter_clone.lock().unwrap();
        *num += 123;
        // counter_clone.lock().unwrap() += 123;
    }

    let value = *counter.lock().unwrap();
    println!("Arc Mutex Counter:{}", value);
}

use rayon::prelude::*;

fn rayon_demo() {
    let data = vec![1, 2, 3, 4];

    let parallel_sum: i32 = data.par_iter().map(|x| x * x).sum();

    println!("Rayon Parallel sum: {}", parallel_sum);
}

fn mpmc_demo() {
    use std::sync::mpmc;
    use std::thread;

    let (tx, rx) = mpmc::channel::<String>();

    // Multiple producers — tx is Clone
    let producer_handles: Vec<_> = (0..3)
        .map(|i| {
            let tx = tx.clone();
            thread::spawn(move || {
                tx.send(format!("msg from producer {}", i)).unwrap();
            })
        })
        .collect();

    drop(tx); // close channel when all producers finish

    // Multiple consumers — rx is Clone in mpmc
    let consumer_handles: Vec<_> = (0..3)
        .map(|i| {
            let rx = rx.clone();
            thread::spawn(move || {
                while let Ok(val) = rx.recv() {
                    println!("mpmc: consumer {} got: {}", i, val);
                }
            })
        })
        .collect();

    for h in producer_handles {
        h.join().unwrap();
    }
    for h in consumer_handles {
        h.join().unwrap();
    }
}

async fn broadcast_demo() {
    use tokio::sync::broadcast;

    let (tx, _) = broadcast::channel::<String>(16);

    // Each subscriber gets its own rx — ALL receive every message
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let mut rx = tx.subscribe();
            tokio::spawn(async move {
                while let Ok(val) = rx.recv().await {
                    println!("tokio bcast: subscriber {} got: {}", i, val);
                }
            })
        })
        .collect();

    // Single producer — every message fans out to all subscribers
    for i in 0..3 {
        tx.send(format!("broadcast msg {}", i)).unwrap();
    }
    drop(tx);

    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    threads_demo();
    mpsc_demo();
    mpmc_demo();
    mutex_demo();
    arc_demo();
    broadcast_demo().await;

    rayon_demo();
}
