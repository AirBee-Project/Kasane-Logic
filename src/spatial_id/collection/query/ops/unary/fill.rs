use crate::spatial_id::collection::query::Query;
use crate::{
    CellValue, ConflictPolicy, Error, IterFlexIds, RangeId, SpatialIdCollection, UnaryOperator,
};

/// 値を持つ領域を包む最小範囲の中で、まだ値が無いセルへ既定値を割り当てる演算。
pub struct FillDefault;

impl<A: CellValue> UnaryOperator<A> for FillDefault {
    /// 隙間へ割り当てる既定値。
    type CustomParameter = A;
    type ResultValue = A;

    fn execution<S, O>(a: S, default: A) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let bbox = RangeId::bounding_box_of(a.scan().map(|(flex_id, _)| flex_id));
        let defaults = bbox
            .as_ref()
            .into_iter()
            .flat_map(|b| b.iter_flex_ids())
            .map(move |flex_id| (flex_id, default.clone()));

        Ok(O::from_cells(
            defaults.chain(a.scan()),
            &ConflictPolicy::Overwrite,
        ))
    }

    fn is_identity(_custom_parameter: &Self::CustomParameter) -> bool {
        false
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn fill_default(self, default: C::Value) -> Self {
        Query::Unary(
            crate::spatial_id::collection::query::ops::unary::UnaryOp::Fill(default),
            alloc::boxed::Box::new(self),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::SpatialIdCollection;
    use crate::{FlexTree, SingleId, SpatialIdTable};

    fn id(z: u8, f: i32, x: u32, y: u32) -> SingleId {
        SingleId::new(z, f, x, y).unwrap()
    }

    fn value_at(t: &SpatialIdTable<i32>, z: u8, f: i32, x: u32, y: u32) -> Option<i32> {
        t.get(&id(z, f, x, y)).next().map(|(_, v)| *v)
    }

    #[test]
    fn fill_default_fills_gap_and_keeps_originals() {
        let mut a = SpatialIdTable::<i32>::new();
        a.insert(id(20, 0, 0, 0), 10);
        a.insert(id(20, 0, 2, 0), 30);

        let filled = a.clone().query().fill_default(7).run().unwrap();

        assert_eq!(value_at(&filled, 20, 0, 0, 0), Some(10)); // 元の値
        assert_eq!(value_at(&filled, 20, 0, 1, 0), Some(7)); // 隙間 → 既定値
        assert_eq!(value_at(&filled, 20, 0, 2, 0), Some(30)); // 元の値
    }

    #[test]
    fn fill_default_writes_nothing_outside_bbox() {
        let mut a = SpatialIdTable::<i32>::new();
        a.insert(id(20, 0, 1, 1), 10);
        a.insert(id(20, 0, 3, 1), 30);

        let filled = a.clone().query().fill_default(7).run().unwrap();

        // AABB は x[1,3], y[1,1], f[0,0]。その外側は空のまま。
        assert_eq!(value_at(&filled, 20, 0, 0, 1), None); // x が bbox 左外
        assert_eq!(value_at(&filled, 20, 0, 4, 1), None); // x が bbox 右外
        assert_eq!(value_at(&filled, 20, 0, 2, 0), None); // y が bbox 外
        assert_eq!(value_at(&filled, 20, 1, 2, 1), None); // f が bbox 外
        // 内側の隙間は埋まる。
        assert_eq!(value_at(&filled, 20, 0, 2, 1), Some(7));
    }

    #[test]
    fn fill_default_fills_2d_gap_and_preserves_corners() {
        // 対角の2セルで AABB は x[0,2] × y[0,2]。9セル中、両端2つは元の値、残り7つが既定値。
        let mut a = SpatialIdTable::<i32>::new();
        a.insert(id(20, 0, 0, 0), 10);
        a.insert(id(20, 0, 2, 2), 30);

        let filled = a.clone().query().fill_default(7).run().unwrap();

        // bbox 内の 3×3=9 セルすべてを検証。両端の角だけ元の値、残りは既定値。
        for x in 0..=2 {
            for y in 0..=2 {
                let expected = match (x, y) {
                    (0, 0) => Some(10),
                    (2, 2) => Some(30),
                    _ => Some(7),
                };
                assert_eq!(value_at(&filled, 20, 0, x, y), expected, "cell ({x},{y})");
            }
        }
    }

    #[test]
    fn fill_default_on_empty_returns_empty() {
        let a = SpatialIdTable::<i32>::new();
        let filled = a.clone().query().fill_default(7).run().unwrap();

        assert_eq!(filled.count(), 0);
    }

    #[test]
    fn fill_default_mixed_zoom_normalizes_bbox() {
        // 粗い z19 セルと細かい z20 セルが離れて存在。AABB は最大ズーム z20 で取られる。
        let mut a = SpatialIdTable::<i32>::new();
        a.insert(id(19, 0, 0, 0), 10); // z20 では x[0,1] を覆う
        a.insert(id(20, 0, 4, 0), 30);

        let filled = a.clone().query().fill_default(7).run().unwrap();

        // 元のセル群は保持。
        assert_eq!(value_at(&filled, 20, 0, 0, 0), Some(10)); // 粗いセル内（元の値）
        assert_eq!(value_at(&filled, 20, 0, 1, 0), Some(10)); // 粗いセル内（元の値）
        assert_eq!(value_at(&filled, 20, 0, 4, 0), Some(30)); // 元の値
        // 隙間（x2,x3）は既定値で埋まる。
        assert_eq!(value_at(&filled, 20, 0, 2, 0), Some(7));
        assert_eq!(value_at(&filled, 20, 0, 3, 0), Some(7));
    }

    #[test]
    fn bounding_box_returns_aabb_corners() {
        let mut core = FlexTree::<i32>::new();
        core.insert(id(20, 0, 0, 0), 1);
        core.insert(id(20, 0, 2, 3), 1);

        let bbox = core.bounding_box().unwrap();

        assert_eq!(bbox.z(), 20);
        assert_eq!(bbox.f(), [0, 0]);
        assert_eq!(bbox.x(), [0, 2]);
        assert_eq!(bbox.y(), [0, 3]);
    }

    #[test]
    fn bounding_box_of_empty_is_none() {
        let core = FlexTree::<i32>::new();
        assert!(core.bounding_box().is_none());
    }
}
