use kasane_logic::geometry::{
    coordinate::Coordinate,
    shapes::line::{line, line_dda},
};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("output.txt")?;
    let a = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let b = Coordinate::new(35.630152, 139.74044000000004, 100.0)?;
    let iter = line(22, a, b)?;
    for id in iter {
        // SingleIDの内容を一行ずつ書き込む
        writeln!(file, "{},", id)?;
    }

    Ok(println!("結果を output.txt に保存しました 。"))
}
