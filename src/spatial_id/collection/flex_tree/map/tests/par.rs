#[cfg(all(test, feature = "rayon"))]
mod par_build {
    use crate::{FlexId, SingleId, SpatialIdMap};
    use alloc::vec::Vec;
    use rayon::prelude::*;

    /// 互いに素なセルに相異なる値を割り当てた入力（値の衝突なし）。
    fn items() -> Vec<(FlexId, u32)> {
        (0..800u32)
            .flat_map(|i| {
                SingleId::new(20, 0, i * 2, i % 11)
                    .unwrap()
                    .into_iter()
                    .map(move |f| (f, i))
            })
            .collect()
    }

    fn sorted(map: &SpatialIdMap<u32>) -> Vec<(FlexId, u32)> {
        let mut v: Vec<_> = map.iter().map(|(f, val)| (f, *val)).collect();
        v.sort();
        v
    }

    #[test]
    fn par_matches_sequential() {
        let its = items();
        let seq: SpatialIdMap<u32> = its.iter().cloned().collect();
        let par: SpatialIdMap<u32> = its.par_iter().cloned().collect();

        assert_eq!(seq.count(), par.count());
        assert_eq!(sorted(&seq), sorted(&par));
    }

    #[test]
    fn par_extend_matches_sequential_extend() {
        let its = items();
        let mut seq: SpatialIdMap<u32> = SpatialIdMap::new();
        seq.extend(its.iter().cloned());

        let mut par: SpatialIdMap<u32> = SpatialIdMap::new();
        par.par_extend(its.par_iter().cloned());

        assert_eq!(sorted(&seq), sorted(&par));
    }
}
