use crate::{FlexId, SpatialIdTable};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// 再現可能な疑似乱数（xorshift64）。
fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// ズームの浅い FlexId を作る。浅いので互いに重なりやすく、木の中で割られる。
fn random_id(state: &mut u64) -> FlexId {
    let mut zoom = || (next(state) % 5) as u8;
    let (fz, xz, yz) = (zoom(), zoom(), zoom());
    let f = (next(state) % (2 << fz)) as i32 - (1 << fz);
    let x = (next(state) % (1 << xz)) as u32;
    let y = (next(state) % (1 << yz)) as u32;
    let id = FlexId::new(fz, f, xz, x, yz, y).unwrap();
    if cfg!(feature = "temporal_id") && next(state).is_multiple_of(2) {
        let tz = (next(state) % 4) as u8;
        id.with_time(tz, next(state) % (1 << tz)).unwrap()
    } else {
        id
    }
}

#[test]
fn table_iterates_in_canonical_order() {
    let mut state = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..200 {
        let table: SpatialIdTable<u64> = (0..40)
            .map(|_| (random_id(&mut state), next(&mut state) % 3))
            .collect();
        let ids: Vec<FlexId> = table.iter().map(|(id, _)| id).collect();
        for pair in ids.windows(2) {
            assert_eq!(pair[0].canonical_cmp(&pair[1]), Some(Ordering::Less));
            assert!(pair[0].canonical_key() < pair[1].canonical_key());
        }
    }
}

#[test]
fn coarser_element_across_finer_split_is_unordered() {
    // a は F を割っていないが、b が F を割るので、木では a が F で 2 つに割られて b を挟む
    let a = FlexId::new(0, 0, 1, 0, 0, 0).unwrap();
    let b = FlexId::new(2, 1, 1, 1, 0, 0).unwrap();
    assert_eq!(a.canonical_cmp(&b), None);
    assert_eq!(b.canonical_cmp(&a), None);

    let a_lower = FlexId::new(1, 0, 1, 0, 0, 0).unwrap();
    let a_upper = FlexId::new(1, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(a_lower.canonical_cmp(&b), Some(Ordering::Less));
    assert_eq!(b.canonical_cmp(&a_upper), Some(Ordering::Less));
}

#[test]
fn overlapping_ids_are_unordered() {
    let parent = FlexId::new(1, 0, 1, 0, 1, 0).unwrap();
    let child = FlexId::new(2, 1, 2, 0, 2, 1).unwrap();
    assert_eq!(parent.canonical_cmp(&child), None);
    assert_eq!(parent.canonical_cmp(&parent), None);
}

#[test]
fn upper_root_comes_first() {
    let [upper, lower] = FlexId::ROOTS;
    assert_eq!(upper.canonical_cmp(&lower), Some(Ordering::Less));
    assert!(upper.canonical_key() < lower.canonical_key());
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "QueryIter の制約違反")]
fn from_ordered_panics_on_unordered_input() {
    let [upper, lower] = FlexId::ROOTS;
    crate::spatial_id::collection::query::traits::QueryIter::from_ordered(
        [Ok((lower, ())), Ok((upper, ()))].into_iter(),
    )
    .for_each(drop);
}
