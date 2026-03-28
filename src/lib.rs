use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::hint::black_box;

#[cfg(target_arch = "x86")]
use core::arch::x86::{_mm_prefetch, _MM_HINT_T0};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};

#[derive(Clone, Debug)]
pub struct Eytzinger {
    data: Vec<u32>,
}

impl Eytzinger {
    pub fn from_sorted(input: &[u32]) -> Self {
        fn fill(sorted: &[u32], out: &mut [u32], next: &mut usize, idx: usize) {
            if idx >= out.len() {
                return;
            }
            fill(sorted, out, next, idx * 2);
            out[idx] = sorted[*next];
            *next += 1;
            fill(sorted, out, next, idx * 2 + 1);
        }

        let mut data = vec![0; input.len() + 1];
        let mut next = 0;
        fill(input, &mut data, &mut next, 1);
        Self { data }
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.data
    }

    pub fn contains_branchy(&self, target: u32) -> bool {
        if self.data.len() <= 1 {
            return false;
        }

        let mut idx = 1usize;
        while idx < self.data.len() {
            let el = self.data[idx];
            if el == target {
                return true;
            }
            idx = idx * 2 + usize::from(el < target);
        }
        false
    }

    pub fn contains_branchless(&self, target: u32) -> bool {
        if self.data.len() <= 1 {
            return false;
        }

        let mut idx = 1usize;
        while idx < self.data.len() {
            let el = self.data[idx];
            idx = idx * 2 + usize::from(el < target);
        }

        idx >>= idx.trailing_ones() as usize + 1;
        idx != 0 && idx < self.data.len() && self.data[idx] == target
    }

    pub fn contains_branchless_prefetch(&self, target: u32) -> bool {
        if self.data.len() <= 1 {
            return false;
        }

        let mut idx = 1usize;
        while idx < self.data.len() {
            prefetch_index(self.data.as_slice(), idx * 2);
            let el = self.data[idx];
            idx = idx * 2 + usize::from(el < target);
        }

        idx >>= idx.trailing_ones() as usize + 1;
        idx != 0 && idx < self.data.len() && self.data[idx] == target
    }
}

#[inline]
pub fn std_binary_search_contains(data: &[u32], target: u32) -> bool {
    data.binary_search(&target).is_ok()
}

#[inline]
pub fn partition_point_contains(data: &[u32], target: u32) -> bool {
    let idx = data.partition_point(|&x| x < target);
    idx < data.len() && data[idx] == target
}

#[inline]
pub fn manual_binary_search_contains(data: &[u32], target: u32) -> bool {
    let mut lo = 0usize;
    let mut hi = data.len();

    while lo < hi {
        let mid = lo + ((hi - lo) >> 1);
        let value = data[mid];
        if value < target {
            lo = mid + 1;
        } else if value > target {
            hi = mid;
        } else {
            return true;
        }
    }

    false
}

#[inline]
fn prefetch_index(data: &[u32], idx: usize) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        let ptr = data.as_ptr().wrapping_add(idx) as *const i8;
        _mm_prefetch::<_MM_HINT_T0>(ptr);
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = (data, idx);
    }
}

