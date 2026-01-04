use kasane_logic::geometry::{
    coordinate::Coordinate,
    shapes::line::{line, line_dda},
};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("output.txt")?;
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let nagoya = Coordinate::new(35.1706431, 136.8816945, 100.0)?;
    let yokohama = Coordinate::new(35.4660694, 139.6226196, 100.0)?;
    let (z, start, goal) = (23, nagoya, tokyo);
    let iter = line_dda(z, start, goal)?;
    for id in iter {
        // SingleIDの内容を一行ずつ書き込む
        writeln!(file, "{},", id)?;
    }
    println!("{},{}", start.to_id(z), goal.to_id(z));
    Ok(println!("結果を output.txt に保存しました 。"))
}
