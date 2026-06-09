use std::{
    fs::{File, read_to_string},
    io::{BufWriter, Write},
    str::FromStr,
};

use kasane_logic::{IntoFlexIds, LevelOps, RangeId, SingleId, SpatialIdTable};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binding = read_to_string("sample/tran1.txt")?;

    let mut table: SpatialIdTable<bool> = SpatialIdTable::new();

    // ファイルは `z/f/x/y,` の繰り返しで、改行(CRLF)を含む。`,` で分割し、
    // 各トークンの前後空白(改行含む)を除去し、空トークンは読み飛ばす。
    for id_str in binding.split(',') {
        let id_str = id_str.trim();
        if id_str.is_empty() {
            continue;
        }
        let single_id = SingleId::from_str(id_str)?;
        table.insert(single_id, true);
    }

    let a = table.level_f(23, -10, 10).unwrap();

    // 入力と同じ `id,` 形式でファイルへ書き出す。
    let output_path = "sample/tran1_shifted.txt";
    let mut writer = BufWriter::new(File::create(output_path)?);
    for ele in a.into_flex_ids() {
        let range_id = RangeId::from(ele);
        write!(writer, "{range_id},")?;
    }
    writer.flush()?;

    println!("wrote {output_path}");

    Ok(())
}
