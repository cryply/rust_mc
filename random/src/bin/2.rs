


use rand::prelude::*;

fn main(){

    let mut rng = rand::rng();
    let x = vec![2,3,4];

    println!("{}", x.choose(&mut rng).unwrap());
}