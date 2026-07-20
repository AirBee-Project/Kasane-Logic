use super::{shift_f::ShiftF, shift_x::ShiftX, shift_y::ShiftY};
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{MergeAccumulator, UnaryOperator, WorkingTree},
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

impl<W: WorkingTree + 'static> UnaryOperator<W> for ShiftFXY {
    fn validate(&self) -> Result<(), Error> {
        self.f.0.check_f(self.f.1)?;
        self.x.0.check_x(self.x.1.unsigned_abs())?;
        self.y.0.check_y(self.y.1.unsigned_abs())?;
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut W) -> Result<(), Error> {
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

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::separable_injective()
    }

    /// `ShiftX`/`ShiftY`/`ShiftF`/`ShiftFXY`（自分自身を含む）は、軸ごとにオフセットを加算した
    /// 1つの `ShiftFXY` に統合できる（平行移動の合成は軸ごとの加算そのもの）。
    fn try_merge(&self, other: &dyn UnaryOperator<W>) -> Option<Box<dyn UnaryOperator<W>>> {
        crate::spatial_id::collection::query::traits::try_merge_via_accumulator::<W, ShiftFXY>(
            self, other,
        )
    }
}

/// `ShiftFXY` の1軸ぶんの状態 `(ZoomLevel, オフセット)` を合成する。
/// オフセット0は「未使用（恒等）」を表す。片方が未使用ならもう片方を採用し、両方使用中なら
/// ズームレベルが一致する場合のみオフセットを加算する（不一致は合成不能）。
fn merge_axis(cur: (ZoomLevel, i32), add: (ZoomLevel, i32)) -> Option<(ZoomLevel, i32)> {
    if add.1 == 0 {
        Some(cur)
    } else if cur.1 == 0 {
        Some(add)
    } else if cur.0 == add.0 {
        Some((cur.0, cur.1 + add.1))
    } else {
        None
    }
}

impl<W: WorkingTree + 'static> MergeAccumulator<W> for ShiftFXY {
    fn seed(op: &dyn UnaryOperator<W>) -> Option<Self> {
        let any = op.as_any();
        if let Some(o) = any.downcast_ref::<ShiftFXY>() {
            return Some(Self {
                f: o.f,
                x: o.x,
                y: o.y,
            });
        }
        if let Some(o) = any.downcast_ref::<ShiftX>() {
            let z = o.z();
            return Some(Self {
                f: (z, 0),
                x: (z, o.x()),
                y: (z, 0),
            });
        }
        if let Some(o) = any.downcast_ref::<ShiftY>() {
            let z = o.z();
            return Some(Self {
                f: (z, 0),
                x: (z, 0),
                y: (z, o.y()),
            });
        }
        if let Some(o) = any.downcast_ref::<ShiftF>() {
            let z = o.z();
            return Some(Self {
                f: (z, o.f()),
                x: (z, 0),
                y: (z, 0),
            });
        }
        None
    }

    fn absorb(&mut self, op: &dyn UnaryOperator<W>) -> bool {
        let Some(delta) = <Self as MergeAccumulator<W>>::seed(op) else {
            return false;
        };
        let (Some(f), Some(x), Some(y)) = (
            merge_axis(self.f, delta.f),
            merge_axis(self.x, delta.x),
            merge_axis(self.y, delta.y),
        ) else {
            return false;
        };
        self.f = f;
        self.x = x;
        self.y = y;
        true
    }
}
