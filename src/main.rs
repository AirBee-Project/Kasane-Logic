use kasane_logic::Coordinate;
use kasane_logic::CoverSingleIds;
use kasane_logic::Cylinder;
use kasane_logic::IterSolids;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokyo = Coordinate::new(35.681382, 139.76608399999998, 0.0)?;
    let tokyo_dash = Coordinate::new(35.6813, 139.764, 500.0)?;
    let c = Cylinder::new(tokyo, tokyo_dash, 6.0)?;
    let s = c.iter_solids().next().unwrap();
    let ids = s.cover_single_ids(25).unwrap();

    let mut file = std::fs::File::create("output.txt")?;
    for id in ids {
        writeln!(file, "{},", id)?;
    }

    Ok(())
}
