use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::hint::black_box;

const N: usize = 10_000;

fn bench_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("push");

    group.bench_function(BenchmarkId::new("Vec", "push"), |b| {
        b.iter_batched(
            || Vec::new(),
            |mut v| {
                for i in 0..N {
                    v.push(black_box(i));
                }
                v
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("VecDeque", "push"), |b| {
        b.iter_batched(
            || VecDeque::new(),
            |mut v| {
                for i in 0..N {
                    v.push_back(black_box(i));
                }
                v
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("LinkedList", "push"), |b| {
        b.iter_batched(
            || LinkedList::new(),
            |mut v| {
                for i in 0..N {
                    v.push_back(black_box(i));
                }
                v
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}


fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    group.bench_function(BenchmarkId::new("Vec", "get"), |b| {
        b.iter_batched(
            || {
                let mut v = Vec::with_capacity(N);
                for i in 0..N {
                    v.push(black_box(i));
                }
                v
            },
            | v| {
                let mut sum: usize = 0;
                for i in 0..N {
                    let c = black_box(v.get(i)).unwrap();
                    sum += c;
                }
                sum
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("VecDeque", "get"), |b| {
        b.iter_batched(
            || {
                let mut v = VecDeque::with_capacity(N);
                for i in 0..N {
                    v.push_back(black_box(i));
                }
                v
            },
            | v| {
                let mut sum: usize = 0;
                for i in 0..N {
                    let c = black_box(v.get(i)).unwrap();
                    sum += c;
                }
                sum
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("LinkedList", "get"), |b| {
        b.iter_batched(
            || {
                let mut l = LinkedList::new();
                for i in 0..N {
                    l.push_back(black_box(i));
                }
                l            },
            |mut l| {
                let mut sum = 0;
                for el in l.iter() {
                    sum += black_box(el);
                };
                sum

            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}



fn bench_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("pop");

    group.bench_function(BenchmarkId::new("Vec", "pop"), |b| {
        b.iter_batched(
            || {
                let mut v = Vec::with_capacity(N);
                for i in 0..N {
                    v.push(black_box(i));
                }
                v
            },
            |mut v| {
                for _ in 0..N {
                    black_box(v.pop());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("VecDeque", "pop"), |b| {
        b.iter_batched(
            || {
                let mut v = VecDeque::with_capacity(N);
                for i in 0..N {
                    v.push_back(black_box(i));
                }
                v
            },
            |mut v| {
                for _ in 0..N {
                    black_box(v.pop_back());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("LinkedList", "pop"), |b| {
        b.iter_batched(
            || {
                let mut v = LinkedList::new();
                for i in 0..N {
                    v.push_back(black_box(i));
                }
                v
            },
            |mut v| {
                for _ in 0..N {
                    black_box(v.pop_back());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    group.bench_function(BenchmarkId::new("HashMap", "insert"), |b| {
        b.iter_batched(
            || HashMap::new(),
            |mut map| {
                for i in 0..N {
                    map.insert(black_box(i), black_box(i + 1));
                }
                map
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("BTreeMap", "insert"), |b| {
        b.iter_batched(
            || BTreeMap::new(),
            |mut map| {
                for i in 0..N {
                    map.insert(black_box(i), black_box(i + 1));
                }
                map
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");

    group.bench_function(BenchmarkId::new("HashMap", "remove"), |b| {
        b.iter_batched(
            || {
                let mut map = HashMap::new();
                for i in 0..N {
                    map.insert(black_box(i), black_box(i + 1));
                }
                map
            },
            |mut map| {
                for i in 0..N {
                    black_box(map.remove(&black_box(i)));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("BTreeMap", "remove"), |b| {
        b.iter_batched(
            || {
                let mut map = BTreeMap::new();
                for i in 0..N {
                    map.insert(black_box(i), black_box(i + 1));
                }
                map
            },
            |mut map| {
                for i in 0..N {
                    black_box(map.remove(&black_box(i)));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}


// === Lookup (Map) Benchmarks ===
fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");

    group.bench_function(BenchmarkId::new("HashMap", "lookup"), |b| {
        b.iter_batched(
            || {
                let mut map = HashMap::new();
                for i in 0..N {
                    map.insert(i, i + 1);
                }
                map
            },
            |map| {
                for i in 0..N {
                    black_box(map.get(&black_box(i)));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("BTreeMap", "lookup"), |b| {
        b.iter_batched(
            || {
                let mut map = BTreeMap::new();
                for i in 0..N {
                    map.insert(i, i + 1);
                }
                map
            },
            |map| {
                for i in 0..N {
                    black_box(map.get(&black_box(i)));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}


criterion_group!(benches, bench_push, bench_get, bench_pop, bench_insert, bench_remove, bench_lookup);
criterion_main!(benches);
