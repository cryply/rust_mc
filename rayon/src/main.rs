/* Extend this example */
use rayon::prelude::*;
use std::time::{Duration, Instant};
use rand::{Rng, rng};

fn main() {
    let mut data = vec![1, 2, 3];

    let mut rng = rand::rng();

    for _ in 0..1_000_000_0 {
        data.push(rng.random_range(1..=6));
    }

    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    println!("Num threads: {}", rayon::current_num_threads());

    let start = Instant::now();
    let parallel_sum: i32 = data.par_iter() // Specify the type
        .map(|x| x * x)
        .sum();

    println!("Parallel sum: {} elapsed: {:?}", parallel_sum, start.elapsed());

    let start = Instant::now();
    let sum: i32 = data.iter() // Specify the type
        .map(|x| x * x)
        .sum();

    println!("sum: {} elapsed: {:?}", parallel_sum, start.elapsed());


}
