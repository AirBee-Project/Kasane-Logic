use kasane_logic::Coordinate;
use kasane_logic::CoverSingleIds;
// use kasane_logic::Cylinder;
// use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    // let tokyo_dash = Coordinate::new(35.6813, 139.764, 500.0)?;
    // let c = Cylinder::new(tokyo, tokyo_dash, 6.0)?;
    // let s = c.rough_solid();
    // let ids = s.cover_single_ids(25).unwrap();

    // let mut file = std::fs::File::create("output1.txt")?;
    // for id in ids {
    //     writeln!(file, "{},", id)?;
    // }
    let shinagawa = Coordinate::new(35.630152, 139.74044000000004, 50.0)?;
    println!(
        "{},{}",
        tokyo.cover_single_ids(23)?.next().unwrap(),
        shinagawa.cover_single_ids(23)?.next().unwrap()
    );

    Ok(())
}
