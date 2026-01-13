use std::{arch::x86_64, sync::{Arc, Mutex, mpsc}};

fn main(){
    let fork1 = mpsc::channel::<u32>();
    let fork2 = mpsc::channel::<u32>();

    let rh1 = Arc::new(Mutex::new(fork1.1));
    let rh2 = Arc::new(Mutex::new(fork2.1));

    let th1 = Arc::new(Mutex::new(fork1.0));
    let th2 = Arc::new(Mutex::new(fork2.0));

    
    let hp: Vec<_> = (0..2).map(|i| {
        let rh1c = rh1.clone();
        std::thread::spawn(move ||{
            let myreader = rh1c.lock().unwrap();
            let read_val = myreader.recv().unwrap();
            println!("Client{i} got: {read_val}");
        })

    }).collect(); 



         let th1c = th1.clone();   
    let hw1 = std::thread::spawn(move ||{

        let my_writer = th1c.lock().unwrap();
        my_writer.send(12).unwrap();
   
    });
         let th1c = th1.clone();
    let hw2 = std::thread::spawn(move ||{

        let my_writer = th1c.lock().unwrap();
        my_writer.send(123).unwrap();
   
    });

    hw1.join().unwrap();
    hw2.join().unwrap();
    for h in hp {
        h.join().unwrap();
    }

}