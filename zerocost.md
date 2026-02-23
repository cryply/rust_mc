## Iterators
## Zero Cost 

### Generators
1. iter(&x)-> &T
2. iter_mut(&x) -> &mut T
3. into_iter(x) -> T
4. (i, &x) = enumerate() 
5. rev() -> DoubleEndedIterator
6. cycle()
7. range(0..10)
8. once(x) //use std::iter; iter::once(&314)
9. repeat(x)

### Duplicators
1. cloned()
2. copied()

###  Adapters(Transformers)
1. flatten()
2. (x,y) = partition(|x|) -> (Vec<T>, Vec<T>)
3. for_each(|&x|) // consumes 
4. map(|x|) // moved , transform items
5. filter_map(|&x|) //return Option()., drop Nones.
6. flat_map() - maps then flatten
7. chain(x.iter()) // concat
8. zip() 
9. reduce()  / like fold returns Option() if empty.
10. scan(init_state, |state, item| {})
11. chunks() // reverse to flatten.
12. chunks_exact()
13. (a,b) = cloned().unzip()
14. fused()
15. by_ref()


### Filters
1. filter(|&&x|)
2. cycle()
3. all() // -> bool
4. any() // -> bool
5. nth() // consume up to n, returns Option() - Jumping ahead.
6. skip()
7. skip_while()
8. take() // yield first n elements, returns Iterator(range)
9. step_by()
10. min(), max() //itertools
11. find() // first element
12. position()
13. last()
14. min_by(|&&a, &&b|, a.cmp(b)), max_by()
15. next()
16. take_while()
17. unique()
18. dedup() 


### Accumulators
1. fold(0, |acc, &x| acc + x)
2. sum()
3. product()
4. count()
5. reduce()
6, max_by(), min_by()

### Debug
1. inspect(|x| operator()) // hook for logging.
2. peekable, peek // no consume, reference.


### output.
1. collect()
2. collect_into()
3. try_into()
4. into_boxed_slice()
5. into_boxed_str()
6. for_each()
7. to_vec()
8. to_string()


repeat_n
repeat_with
empty
from_fn
successors
intersperse let mut a = [0, 1, 2].into_iter().intersperse(100);
map_while


Rustlings, 100 Exercises To Learn Rust 


Specialization
    
Generators/coroutines
    
Async generators/coroutines
    
Try blocks 	
Never type 	
Trait aliases
    
Type Alias Impl Trait (TAIT) 	
Associated type defaults
    
Generic const expressions
    
Const trait methods
    
Declarative (macro_rules!) attributes (#[attr]) and derives (#[derive(Trait)]) 	
Compile time reflection 	
Variadic generics 	
Arbitrary self types
    
Enum variant types 	
Allocator trait and better OOM handling
    
Stable ABI 	
Portable SIMD
Strict provenance API (1.84) 	
Async closures (1.85) 	
diagnostic::do_not_recommend (1.85) 	
Trait upcasting (1.86) 	
Anonymous pipes (1.87) 	
Let chains (1.88) 	
Naked functions (1.88)

 This Week in Rust, Official  Rust blog, account Rust in  Bluesky 

Discuss Rust in official Zulip or Discord

Open issue in any repo of  rust-lang на GitHub


users.rust-lang.org, internals.rust-lang.org, официальный сервер Rust Discord или Rust Zulip

 reddit.com/r/rust, Hacker News, Rust Community Discord, 

Rust Conference


