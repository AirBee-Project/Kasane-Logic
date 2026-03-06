use kasane_logic::{RangeId, SpatialId};

fn main() {
    let id1 = RangeId::random_at(10);
    println!("{}", id1);
    let all = id1.single_ids().into_iter().count();
    let optimze = id1.optimize_single_ids().into_iter().count();

    println!("{}>{}", all, optimze)
}
