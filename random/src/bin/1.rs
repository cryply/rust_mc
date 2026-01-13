use rand::prelude::*;

fn main(){
    let mut rng = rand::rng();

    let x:u128 = rng.random();
    println!("{x}");

    let x = vec![1,2,3,4,5,6];

    println!("{:?}", x.choose(&mut rng).unwrap());

}