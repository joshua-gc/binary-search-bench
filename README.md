# binary-search-bench

A small Rust project for comparing multiple ways to search sorted data, inspired by Denis Bazhenov's article **"Fast(er) binary search in Rust"**. The article benchmarks standard slice binary search against Eytzinger-layout variants, including branchless and software-prefetch versions.

This project includes:

- `slice::binary_search()`
- a manual branchy binary search
- `slice::partition_point()` as another standard-library baseline
- Eytzinger layout + branchy search
- Eytzinger layout + branchless search
- Eytzinger layout + branchless search + prefetch on x86/x86_64

## Why Eytzinger?

Bazhenov's post explains that normal binary search becomes increasingly memory-latency bound as the array grows beyond cache, while the Eytzinger layout makes access patterns more predictable. The article then improves that baseline by removing a branch and adding software prefetching.

## Project layout

- `src/lib.rs` - search implementations and helpers
- `src/main.rs` - quick CLI runner that prints a table or CSV
- `benches/compare.rs` - Criterion benchmark suite

## Requirements

- Rust stable (recent toolchain)
- For the prefetch variant: `x86` or `x86_64` target to use the intrinsic; other targets fall back to the branchless Eytzinger version.

## Run the quick CLI comparison

```bash
cargo run --release -- --queries 200000 --min-exp 10 --max-exp 22
```

That benchmarks arrays of sizes `2^10` through `2^22` and prints average nanoseconds per query.

To write CSV:

```bash
cargo run --release -- --queries 200000 --min-exp 10 --max-exp 22 --csv results.csv
```

## Run Criterion

```bash
cargo bench
```

Open the Criterion HTML report under `target/criterion/report/index.html`.

## Notes on the data set

The article's sample code generates a sorted array with gaps so random lookups are present about half the time. This project uses the same idea for fairer benchmarking of hits and misses together. The article's benchmark repo uses this style of generator as well.

## A small implementation detail

For the prefetch variant, this project prefetches the target data address directly. The article motivates unconditional prefetching via `wrapping_offset()` and notes that prefetch is just a hint, so going past the logical end is acceptable as long as the pointer is not dereferenced.
