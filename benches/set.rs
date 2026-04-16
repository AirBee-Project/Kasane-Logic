use kasane_logic::{FlexTreeCore, SingleId};
use memori::{Bench, Func, TrackingAllocator};
use std::fs;
use std::hint::black_box;
use std::path::Path;

#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

fn main() {
    let data: &'static [SingleId] = load(Path::new("benches/data/sample1.txt")).leak();

    let percentages: Vec<usize> = (0..=10).map(|i| i * 10).collect();

    let mut func = Func::new("VBitSet Insert")
        .add_bench(
            "新宿区の建物データ",
            "データ量を徐々に増やして増加を確認",
            Bench::Scaling(percentages),
        )
        .add_function("VBitSet Insert", |&pct| {
            let target_len = (data.len() * pct) / 100;
            let subset = &data[..target_len];

            let mut vbit_set = FlexTreeCore::new();

            for single_id in subset {
                vbit_set.insert(single_id.clone(), ());
            }

            black_box(vbit_set.iter().count())
        });

    func.run_and_save().unwrap();
}

pub fn load<P: AsRef<Path>>(path: P) -> Vec<SingleId> {
    let content = fs::read_to_string(path).unwrap();
    let mut ids = Vec::new();
    for token in content.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() != 4 {
            panic!("Parser Error");
        }
        let z = parts[0].parse::<u8>().unwrap();
        let f = parts[1].parse::<i32>().unwrap();
        let x = parts[2].parse::<u32>().unwrap();
        let y = parts[3].parse::<u32>().unwrap();
        let id = SingleId::new(z, f, x, y).unwrap();
        ids.push(id);
    }
    ids
}
