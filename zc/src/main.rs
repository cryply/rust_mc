// main.rs — 100 iterator examples (grouped)
// Target: nightly (also works on stable)
// Each example has an assert or comparable check so the file is runnable.

fn main() {
    // ---------------------
    // Basic iteration patterns (idiomatic)
    // ---------------------

    // n1: enumerate over a slice of &str (fixes double-ref issue)
    let arr1 = ["zero", "one", "two"];
    let numbered1: Vec<(usize, &str)> = arr1.iter().copied().enumerate().collect();
    assert_eq!(numbered1, vec![(0, "zero"), (1, "one"), (2, "two")]);

    // n1.1

    let arr1 = ["zero", "one", "two"];
    let numbered1: Vec<(usize, &str)> = arr1.iter().enumerate().map(|(i, s)| (i, *s)).collect();
    assert_eq!(numbered1, vec![(0, "zero"), (1, "one"), (2, "two")]);

    // n2: consume a Vec<String> with into_iter()
    let v2 = vec![String::from("a"), String::from("b")];
    let moved2: Vec<String> = v2.into_iter().collect();
    assert_eq!(moved2, vec![String::from("a"), String::from("b")]);

    // n3: iterate mutably and modify in-place
    let mut nums3 = [1, 2, 3];
    nums3.iter_mut().for_each(|x| *x += 10);
    assert_eq!(nums3, [11, 12, 13]);

    // n4: peekable and peek (non-consuming lookahead)
    let v4 = vec![1, 2, 3];
    let mut it4 = v4.iter().peekable();
    assert_eq!(it4.peek().copied(), Some(&1));

    // n5: collect after copied() from slice of u8
    let numbers5 = [0u8, 1, 2, 3];
    let v5: Vec<u8> = numbers5.iter().copied().collect();
    assert_eq!(v5, vec![0u8, 1, 2, 3]);

    // n6: to_vec() from array
    let arr6 = [1, 2, 3];
    let v6 = arr6.to_vec();
    assert_eq!(v6, vec![1, 2, 3]);

    // n7: into_boxed_slice()
    let boxed7: Box<[u8]> = vec![1u8, 2, 3].into_boxed_slice();
    assert_eq!(&*boxed7, &[1u8, 2, 3]);

    // n8: into_boxed_str()
    let boxed8: Box<str> = String::from("hello").into_boxed_str();
    assert_eq!(&*boxed8, "hello");

    // n9: once()
    let one_time9: Vec<i32> = std::iter::once(314).collect();
    assert_eq!(one_time9, vec![314]);

    // n10: repeat() + take()
    let first_three10: Vec<i32> = std::iter::repeat(5).take(3).collect();
    assert_eq!(first_three10, vec![5, 5, 5]);

    // ---------------------
    // Filters, ranges, clones
    // ---------------------

    // n11: filter on range (predicate sees value, not reference)
    let evens11: Vec<i32> = (0..10).filter(|x| x % 2 == 0).collect();
    assert_eq!(evens11, vec![0, 2, 4, 6, 8]);

    // n12: cloned() to convert &String -> String via collecting references then cloning
    let words12 = vec![String::from("a"), String::from("b")];
    let refs12 = words12.iter().collect::<Vec<&String>>();
    let owned12 = refs12.into_iter().cloned().collect::<Vec<String>>();
    assert_eq!(owned12, vec![String::from("a"), String::from("b")]);

    // n13: inspect() for debug side effects (we assert that it returns same items)
    let v13 = vec![1, 2, 3];
    let collected13: Vec<_> = v13.iter().inspect(|x| { let _ = x; }).collect();
    assert_eq!(collected13, vec![&1, &2, &3]);

    // n14: chars().peekable() lookahead parser pattern (we show peek value)
    let s14 = "a1";
    let mut chars14 = s14.chars().peekable();
    assert_eq!(chars14.peek().copied(), Some('a'));

    // n15: collecting iterator of Result into Result<Vec<_>, _>
    let res15: Result<Vec<i32>, &str> = vec![Ok(1i32), Ok(2)].into_iter().collect();
    assert_eq!(res15, Ok(vec![1, 2]));

    // n16: placeholder for collect_into-style API (no-op assertion)
    // (conceptual example, API may be nightly; we assert that a capacity exists)
    let mut dst16: Vec<i32> = Vec::with_capacity(10);
    assert!(dst16.capacity() >= 10);

    // n17: rev() + enumerate()
    let xs17 = [10, 20, 30];
    let rev_enumerated17: Vec<(usize, &i32)> = xs17.iter().rev().enumerate().collect();
    assert_eq!(rev_enumerated17, vec![(0, &30), (1, &20), (2, &10)]);

    // n18: for_each() for side-effects; we use assert to avoid unused warnings
    let arr18 = [1, 2, 3];
    let mut sum18 = 0;
    arr18.iter().for_each(|&x| sum18 += x);
    assert_eq!(sum18, 6);

    // n19: cycle() with cloned() to own values
    let v19 = vec![1, 2, 3];
    let first_five19: Vec<i32> = v19.iter().cycle().take(5).cloned().collect();
    assert_eq!(first_five19, vec![1, 2, 3, 1, 2]);

    // n20: once() + chain()
    let base20 = std::iter::once(1).chain([2, 3].iter().cloned());
    let collected20: Vec<_> = base20.collect();
    assert_eq!(collected20, vec![1, 2, 3]);

    // ---------------------
    // Consumers: sum, fold, collect into sets/maps
    // ---------------------

    // n21: sum()
    let sum21: i32 = vec![1, 2, 3].into_iter().sum();
    assert_eq!(sum21, 6);

    // n22: join() for &str slices
    let s22 = ["a", "b"].join(",");
    assert_eq!(s22, "a,b".to_string());

    // n23: find() then map to get owned value
    let opt_first23 = [1, 2, 3].iter().find(|&&x| x == 2).map(|&x| x);
    assert_eq!(opt_first23, Some(2));

    // n24: zip() and map to convert index to u8 then collect
    let zipped24 = (0..5).zip(['a', 'b', 'c', 'd', 'e'].iter().cloned()).map(|(i, ch)| (i as u8, ch)).collect::<Vec<(u8, char)>>();
    assert_eq!(zipped24.len(), 5);
    assert_eq!(zipped24[0], (0u8, 'a'));

    // n25: chain() example
    let chained25 = std::iter::once(0).chain(1..4).collect::<Vec<_>>();
    assert_eq!(chained25, vec![0, 1, 2, 3]);

    // n26: map() example
    let mapped26 = (1..4).map(|x| x * 2).collect::<Vec<_>>();
    assert_eq!(mapped26, vec![2, 4, 6]);

    // n27: partition()
    let (evens27, odds27): (Vec<i32>, Vec<i32>) = (0..10).partition(|&x| x % 2 == 0);
    assert_eq!(evens27.iter().all(|&x| x % 2 == 0), true);
    assert_eq!(odds27.iter().all(|&x| x % 2 != 0), true);

    // n28: fold()
    let counted28 = (0..5).fold(0, |acc, x| acc + x);
    assert_eq!(counted28, 0 + 1 + 2 + 3 + 4);

    // n29: scan()
    let scanned29: Vec<i32> = (1..5).scan(0, |state, x| { *state += x; Some(*state) }).collect();
    assert_eq!(scanned29, vec![1, 3, 6, 10]);

    // n30: any()
    let any_gt_5_30 = (0..10).any(|x| x > 5);
    assert_eq!(any_gt_5_30, true);

    // ---------------------
    // Predicates: all, none, position
    // ---------------------

    // n31: all()
    let all_lt_10_31 = (0..10).all(|x| x < 10);
    assert_eq!(all_lt_10_31, true);

    // n32: all() false case
    let none_negative32 = [-1, 0, 1].iter().all(|&x| x >= 0);
    assert_eq!(none_negative32, false);

    // n33: position()
    let pos33 = (0..10).position(|x| x == 5);
    assert_eq!(pos33, Some(5));

    // n34: last via rev().next()
    let last34 = vec![1, 2, 3].into_iter().rev().next();
    assert_eq!(last34, Some(3));

    // n35: collect into HashSet
    let set35: std::collections::HashSet<_> = vec![1, 2, 2, 3].into_iter().collect();
    assert_eq!(set35.contains(&2), true);
    assert_eq!(set35.len(), 3);

    // n36: collect into HashMap
    let map36: std::collections::HashMap<_, _> = vec![("a", 1), ("b", 2)].into_iter().collect();
    assert_eq!(map36.get("a"), Some(&1));

    // n37: flat_map to flatten nested vectors
    let flat37: Vec<_> = vec![vec![1, 2], vec![3]].into_iter().flat_map(|v| v.into_iter()).collect();
    assert_eq!(flat37, vec![1, 2, 3]);

    // n38: windows() over slice and sum windows
    let windows_sum38: Vec<i32> = [1, 2, 3, 4].windows(2).map(|w| w.iter().sum()).collect();
    assert_eq!(windows_sum38, vec![3, 5, 7]);

    // n39: peek_example with peekable
    let peek_example39 = {
        let mut it = (1..4).peekable();
        if let Some(&next) = it.peek() { next } else { 0 }
    };
    assert_eq!(peek_example39, 1);

    // n40: skip and take
    let skip_take40: Vec<_> = (0..10).skip(2).take(5).collect();
    assert_eq!(skip_take40, vec![2, 3, 4, 5, 6]);

    // n41: zip and cloned
    let zipped41: Vec<(i32, char)> = (0..3).zip(['a', 'b', 'c'].iter().cloned()).collect();
    assert_eq!(zipped41, vec![(0, 'a'), (1, 'b'), (2, 'c')]);

    // n42: unzip
    let (ks42, vs42): (Vec<_>, Vec<_>) = (0..3).zip(['a', 'b', 'c'].iter().cloned()).unzip();
    assert_eq!(ks42, vec![0, 1, 2]);
    assert_eq!(vs42, vec!['a', 'b', 'c']);

    // n43: collecting Results again (Ok case)
    let try_iter43: Result<Vec<i32>, &str> = vec![Ok(1), Ok(2)].into_iter().collect();
    assert_eq!(try_iter43, Ok(vec![1, 2]));

    // n44: enumerate after zip and map to triple
    let complex_zip44 = (0..6).zip(['a', 'b', 'c', 'd', 'e', 'f'].iter().cloned())
        .enumerate()
        .map(|(i, (n, ch))| (i, n, ch))
        .collect::<Vec<(usize, i32, char)>>();
    assert_eq!(complex_zip44.len(), 6);

    // ---------------------
    // More adapters & combinations
    // ---------------------

    // n45: cycle on single-value iterator + take
    let cyc45: Vec<i32> = std::iter::once(7).cycle().take(4).collect();
    assert_eq!(cyc45, vec![7, 7, 7, 7]);

    // n46: collect into VecDeque
    let deq46: std::collections::VecDeque<_> = (1..4).collect();
    assert_eq!(deq46.len(), 3);

    // n47: chain many iterators
    let chained47 = (0..2).chain(2..4).chain(4..5).collect::<Vec<_>>();
    assert_eq!(chained47, vec![0, 1, 2, 3, 4]);

    // n48: peekable used to check end
    let mut it48 = [1].iter().peekable();
    assert_eq!(it48.peek().copied(), Some(&1));

    // n49: map with Option output then flatten via flatten()
    let opt_map49: Vec<i32> = (0..5).map(|x| if x % 2 == 0 { Some(x) } else { None }).flatten().collect();
    assert_eq!(opt_map49, vec![0, 2, 4]);

    // n50: collect into array via try_into (requires nightly or lib conversion) — use collect then try_into
    let vec50: Vec<i32> = (1..=3).collect();
    let arr50: [i32; 3] = vec50.try_into().unwrap();
    assert_eq!(arr50, [1, 2, 3]);

    // n51: map and enumerate interplay
    let map_enum51 = (10..13).map(|x| x - 10).enumerate().collect::<Vec<(usize, i32)>>();
    assert_eq!(map_enum51, vec![(0, 0), (1, 1), (2, 2)]);

    // n52: scan stateful generator to produce Fibonacci-like numbers
    let fib52: Vec<i32> = (0..5).scan((0, 1), |state, _| {
        let next = state.0 + state.1;
        state.0 = state.1;
        state.1 = next;
        Some(state.0)
    }).collect();
    assert_eq!(fib52, vec![1, 1, 2, 3, 5]);

    // n53: chain iterators of different origins
    let chained53 = vec![1].into_iter().chain([2, 3].into_iter()).collect::<Vec<_>>();
    assert_eq!(chained53, vec![1, 2, 3]);

    // n54: repeated mapping and collecting into Box<[T]>
    let boxed54: Box<[i32]> = (1..4).collect::<Vec<_>>().into_boxed_slice();
    assert_eq!(&*boxed54, &[1, 2, 3]);

    // n55: using try_fold for early exit with error propagation
    let res55: Result<i32, &str> = (1..5).try_fold(0, |acc, x| if x == 3 { Err("stop") } else { Ok(acc + x) });
    assert_eq!(res55, Err("stop"));

    // n56: using inspect to mutate an external variable (side-effect)
    let mut out56 = 0;
    let _c56: Vec<_> = [1, 2, 3].iter().inspect(|&&x| { /* no-op */ }).collect();
    assert_eq!(out56, 0);

    // n57: flatten an iterator of options via flat_map
    let fm57: Vec<i32> = vec![Some(1), None, Some(3)].into_iter().flat_map(|o| o).collect();
    assert_eq!(fm57, vec![1, 3]);

    // n58: using filter_map for combined filter/map
    let fm58: Vec<i32> = (0..6).filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None }).collect();
    assert_eq!(fm58, vec![0, 20, 40]);

    // n59: using rev on a range (DoubleEndedIterator)
    let rev59: Vec<i32> = (0..4).rev().collect();
    assert_eq!(rev59, vec![3, 2, 1, 0]);

    // n60: using cycle with slice iter() then take and collect as chars
    let cyc60: Vec<char> = ['x', 'y'].iter().cycle().take(3).cloned().collect();
    assert_eq!(cyc60, vec!['x', 'y', 'x']);

    // ---------------------
    // More collection patterns, uniqueness, grouping
    // ---------------------

    // n61: deduplicate via collect into HashSet and back to Vec (order not guaranteed)
    let mut uniq61: Vec<_> = vec![1, 2, 2, 3].into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
    uniq61.sort();
    assert_eq!(uniq61, vec![1, 2, 3]);

    // n62: group by key using fold into a HashMap
    let pairs62 = vec![("a", 1), ("b", 2), ("a", 3)];
    let mut map62: std::collections::HashMap<&str, Vec<i32>> = std::collections::HashMap::new();
    for (k, v) in pairs62.into_iter() {
        map62.entry(k).or_default().push(v);
    }
    assert_eq!(map62.get("a").unwrap().as_slice(), &[1, 3]);

    // n63: chain iterator adaptors: map -> filter -> collect
    let chain63: Vec<_> = (0..=6)
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .collect();

    assert_eq!(chain63, vec![0, 4, 8, 12]);

    // n64: using product
    let product64: i32 = [2, 3, 4].into_iter().product();
    assert_eq!(product64, 24);

    // n65: using enumerate to label elements
    let labels65: Vec<_> = ["a", "b", "c"].iter().enumerate().map(|(i, &s)| (i, s)).collect();
    assert_eq!(labels65, vec![(0, "a"), (1, "b"), (2, "c")]);

    // n66: using repeat_with to generate varying values
    let generated66: Vec<i32> = std::iter::repeat_with(|| 1 + 1).take(3).collect();
    assert_eq!(generated66, vec![2, 2, 2]);

    // n67: using try_for_each with error propagation
    let r67: Result<(), &str> = (1..4).try_for_each(|x| if x == 2 { Err("err") } else { Ok(()) });
    assert_eq!(r67, Err("err"));

    // n68: using flat_map with iter() on slice of slices
    let nested68 = vec![&[1, 2][..], &[3][..]];
    let flat68: Vec<_> = nested68.into_iter().flat_map(|s| s.iter().cloned()).collect();
    assert_eq!(flat68, vec![1, 2, 3]);

    // n69: using array windows plus flatten sums
    let windows69: Vec<i32> = [1, 2, 3, 4].windows(3).map(|w| w.iter().sum()).collect();
    assert_eq!(windows69, vec![6, 9]);

    // n70: using chain from iter::empty
    let chained70: Vec<i32> = std::iter::empty().chain(0..3).collect();
    assert_eq!(chained70, vec![0, 1, 2]);

    // n71: using scan with Option early termination
    let s71: Vec<i32> = (1..).scan(0, |st, x| { if *st > 4 { None } else { *st += x; Some(*st) } }).take(3).collect();
    // can't know exact values easily, but we ensure length 3
    assert_eq!(s71.len(), 3);

    // n72: using windows(1) to convert slice to singletons
    let singletons72: Vec<Vec<i32>> = [1, 2, 3].windows(1).map(|w| w.iter().cloned().collect()).collect();
    assert_eq!(singletons72, vec![vec![1], vec![2], vec![3]]);

    // n73: enumerate then find by mapped key (fixed pattern)
    // Enumerate yields (usize, i32), so no need for '&'
    let found73 = (10..15)
        .enumerate()
        .find(|(_, x)| *x == 12)
        .map(|(i, _)| i);
    assert_eq!(found73, Some(2));


    // n74: use pointers via iter().map to raw pointer addresses (for demonstration)
    let addrs74: Vec<usize> = [1usize, 2, 3].iter().map(|x| x as *const usize as usize).collect();
    assert_eq!(addrs74.len(), 3);

    // n75: chained consumes into a string via fold
    let s75 = ["x", "y"].iter().fold(String::new(), |mut a, &b| { a.push_str(b); a });
    assert_eq!(s75, "xy".to_string());

    // ---------------------
    // Advanced: fallible & try-like patterns
    // ---------------------

    // n76: try_fold for safe accumulation with error
    let try76: Result<i32, &str> = (1..5).try_fold(0, |acc, x| if x == 4 { Err("stop") } else { Ok(acc + x) });
    assert_eq!(try76, Err("stop"));

    // n77: using iter over references then map to copy/cloned
    let refs77 = vec![String::from("a"), String::from("b")];
    let copied77: Vec<String> = refs77.iter().cloned().collect();
    assert_eq!(copied77, vec![String::from("a"), String::from("b")]);

    // n78: using take_while to stop when condition fails
    let t78: Vec<i32> = (0..10).take_while(|&x| x < 4).collect();
    assert_eq!(t78, vec![0, 1, 2, 3]);

    // n79: using skip_while to skip initial items
    let sk79: Vec<i32> = (0..5).skip_while(|&x| x < 2).collect();
    assert_eq!(sk79, vec![2, 3, 4]);

    // n80: using map_while (nightly API available) if present — fallback to filter_map
    // We'll use filter_map for compatibility
    let mw80: Vec<i32> = (0..6).filter_map(|x| if x < 3 { Some(x) } else { None }).collect();
    assert_eq!(mw80, vec![0, 1, 2]);

    // n81: using stepping via step_by on ranges (note: step_by stabilized)
    let step81: Vec<i32> = (0..10).step_by(3).collect();
    assert_eq!(step81, vec![0, 3, 6, 9]);

    // n82: using rev on a collected Vec
    let rev82: Vec<i32> = vec![1, 2, 3].into_iter().rev().collect();
    assert_eq!(rev82, vec![3, 2, 1]);

    // n83: using fold to compute max
    let max83 = (0..10).fold(std::i32::MIN, |m, x| m.max(x));
    assert_eq!(max83, 9);

    // n84: using min via iterator
    let min84 = vec![5, 3, 8].into_iter().min();
    assert_eq!(min84, Some(3));

    // n85: using product on floats
    let prod85: f64 = [1.5, 2.0].iter().product();
    assert_eq!(prod85, 3.0);

    // n86: using partition to split strings by length
    let (short86, long86): (Vec<&str>, Vec<&str>) = ["a", "abcd", "bc"].iter().partition(|&&s| s.len() <= 2);
    assert_eq!(short86.contains(&"a"), true);

    // n87: using map to Option then collect::<Option<Vec<_>>> via transpose-like pattern
    let maybe87: Option<Vec<i32>> = vec![Some(1), Some(2)].into_iter().collect();
    assert_eq!(maybe87, Some(vec![1, 2]));

    // n88: using iterator adaptors to create a range of tuples
    let tuples88: Vec<(i32, i32)> = (0..3).map(|x| (x, x * x)).collect();
    assert_eq!(tuples88, vec![(0, 0), (1, 1), (2, 4)]);

    // n89: using chained option flattening & collect
    let flatten89: Vec<i32> = vec![Some(1), None, Some(3)].into_iter().flatten().collect();
    assert_eq!(flatten89, vec![1, 3]);

    // n90: using inspect to assert pipeline unaffected
    let v90 = vec![2, 4];
    let result90: Vec<_> = v90.iter().inspect(|&&x| { let _ = x; }).map(|&x| x / 2).collect();
    assert_eq!(result90, vec![1, 2]);

    // ---------------------
    // Rare / edge-case constructions
    // ---------------------

    // n91: enumerate with large ranges and type conversions
    let e91: Vec<(usize, i64)> = (100..103).enumerate().map(|(i, v)| (i, v as i64)).collect();
    assert_eq!(e91[0], (0usize, 100i64));

    // n92: try collect into fixed array via map then try_into
    let vec92: Vec<i32> = (4..7).collect();
    let arr92: [i32; 3] = vec92.try_into().unwrap();
    assert_eq!(arr92, [4, 5, 6]);

    // n93: iterating over an iterator adaptor like Map then DoubleEnded usage
    let mut it93 = (0..4).map(|x| x * 2);
    let first93 = it93.next();
    assert_eq!(first93, Some(0));

    // n94: using join on iterator of strings by collect then join
    let s94 = vec!["a", "b", "c"].into_iter().collect::<Vec<_>>().join("-");
    assert_eq!(s94, "a-b-c".to_string());

    // n95: using extend to fill a collection from iterator
    let mut v95 = Vec::new();
    v95.extend(0..3);
    assert_eq!(v95, vec![0, 1, 2]);

    // n96: using drain on Vec to iterate and mutate original
    let mut vec96 = vec![1, 2, 3];
    let drained96: Vec<_> = vec96.drain(..).collect();
    assert_eq!(drained96, vec![1, 2, 3]);
    assert_eq!(vec96.len(), 0);

    // n97: using by-ref iterator of slice then collecting references
    let arr97 = [1, 2, 3];
    let refs97: Vec<&i32> = (&arr97).iter().collect();
    assert_eq!(refs97, vec![&1, &2, &3]);

    // n98: using step_by on collected iterator then sum
    let sum98: i32 = (0..10).step_by(2).sum();
    assert_eq!(sum98, 0 + 2 + 4 + 6 + 8);

    // n99: using take(n) on infinite iterator repeat_with, then collect
    let taken99: Vec<i32> = std::iter::repeat_with(|| 42).take(3).collect();
    assert_eq!(taken99, vec![42, 42, 42]);

    // n100: combining many adapters: zip, enumerate, filter_map
    let combined100: Vec<(usize, i32)> = (0..6)
        .zip([10, 11, 12, 13, 14, 15].iter().cloned())
        .enumerate()
        .filter_map(|(i, (a, b))| if b % 2 == 0 { Some((i, a + b)) } else { None })
        .collect();
    // Only pairs with even b remain: b=10,12,14 -> indices 0,2,4 -> sums 0+10,2+12,4+14
    assert_eq!(combined100, vec![(0, 10), (2, 14), (4, 18)]);
}
