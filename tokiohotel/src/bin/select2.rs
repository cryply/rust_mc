use tokio::time;
use std::time::Duration;
use tokio::select;


#[tokio::main]
async fn main(){
    select!{
        _ = time::sleep(Duration::from_millis(100)) => println!("100"),
        
        _ = time::sleep(Duration::from_millis(20)) => println!("200"),

    }
}
