extern crate alloc;

#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;
#[allow(unused_imports)]
use kasane_logic::{Interval, SingleId, TemporalId};

fn main() {
    let temporal_id = TemporalId::new(30_u64, 10).unwrap();
    let single_id = SingleId::new(10, 10, 10, 10)
        .unwrap()
        .with_temporal(temporal_id);

    println!("{}", single_id)
}