pub fn generate_gapped_sorted(size: usize, seed: u64) -> Vec<u32> {
    if size == 0 {
        return Vec::new();
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    let mut result = Vec::with_capacity(size);
    let mut current = 1u32;
    result.push(current);

    while result.len() < size {
        current = current.saturating_add(1 + 2 * u32::from(rng.gen::<bool>()));
        result.push(current);
    }

    result
}

pub fn generate_queries(max_value: u32, count: usize, seed: u64) -> Vec<u32> {
    let mut rng = SmallRng::seed_from_u64(seed);
    (0..count)
        .map(|_| rng.gen_range(1..=max_value))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Variant {
    StdBinarySearch,
    ManualBinarySearch,
    PartitionPoint,
    EytzingerBranchy,
    EytzingerBranchless,
    EytzingerBranchlessPrefetch,
}

impl Variant {
    pub const ALL: [Variant; 6] = [
        Variant::StdBinarySearch,
        Variant::ManualBinarySearch,
        Variant::PartitionPoint,
        Variant::EytzingerBranchy,
        Variant::EytzingerBranchless,
        Variant::EytzingerBranchlessPrefetch,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Variant::StdBinarySearch => "std::binary_search",
            Variant::ManualBinarySearch => "manual_branchy",
            Variant::PartitionPoint => "slice::partition_point",
            Variant::EytzingerBranchy => "eytzinger_branchy",
            Variant::EytzingerBranchless => "eytzinger_branchless",
            Variant::EytzingerBranchlessPrefetch => "eytzinger_branchless_prefetch",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BenchInput {
    pub sorted: Vec<u32>,
    pub eytzinger: Eytzinger,
    pub queries: Vec<u32>,
}

impl BenchInput {
    pub fn new(size: usize, query_count: usize, seed: u64) -> Self {
        let sorted = generate_gapped_sorted(size, seed);
        let max_value = *sorted.last().unwrap_or(&1);
        let eytzinger = Eytzinger::from_sorted(&sorted);
        let queries = generate_queries(max_value, query_count, seed ^ 0x9E37_79B9_7F4A_7C15);

        Self {
            sorted,
            eytzinger,
            queries,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self
            .queries
            .iter()
            .filter(|&&q| self.sorted.binary_search(&q).is_ok())
            .count();
        hits as f64 / self.queries.len() as f64
    }
}

pub fn run_variant(input: &BenchInput, variant: Variant) -> usize {
    match variant {
        Variant::StdBinarySearch => input
            .queries
            .iter()
            .copied()
            .map(|q| std_binary_search_contains(black_box(&input.sorted), black_box(q)) as usize)
            .sum(),
        Variant::ManualBinarySearch => input
            .queries
            .iter()
            .copied()
            .map(|q| manual_binary_search_contains(black_box(&input.sorted), black_box(q)) as usize)
            .sum(),
        Variant::PartitionPoint => input
            .queries
            .iter()
            .copied()
            .map(|q| partition_point_contains(black_box(&input.sorted), black_box(q)) as usize)
            .sum(),
        Variant::EytzingerBranchy => input
            .queries
            .iter()
            .copied()
            .map(|q| input.eytzinger.contains_branchy(black_box(q)) as usize)
            .sum(),
        Variant::EytzingerBranchless => input
            .queries
            .iter()
            .copied()
            .map(|q| input.eytzinger.contains_branchless(black_box(q)) as usize)
            .sum(),
        Variant::EytzingerBranchlessPrefetch => input
            .queries
            .iter()
            .copied()
            .map(|q| input.eytzinger.contains_branchless_prefetch(black_box(q)) as usize)
            .sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eytzinger_matches_sorted_membership() {
        for size in [1usize, 2, 3, 7, 8, 31, 32, 100, 1000] {
            let sorted = generate_gapped_sorted(size, 1234 + size as u64);
            let e = Eytzinger::from_sorted(&sorted);
            let max = *sorted.last().unwrap();

            for q in 1..=max + 2 {
                let expected = sorted.binary_search(&q).is_ok();
                assert_eq!(e.contains_branchy(q), expected, "branchy failed for size={size}, q={q}");
                assert_eq!(e.contains_branchless(q), expected, "branchless failed for size={size}, q={q}");
                assert_eq!(
                    e.contains_branchless_prefetch(q),
                    expected,
                    "prefetch failed for size={size}, q={q}"
                );
            }
        }
    }

    #[test]
    fn standard_variants_agree() {
        let sorted = generate_gapped_sorted(10_000, 99);
        for q in 1..=*sorted.last().unwrap() + 3 {
            let expected = sorted.binary_search(&q).is_ok();
            assert_eq!(manual_binary_search_contains(&sorted, q), expected);
            assert_eq!(partition_point_contains(&sorted, q), expected);
        }
    }

    #[test]
    fn hit_rate_is_reasonable() {
        let input = BenchInput::new(100_000, 20_000, 7);
        let rate = input.hit_rate();
        assert!(rate > 0.40 && rate < 0.60, "unexpected hit rate: {rate}");
    }
}
