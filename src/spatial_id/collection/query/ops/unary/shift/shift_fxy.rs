use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    Error, FlexTreeCore, ZoomLevel, spatial_id::collection::flex_tree::core::SafeValue,
    spatial_id::collection::query::traits::UnaryOperator,
};

/// コレクション全体をF, X, Yの各方向へ同時に平行移動する単項演算。
/// オプティマイザが複数の `ShiftF`、`ShiftX`、`ShiftY` を1つにまとめる（Fusion）際のターゲットとして使用されます。
pub struct ShiftFXY {
    f: (ZoomLevel, i32),
    x: (ZoomLevel, i32),
    y: (ZoomLevel, i32),
}

impl ShiftFXY {
    /// 3次元方向それぞれの (ZoomLevel, 移動量) を指定して移動演算子を作る。
    pub fn new<T: Into<u8>, U: Into<u8>, V: Into<u8>>(
        f: (T, i32),
        x: (U, i32),
        y: (V, i32),
    ) -> Result<Self, Error> {
        Ok(Self {
            f: (ZoomLevel::new(f.0.into())?, f.1),
            x: (ZoomLevel::new(x.0.into())?, x.1),
            y: (ZoomLevel::new(y.0.into())?, y.1),
        })
    }
}

impl<V: SafeValue> UnaryOperator<V> for ShiftFXY {
    fn run(
        &self,
        target: &mut FlexTreeCore<V>,
    ) -> Result<(), Box<dyn core::error::Error + 'static>> {
        if self.f.1 == 0 && self.x.1 == 0 && self.y.1 == 0 {
            return Ok(());
        }

        let shifted = target.map_rebuild(|id, value| {
            let mut ids = alloc::vec![id];

            if self.f.1 != 0 {
                let z = self.f.0.get();
                let mut next = Vec::with_capacity(ids.len());
                for i in ids {
                    next.extend(i.shift_f(z, self.f.1)?);
                }
                ids = next;
            }

            if self.x.1 != 0 {
                let z = self.x.0.get();
                let mut next = Vec::with_capacity(ids.len());
                for i in ids {
                    next.extend(i.shift_x(z, self.x.1)?);
                }
                ids = next;
            }

            if self.y.1 != 0 {
                let z = self.y.0.get();
                let mut next = Vec::with_capacity(ids.len());
                for i in ids {
                    next.extend(i.shift_y(z, self.y.1)?);
                }
                ids = next;
            }

            let value = value.clone();
            Ok(ids.into_iter().map(move |i| (i, value.clone())))
        })?;

        *target = shifted;
        Ok(())
    }
}
