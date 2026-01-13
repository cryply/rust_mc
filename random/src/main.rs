
use rand::prelude::*;
use std::collections::HashMap;


fn main() {

    let mut rng = rand::rng();

    let ri:i32 = rng.random();
    println!("i32:{}", ri);

    let ru:u32 = rng.random();
    println!("u32:{}", ru);

    let ri:bool = rng.random();
    println!("bool:{}", ri);

    let ru:f32 = rng.random();
    println!("f32:{}", ru);

    let die_roll = rng.random_range(1..=6);
    println!("Die cast:{}", die_roll);

    let races = vec!["elves", "orcs", "humans", "undead"];
    let random_race = races[rng.random_range(0..races.len())];
    println!("random race:{}", random_race);
    println!("random race2:{}", races.choose(&mut rng).unwrap());

}
