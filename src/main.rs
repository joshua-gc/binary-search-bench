use binary_search_bench::{BenchInput, Variant, run_variant};
use clap::Parser;
use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Compare binary-search implementations")]
struct Args {
    /// Smallest input size as a power of two.
    #[arg(long, default_value_t = 10)]
    min_exp: u32,

    /// Largest input size as a power of two.
    #[arg(long, default_value_t = 22)]
    max_exp: u32,

    /// Number of random queries to issue per size.
    #[arg(long, default_value_t = 200_000)]
    queries: usize,

    /// Seed used for reproducible data and query generation.
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Optional CSV path.
    #[arg(long)]
    csv: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut csv = match &args.csv {
        Some(path) => Some(BufWriter::new(File::create(path)?)),
        None => None,
    };

    if let Some(writer) = csv.as_mut() {
        writeln!(
            writer,
            "variant,size,queries,hit_rate,elapsed_ns,total_hits,ns_per_query"
        )?;
    }

    println!(
        "{:<30} {:>12} {:>12} {:>10} {:>16}",
        "variant", "size", "queries", "hit_rate", "ns/query"
    );
    println!("{}", "-".repeat(86));

    for exp in args.min_exp..=args.max_exp {
        let size = 1usize << exp;
        let input = BenchInput::new(size, args.queries, args.seed ^ size as u64);
        let hit_rate = input.hit_rate();

        for variant in Variant::ALL {
            let start = Instant::now();
            let total_hits = run_variant(&input, variant);
            let elapsed = start.elapsed();
            let elapsed_ns = elapsed.as_nanos();
            let ns_per_query = elapsed_ns as f64 / args.queries as f64;

            println!(
                "{:<30} {:>12} {:>12} {:>9.3}% {:>16.2}",
                variant.name(),
                size,
                args.queries,
                hit_rate * 100.0,
                ns_per_query
            );

            if let Some(writer) = csv.as_mut() {
                writeln!(
                    writer,
                    "{},{},{},{:.6},{},{},{}",
                    variant.name(),
                    size,
                    args.queries,
                    hit_rate,
                    elapsed_ns,
                    total_hits,
                    ns_per_query
                )?;
            }
        }

        println!();
    }

    if let Some(writer) = csv.as_mut() {
        writer.flush()?;
    }

    Ok(())
}
