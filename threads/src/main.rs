use std::{clone, cmp::Ordering, sync::Mutex};

fn threads_demo() {
    use std::thread;

    let handle = thread::spawn(||{
        println!("Dis is von thread:");

    });

    handle.join().unwrap();
}

fn mpsc_demo(){
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move|| {
        let msg = String::from("Polo");
        tx.send(msg).unwrap();
    });

    let received = rx.recv().unwrap();
    println!("Marco: {}", received);    
} 

fn mutex_demo(){
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

    for _ in 0..5 {
        let data_shared = data.clone();
        thread::spawn(move ||{
            println!("{:?}", data_shared);
        });
    }

    use std::sync::atomic::{AtomicI32, Ordering};

    let counter = Arc::new(AtomicI32::new(0));
    let handles: Vec<_> =  (0..5)
    .map( |_| {
        
        let counter_clone = Arc::clone(&counter);

        thread::spawn(move ||{
            for _ in 0..100 {
                counter_clone.fetch_add(1, Ordering::Relaxed);
                
            }
        })
    })
    .collect();


    for handle in handles {
        handle.join().unwrap();
    }

    println!("Atomic I32 Final value: {}", counter.load(Ordering::Relaxed));



    let counter = Arc::new(Mutex::new(0));

    let counter_clone = Arc::clone(&counter);
    {
        let mut num = counter_clone.lock().unwrap();
        *num += 123;
    }


    let value = *counter.lock().unwrap();
    println!("Arc Mutex Counter:{}", value);
}



use rayon::prelude::*;

fn rayon_demo() {
    let data = vec![1,2,3,4];

    let parallel_sum: i32 = data.par_iter()
        .map(|x| x*x)
        .sum();

    println!("Rayon Parallel sum: {}", parallel_sum);
    
}


fn main() {
    threads_demo();
    mpsc_demo();
    mutex_demo();
    arc_demo();

    rayon_demo();
}


