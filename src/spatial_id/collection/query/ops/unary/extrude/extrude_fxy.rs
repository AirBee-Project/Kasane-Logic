use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, FlexId,
    spatial_id::{
        collection::query::{
            merge_policy::MergePolicy,
            traits::{UnaryOperator, WorkingTree},
        },
        zoom_level::ZoomLevel,
    },
};
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// 任意のボクセルの現在のF, X, Y座標を無視し、指定範囲に引き延ばす演算子。
pub struct ExtrudeFXY<P> {
    pub target_z: ZoomLevel,
    pub f_range: Option<(i32, i32)>,
    pub x_range: Option<(u32, u32)>,
    pub y_range: Option<(u32, u32)>,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<P> ExtrudeFXY<P> {
    pub fn new(
        target_z: ZoomLevel,
        f_range: Option<(i32, i32)>,
        x_range: Option<(u32, u32)>,
        y_range: Option<(u32, u32)>,
    ) -> Self {
        Self {
            target_z,
            f_range,
            x_range,
            y_range,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<W, P> UnaryOperator<W> for ExtrudeFXY<P>
where
    W: WorkingTree + 'static,
    P: MergePolicy<W::Value>,
{
    fn validate(&self) -> Result<(), Error> {
        let z = self.target_z.get();
        let zl = ZoomLevel::new(z)?;
        if let Some((start_f, end_f)) = self.f_range {
            zl.check_f(start_f)?;
            zl.check_f(end_f)?;
        }
        if let Some((start_x, end_x)) = self.x_range {
            zl.check_x(start_x)?;
            zl.check_x(end_x)?;
        }
        if let Some((start_y, end_y)) = self.y_range {
            zl.check_y(start_y)?;
            zl.check_y(end_y)?;
        }
        Ok(())
    }

    fn run(&self, core: &mut W) -> Result<(), Error> {
        let mut extruded: Vec<(FlexId, W::Value)> = Vec::with_capacity(core.count());

        // 元のツリーから全セルを取り出し、それぞれを引き延ばす
        for (id, v) in core.iter_ref() {
            if let Ok(iter) = id.extrude_fxy(
                self.target_z.get(),
                self.f_range,
                self.x_range,
                self.y_range,
            ) {
                for new_id in iter {
                    extruded.push((new_id, v.clone()));
                }
            }
        }

        // 重複や競合を解決するため、IDでソートする
        #[cfg(feature = "rayon")]
        extruded.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        #[cfg(not(feature = "rayon"))]
        extruded.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        // 連続する同じIDのグループごとに resolve_many を適用
        let mut new_items = Vec::with_capacity(extruded.len());
        for chunk in extruded.chunk_by(|a, b| a.0 == b.0) {
            let id = chunk[0].0.clone();
            if let Some(merged) = P::resolve_many(chunk.iter().map(|(_, v)| v.clone())) {
                new_items.push((id, merged));
            }
        }

        // 重複のない (FlexId, V) のリストからツリーを再構築
        *core = W::from_items(new_items);

        Ok(())
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::absolute_target::<P>(P::IS_COMMUTATIVE)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn try_merge(
        &self,
        _other: &dyn UnaryOperator<W>,
    ) -> Option<alloc::boxed::Box<dyn UnaryOperator<W>>> {
        // マージ（FXY化）による次元の呪い（Cullingの遅延）を防ぐため、
        // 意図的にマージを無効化し、各軸ごとに逐次適用・合成させる。
        // MergeAccumulatorのTrait実装自体は構造的に残している。
        None
    }

    fn expansion_ratio(&self) -> f32 {
        let f = self
            .f_range
            .map(|(s, e)| (s.abs_diff(e) + 1) as f32)
            .unwrap_or(1.0);
        let x = self
            .x_range
            .map(|(s, e)| (s.abs_diff(e) + 1) as f32)
            .unwrap_or(1.0);
        let y = self
            .y_range
            .map(|(s, e)| (s.abs_diff(e) + 1) as f32)
            .unwrap_or(1.0);
        f * x * y
    }

    fn effective_expansion_ratio(&self, bbox: Option<&crate::RangeId>) -> f32 {
        if let Some(bb) = bbox {
            let x = bb.x();
            let y = bb.y();
            let f = bb.f();

            let s_x = x[1].saturating_sub(x[0]) as f32 + 1.0;
            let s_y = y[1].saturating_sub(y[0]) as f32 + 1.0;
            let s_f = f[1].saturating_sub(f[0]).max(0) as f32 + 1.0;

            let e_x = if self.x_range.is_some() {
                self.x_range
                    .map(|(s, e)| (s.abs_diff(e) + 1) as f32)
                    .unwrap_or(1.0)
                    / s_x
            } else {
                1.0
            };
            let e_y = if self.y_range.is_some() {
                self.y_range
                    .map(|(s, e)| (s.abs_diff(e) + 1) as f32)
                    .unwrap_or(1.0)
                    / s_y
            } else {
                1.0
            };
            let e_f = if self.f_range.is_some() {
                self.f_range
                    .map(|(s, e)| (s.abs_diff(e) + 1) as f32)
                    .unwrap_or(1.0)
                    / s_f
            } else {
                1.0
            };

            e_x * e_y * e_f
        } else {
            <Self as UnaryOperator<W>>::expansion_ratio(self)
        }
    }
}

fn merge_extrude_axis<T: PartialEq>(
    cur: Option<(T, T)>,
    add: Option<(T, T)>,
) -> Option<Option<(T, T)>> {
    match (cur, add) {
        (Some(c), Some(a)) if c == a => Some(Some(c)),
        (Some(_), Some(_)) => None, // Conflict
        (Some(c), None) => Some(Some(c)),
        (None, Some(a)) => Some(Some(a)),
        (None, None) => Some(None),
    }
}

impl<W, P> crate::spatial_id::collection::query::traits::MergeAccumulator<W> for ExtrudeFXY<P>
where
    W: WorkingTree + 'static,
    P: MergePolicy<W::Value> + 'static,
{
    fn seed(op: &dyn UnaryOperator<W>) -> Option<Self> {
        let any = op.as_any();
        if let Some(o) = any.downcast_ref::<ExtrudeFXY<P>>() {
            return Some(Self {
                target_z: o.target_z,
                f_range: o.f_range,
                x_range: o.x_range,
                y_range: o.y_range,
                _marker: core::marker::PhantomData,
            });
        }
        if let Some(o) = any.downcast_ref::<super::extrude_x::ExtrudeX<P>>() {
            return Some(Self {
                target_z: o.target_z,
                f_range: None,
                x_range: Some((o.start_x, o.end_x)),
                y_range: None,
                _marker: core::marker::PhantomData,
            });
        }
        if let Some(o) = any.downcast_ref::<super::extrude_y::ExtrudeY<P>>() {
            return Some(Self {
                target_z: o.target_z,
                f_range: None,
                x_range: None,
                y_range: Some((o.start_y, o.end_y)),
                _marker: core::marker::PhantomData,
            });
        }
        if let Some(o) = any.downcast_ref::<super::extrude_f::ExtrudeF<P>>() {
            return Some(Self {
                target_z: o.target_z,
                f_range: Some((o.start_f, o.end_f)),
                x_range: None,
                y_range: None,
                _marker: core::marker::PhantomData,
            });
        }
        None
    }

    fn absorb(&mut self, op: &dyn UnaryOperator<W>) -> bool {
        let Some(delta) =
            <Self as crate::spatial_id::collection::query::traits::MergeAccumulator<W>>::seed(op)
        else {
            return false;
        };
        if self.target_z != delta.target_z {
            return false;
        }
        let (Some(f), Some(x), Some(y)) = (
            merge_extrude_axis(self.f_range, delta.f_range),
            merge_extrude_axis(self.x_range, delta.x_range),
            merge_extrude_axis(self.y_range, delta.y_range),
        ) else {
            return false;
        };
        self.f_range = f;
        self.x_range = x;
        self.y_range = y;
        true
    }
}
