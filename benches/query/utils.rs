use kasane_logic::SpatialIdTable;
use std::fs;
use std::sync::OnceLock;

static FULL_DATA: OnceLock<SpatialIdTable<u32>> = OnceLock::new();

/// ベンチマーク起動時に一度だけ JSON を読み込んでパースする
pub fn get_full_data() -> &'static SpatialIdTable<u32> {
    FULL_DATA.get_or_init(|| {
        let json_str = fs::read_to_string("sample/bldg_risk.json")
            .expect("Failed to read sample/bldg_risk.json. Make sure you run from workspace root.");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    })
}

/// ベンチマーク結果のテーブルをJSONとしてファイルに保存する
#[allow(dead_code)]
pub fn save_result_json(name: &str, table: &SpatialIdTable<u32>) {
    let dir = "bench_results";
    fs::create_dir_all(dir).expect("Failed to create bench_results directory");
    let path = format!("{}/{}.json", dir, name);
    let json_str = serde_json::to_string(table).expect("Failed to serialize table");
    fs::write(&path, json_str).expect("Failed to write result json");
}
