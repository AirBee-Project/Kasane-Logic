extern crate alloc;

#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;
use kasane_logic::{Interval, SingleId, TemporalId};

fn main() {
    #[cfg(feature = "temporal_id")]
    {
        let interval = Interval::Day;
        let temporal_id = TemporalId::new(interval, 10).unwrap();
        let _single_id = SingleId::new(3, 10, 10, 10)
            .unwrap()
            .with_temporal(temporal_id);
    }
}
