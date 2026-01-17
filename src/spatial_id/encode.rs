use crate::spatial_id::{
    range::RangeId,
    segment::{
        Segment,
        encode::{EncodeSegment, SegmentRelation},
    },
};

#[derive(Clone, PartialEq)]
pub struct EncodeId {
    pub(crate) f: EncodeSegment,
    pub(crate) x: EncodeSegment,
    pub(crate) y: EncodeSegment,
}

pub enum EncodeIdRelation {
    Disjoint,
    Intersecting,
}

impl EncodeId {
    ///[RangeId]に戻す
    pub fn decode(&self) -> RangeId {
        let f_seg = Segment::<i32>::from(self.f.clone());
        let x_seg = Segment::<u32>::from(self.x.clone());
        let y_seg = Segment::<u32>::from(self.y.clone());

        let max_z = f_seg.as_z().max(x_seg.as_z().max(y_seg.as_z()));

        let scale_to_range = |val: i64, current_z: u8, target_z: u8| -> [i64; 2] {
            let diff = target_z - current_z;
            let scale = 1_i64 << diff;

            let start = val * scale;
            let end = start + scale - 1;

            [start, end]
        };

        let f_range = scale_to_range(f_seg.as_dimension() as i64, f_seg.as_z(), max_z);
        let x_range = scale_to_range(x_seg.as_dimension() as i64, x_seg.as_z(), max_z);
        let y_range = scale_to_range(y_seg.as_dimension() as i64, y_seg.as_z(), max_z);

        RangeId {
            z: max_z,
            f: [f_range[0] as i32, f_range[1] as i32],
            x: [x_range[0] as u32, x_range[1] as u32],
            y: [y_range[0] as u32, y_range[1] as u32],
        }
    }

    pub fn as_f(&self) -> &EncodeSegment {
        &self.f
    }

    pub fn as_x(&self) -> &EncodeSegment {
        &self.x
    }

    pub fn as_y(&self) -> &EncodeSegment {
        &self.y
    }

    ///EncodeId同士の関連を返す関数
    pub fn relation(&self, other: &EncodeId) -> EncodeIdRelation {
        let f_relation = self.as_f().relation(other.as_f());
        let x_relation = self.as_x().relation(other.as_x());
        let y_relation = self.as_y().relation(other.as_y());

        if f_relation == SegmentRelation::Disjoint
            || x_relation == SegmentRelation::Disjoint
            || y_relation == SegmentRelation::Disjoint
        {
            EncodeIdRelation::Disjoint
        } else {
            EncodeIdRelation::Intersecting
        }
    }

    pub fn intersection(&self, other: &EncodeId) -> Option<EncodeId> {
        let f = match self.as_f().relation(other.as_f()) {
            SegmentRelation::Equal => self.as_f(),
            SegmentRelation::Ancestor => other.as_f(),
            SegmentRelation::Descendant => self.as_f(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let x = match self.as_x().relation(other.as_x()) {
            SegmentRelation::Equal => self.as_x(),
            SegmentRelation::Ancestor => other.as_x(),
            SegmentRelation::Descendant => self.as_x(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        let y = match self.as_y().relation(other.as_y()) {
            SegmentRelation::Equal => self.as_y(),
            SegmentRelation::Ancestor => other.as_y(),
            SegmentRelation::Descendant => self.as_y(),
            SegmentRelation::Disjoint => {
                return None;
            }
        };

        Some(EncodeId {
            f: f.clone(),
            x: x.clone(),
            y: y.clone(),
        })
    }

    /// 差集合 (self - other) を計算する。
    ///
    /// 空間的に重なりがない場合は `vec![self]` を返す。
    /// `other` が `self` を完全に覆っている場合は `vec![]` (空) を返す。
    /// それ以外の場合、`self` の領域から `other` との重なりを除外した断片のリストを返す。
    pub fn difference(&self, other: &EncodeId) -> Vec<EncodeId> {
        // 1. まず共通部分 (Intersection) を求める
        //    A - B は、A - (A ∩ B) と等価であるため。
        let intersection = match self.intersection(other) {
            Some(i) => i,
            None => return vec![self.clone()], // 排反ならAそのまま
        };

        // 2. 共通部分が Self と完全に一致する場合 (Self ⊆ Other)
        //    全て削り取られるので、結果は空。
        if *self == intersection {
            return vec![];
        }

        let mut result = Vec::new();

        // 3. 3次元的なくり抜き処理 (Decomposition)
        //    intersection (hole) は必ず self (base) の内側にあります。
        //    以下の3つの領域に分割して収集します。

        // Area 1: F次元の差分 (X, Yは Base のまま)
        // 範囲: (Base.f - Hole.f) × Base.x × Base.y
        let f_diffs = Self::segment_difference_one_way(&self.f, &intersection.f);
        for f_seg in f_diffs {
            result.push(EncodeId {
                f: f_seg,
                x: self.x.clone(),
                y: self.y.clone(),
            });
        }

        // Area 2: X次元の差分 (Fは Hole に制限, Yは Base のまま)
        // 範囲: Hole.f × (Base.x - Hole.x) × Base.y
        let x_diffs = Self::segment_difference_one_way(&self.x, &intersection.x);
        for x_seg in x_diffs {
            result.push(EncodeId {
                f: intersection.f.clone(), // FはHoleに合わせる
                x: x_seg,
                y: self.y.clone(),
            });
        }

        // Area 3: Y次元の差分 (F, Xは Hole に制限)
        // 範囲: Hole.f × Hole.x × (Base.y - Hole.y)
        let y_diffs = Self::segment_difference_one_way(&self.y, &intersection.y);
        for y_seg in y_diffs {
            result.push(EncodeId {
                f: intersection.f.clone(), // FはHoleに合わせる
                x: intersection.x.clone(), // XもHoleに合わせる
                y: y_seg,
            });
        }

        result
    }

    fn segment_difference_one_way(
        base: &EncodeSegment,
        hole: &EncodeSegment,
    ) -> Vec<EncodeSegment> {
        if base == hole {
            return vec![];
        }

        let mut results = Vec::new();
        let mut current = hole.clone();

        // hole から base に到達するまで、親を辿りながら「兄弟」を収集する
        // これにより、baseの内側で hole の外側にある領域を網羅できる
        while &current != base {
            // 兄弟がいれば追加 (兄弟 = holeの親から見て、holeではない方の分岐)
            results.push(current.sibling());

            // 親へ移動
            match current.parent() {
                Some(p) => current = p,
                None => break, // ここには来ないはず (baseに到達するはず)
            }
        }

        results
    }

    pub fn merge() -> impl Iterator<Item = EncodeId> {
        std::iter::empty()
    }
}
