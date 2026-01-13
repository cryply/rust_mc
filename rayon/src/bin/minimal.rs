use rayon::prelude::*;

fn main(){
    let data = vec![1,2,3,4,5];


    let sum : Vec<_>= data.par_iter().map(|x| x * 2).collect();

    println!("{:?}", sum);

}