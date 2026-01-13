
---

# 🚀 Rust Iterators – Idiomatic Cheat Sheet

> ⚙️ Zero-cost abstractions over collections and sequences.

---

## 1. **Iterator Creation**

Methods to obtain or construct iterators.

- `iter(&x)` → Immutable reference iterator (`&T`)
- `iter_mut(&mut x)` → Mutable reference iterator (`&mut T`)
- `into_iter(x)` → Owning iterator (`T`)
- `std::iter::once(val)`
- `std::iter::repeat(val)`
- `std::iter::repeat_with(|| gen())`
- `0..10` / `Range<T>`
- `x.iter().enumerate()` → index + item
- `x.iter().rev()`
- `x.iter().cycle()`

---

## 2. **Iterator Adaptors**

These methods take an iterator and return another iterator. Most are **lazy** (must be consumed explicitly).

### 🔁 **Transformation Adaptors**

- `map(|x| -> y)` – apply a function to each item
- `filter(|x| bool)` – retain items matching predicate
- `filter_map(|x| Option<_>)` – filter + map
- `flat_map(|x| iter)` – map then flatten
- `chain(other_iter)`
- `cloned()` – copy `&T` to `T` (requires `Clone`)
- `copied()` – copy from `&T` to `T` (requires `Copy`)
- `zip(other_iter)`
- `enumerate()` – tuple of index and item
- `by_ref()` – borrow iterator for reuse
- `fused()` – once it returns `None`, always returns `None`
- `inspect(|x| dbg!)` – peek into the pipeline (debugging)
- `scan(initial, |state, x| Option<Y>)` – like fold but yields intermediate values
- `peekable()` / `peek()` – look ahead

### 🧩 **Structure Adaptors**

- `flatten()` – flatten nested iterators
- `chunks(n)` (via `itertools`) – fixed-size chunks
- `chunks_exact(n)`
- `step_by(n)` – yield every n-th item

---

## 3. **Consumption / Terminal Operations**

These methods **consume** the iterator and produce a final value or collection.

### 📦 **Collection + Conversion**

- `collect()` → collect into `Vec`, `HashMap`, etc.
- `collect::<Vec<_>>()`
- `collect_into(&mut target)`
- `.try_into()` → fallible conversion
- `into_boxed_slice()`
- `into_boxed_str()`
- `to_vec()`
- `to_string()` (if possible)

### 🧮 **Aggregation / Reduction**

- `fold(init, |acc, x| ...)`
- `reduce(|a, b| ...) -> Option<_>`
- `sum()`
- `product()`
- `count()`
- `max()`, `min()`
- `max_by(|a, b| ...)`, `min_by(...)`

### 🔍 **Searching / Query**

- `all(|x| ...) -> bool`
- `any(|x| ...) -> bool`
- `find(|x| ...) -> Option<T>`
- `position(|x| ...) -> Option<usize>`
- `nth(n) -> Option<T>` – jump to the nth item
- `last() -> Option<T>`
- `next() -> Option<T>`  – advances the iterator

---

## 4. **Misc. Iterator Tools**

### ➗ **Splitting / Branching**

- `partition(|x| ...)` → `(Vec<T>, Vec<T>)`
- `unzip()` → `(Vec<_>, Vec<_>)`
- `take(n)`
- `skip(n)`
- `take_while(...)`
- `skip_while(...)`

### 🧹 **Deduplication**

(available via **external crates** like `itertools` or `std` slice methods)

- `dedup()` – adjacent duplicates (slice method)
- `unique()` – for unique items (via **`itertools`** crate)

---

## 5. **Side Effects / Debugging**

- `for_each(|x| ...)` – runs a function for each item (consumes iterator)
- `inspect(|x| println!("{:?}", x))`

---

## Summary Table

| Category                    | Examples                                 |
|----------------------------|------------------------------------------|
| **Creation**               | `iter()`, `into_iter()`, `once()`       |
| **Lazy Adaptors**          | `map()`, `filter()`, `flat_map()`       |
| **Stateful Adaptors**      | `scan()`, `enumerate()`, `peekable()`   |
| **Duplication**            | `cloned()`, `copied()`                  |
| **Chaining Iterators**     | `chain()`, `zip()`, `rev()`, `cycle()`  |
| **Consumption**            | `collect()`, `sum()`, `count()`, `find()` |
| **Debug / Peek**           | `for_each()`, `inspect()`, `peek()`     |

---

## ⚠️ About `dedup()` and `unique()`

- `dedup()` is **not** an iterator method; it’s on slices (`Vec`): `vec.dedup()`
- `unique()` is from the `itertools` crate.

---

💡Protip: Use **Rust’s [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)** trait's official documentation or **`.methods()`** suggestion in IDEs like `rust-analyzer` to explore more iterator magic!


