trait Counter {
    fn next(&mut self) -> u64;

    // default implemenation
    fn next_n(&mut self, n: usize) -> Vec<u64> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.next());
        }
        out
    }
}
struct Inc {
    cur: u64,
}
impl Counter for Inc {
    fn next(&mut self) -> u64 {
        self.cur += 1;
        self.cur
    }
}
fn main() {
    let mut c = Inc { cur: 0 };
    println!("{:?}", c.next_n(5));
}
