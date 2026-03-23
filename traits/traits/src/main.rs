


// This is trait bounds.
fn print_debug<T: std::fmt::Debug>(x: T) {
    println!("{:?}", x);
}

// This is trait bounds.
fn print_debug_red<T: std::fmt::Debug>(x: &T) {
    println!("{:?}", x);
}

fn show1<T: std::fmt::Debug + std::fmt::Display>(x: &T) {
    println!("display:{} debug{:?}", x, x);
}

fn process<T>(x: T)
where 
    T: std::fmt::Debug + Clone + Send,
{
    println!("{:?}", x.clone());    
}


fn main() {

    show1(&1);
    process("data.lock().unwrap()");
}
