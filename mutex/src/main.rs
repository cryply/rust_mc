

use std::thread;
use std::sync::{Mutex, Arc, RwLock, Condvar};


/*
| Aspect          | Mutex                         | RwLock                                           |
| --------------- | ----------------------------- | ------------------------------------------------ |
| Concurrency     | Single thread only            | Multiple readers or single writer doc.rust-lang​ |
| Use Case        | Write-heavy or balanced       | Read-heavy workloads redandgreen​                |
| Overhead        | Lower, simpler stackoverflow​ | Higher due to reader tracking redandgreen​       |
| Starvation Risk | None                          | Possible writer starvation stackoverflow​        |
| Trait Req       | T: Send                       | T: Sync + Send stackoverflow​                    |

*/

fn arc_mutex() {
    let  data = Arc::new(Mutex::new(vec![1, 2, 3]));

    let mut handles = vec![];


    for i in 0..3 {
        let data = Arc::clone(&data);        
        let  handle = thread::spawn( move || {
            let mut v = data.lock().unwrap();
            v[i] += 10;
        });

        handles.push(handle);
    }


    for  handle in handles {
        handle.join().unwrap();
    }

    println!("{:?}", data.lock().unwrap());

    // No data race can occur, this will not compile.
}

fn arc_rwlock() {
    let  data = Arc::new(RwLock::new(vec![1, 2, 3]));

    let mut handles = vec![];


    for i in 0..3 {
        let data = Arc::clone(&data);        
        let  handle = thread::spawn( move || {
            let mut v = data.write().unwrap();
            v[i] += 10;
        });

        handles.push(handle);
    }


    for  handle in handles {
        handle.join().unwrap();
    }

    println!("{:?}", data.read().unwrap());

    // No data race can occur, this will not compile.
}


fn cond_vars(){

    let data = Arc::new(RwLock::new(0));
    let flag = Arc::new(Mutex::new(false));
    let condvar = Arc::new(Condvar::new());

    let data_clone = Arc::clone(&data);
    let flag_clone = Arc::clone(&flag);
    let condvar_clone = Arc::clone(&condvar);

    // Поток, который ждёт изменения
    let reader = thread::spawn(move || {
        let mut flag_guard = flag_clone.lock().unwrap();
        while !*flag_guard {
            flag_guard = condvar_clone.wait(flag_guard).unwrap();
        }
        let data_guard = data_clone.read().unwrap();
        println!("Data is ready: {}", *data_guard);
    });

    // Поток, который изменяет данные и уведомляет
    let writer = thread::spawn(move || {
        let mut data_guard = data.write().unwrap();
        *data_guard = 42;
        let mut flag_guard = flag.lock().unwrap();
        *flag_guard = true;
        condvar.notify_all();
    });

    reader.join().unwrap();
    writer.join().unwrap();

}

fn main() {

    arc_mutex();


    arc_rwlock();
    
    cond_vars();

    // use std::sync::Mutex;

    // let m = Mutex::new(6);
    // {
    //     let mut num = m.lock().unwrap();

    //     *num = 1000;
    // }

    // println!("Hello, world! {:?}", m);
}
ip a
