use std::collections::HashSet;

fn main() {
    let a: HashSet<_> = (0..1_000_000).collect();
    let b: HashSet<_> = (500_000..1_500_000).collect();

    let start = std::time::Instant::now();
    let inter: HashSet<_> = a.intersection(&b).copied().collect();
    let dur = start.elapsed();

    println!("intersection size: {}", inter.len());
    println!("elapsed: {:?}", dur);
}
