use rand::prelude::*;

fn main(){
    println!("{}", [1,2,3].choose(&mut rand::rng()).unwrap());
}