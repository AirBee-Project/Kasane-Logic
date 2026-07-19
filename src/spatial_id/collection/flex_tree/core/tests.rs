#[cfg(test)]
mod merge_with_tests {
    use crate::FlexTreeCore;
    use crate::{RangeId, SingleId};
    use alloc::collections::BTreeMap;

    /// 木を最大ズームの [`SingleId`] 群へ平坦化し、セル→値の対応で取り出す。
    fn cells(core: &FlexTreeCore<u32>) -> BTreeMap<SingleId, u32> {
        core.flat_single_ids().collect()
    }

    /// 重なるセルでは resolve(a, b) で合成し、片側だけのセルはそのまま残る。
    #[test]
    fn merge_with_sum_resolves_overlap() {
        let mut a = FlexTreeCore::new();
        a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 3u32);
        a.insert(SingleId::new(20, 0, 1, 0).unwrap(), 5u32);

        let mut b = FlexTreeCore::new();
        b.insert(SingleId::new(20, 0, 1, 0).unwrap(), 7u32);
        b.insert(SingleId::new(20, 0, 2, 0).unwrap(), 9u32);

        let merged = a.merge_with(&b, |x, y| x + y);
        merged.assert_canonical();

        let c = cells(&merged);
        assert_eq!(c.get(&SingleId::new(20, 0, 0, 0).unwrap()), Some(&3)); // a のみ
        assert_eq!(c.get(&SingleId::new(20, 0, 1, 0).unwrap()), Some(&12)); // 5+7
        assert_eq!(c.get(&SingleId::new(20, 0, 2, 0).unwrap()), Some(&9)); // b のみ
        assert_eq!(c.len(), 3);
    }

    /// 粗いセル（a）と、その内部の細かいセル（b）を重ねると、重なった部分セルだけが
    /// resolve され、残りの部分セルは a の値を保つ（降下してセル単位で解決される）。
    #[test]
    fn merge_with_resolves_coarse_against_fine() {
        // z=18 の 1 セルは z=20 では F/X/Y 各 4 分割 = 4x4x4 = 64 の部分セルに展開される。
        let mut a = FlexTreeCore::new();
        a.insert(RangeId::new(18, [0, 0], [0, 0], [0, 0]).unwrap(), 1u32);

        let mut b = FlexTreeCore::new();
        b.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10u32);

        let merged = a.merge_with(&b, |x, y| x + y);
        merged.assert_canonical();

        let c = cells(&merged);
        assert_eq!(c.len(), 64);
        assert_eq!(c.get(&SingleId::new(20, 0, 0, 0).unwrap()), Some(&11)); // 1+10
        // 残りの 63 部分セルは a の値のまま。
        assert_eq!(c.values().filter(|&&v| v == 1).count(), 63);
    }

    /// resolve(v, v) != v になるポリシー（加算）でも、構造共有の同一部分木を
    /// 素通りさせず、重なりを正しく二重計上する（ptr_eq ショートカット無効の確認）。
    #[test]
    fn merge_with_same_tree_doubles() {
        let mut a = FlexTreeCore::new();
        a.insert(SingleId::new(20, 0, 5, 5).unwrap(), 4u32);
        a.insert(SingleId::new(20, 0, 6, 6).unwrap(), 8u32);

        let merged = a.merge_with(&a, |x, y| x + y);
        merged.assert_canonical();

        let c = cells(&merged);
        assert_eq!(c.get(&SingleId::new(20, 0, 5, 5).unwrap()), Some(&8));
        assert_eq!(c.get(&SingleId::new(20, 0, 6, 6).unwrap()), Some(&16));
    }

    /// 空木との merge_with は相手をそのまま返す（両方向）。
    #[test]
    fn merge_with_empty_is_identity() {
        let mut a = FlexTreeCore::new();
        a.insert(SingleId::new(20, 0, 1, 2).unwrap(), 42u32);
        let empty = FlexTreeCore::<u32>::new();

        let left = empty.merge_with(&a, |x, y| x + y);
        let right = a.merge_with(&empty, |x, y| x + y);
        left.assert_canonical();
        right.assert_canonical();

        assert_eq!(cells(&left), cells(&a));
        assert_eq!(cells(&right), cells(&a));
    }
}
