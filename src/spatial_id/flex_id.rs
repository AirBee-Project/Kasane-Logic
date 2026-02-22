use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use crate::spatial_id::segment::SegmentRelation;
use crate::spatial_id::{FlexIds, segment::Segment};
use crate::spatial_id::{HyperRect, HyperRectSegments};
use crate::{RangeId, SpatioTemporalId};

/// FlexIdは拡張時空間IDを表す。
///
/// 各インデックスを[Segment]を用いて表すことで、空間3次元と時間1次元において
/// 独立したズームレベル（解像度）を持つことが可能です。
/// 内部的には `[Segment; 4]` として保持されます。
#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord, Hash)]
pub struct FlexId {
    /// 0: F (高度), 1: X (東西), 2: Y (南北), 3: T (時間)
    segments: [Segment; 4],
}

/// [FlexId]同士の関係を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexIdRelation {
    /// 重なりがない。
    Disjoint,
    /// 重なりがある。
    Related,
}

impl FlexId {
    /// 新しく[FlexId]を作成する。
    pub fn new(f: Segment, x: Segment, y: Segment, t: Segment) -> FlexId {
        FlexId {
            segments: [f, x, y, t],
        }
    }

    /// [FlexId]を[RangeId]に変換する。
    pub(crate) fn range_id(&self) -> RangeId {
        self.clone().into()
    }

    /// 高度セグメントを参照する。
    pub fn f(&self) -> &Segment {
        &self.segments[0]
    }
    /// 東西セグメントを参照する。
    pub fn x(&self) -> &Segment {
        &self.segments[1]
    }
    /// 南北セグメントを参照する。
    pub fn y(&self) -> &Segment {
        &self.segments[2]
    }
    /// 時間セグメントを参照する。
    pub fn t(&self) -> &Segment {
        &self.segments[3]
    }
    pub fn segments(&self) -> &[Segment; 4] {
        &self.segments
    }

    /// この時空間ブロックの「体積」（最小単位ボクセルの総数）を返す。
    /// 空間3次元に加え、時間軸の長さも乗算されます。
    pub fn volume(&self) -> u128 {
        let mut total_volume = 1u128;
        for i in 0..4 {
            let (z, _) = if i == 0 {
                self.segments[i].to_f()
            } else {
                let (z, idx) = self.segments[i].to_xy();
                (z, idx as i64) // 型合わせ
            };
            total_volume *= 1u128 << (MAX_ZOOM_LEVEL as u8 - z);
        }
        total_volume
    }

    /// [FlexId]同士の関連を[FlexIdRelation]として返す。
    pub fn relation(&self, other: &FlexId) -> FlexIdRelation {
        for i in 0..4 {
            if self.segments[i].relation(&other.segments[i]) == SegmentRelation::Disjoint {
                return FlexIdRelation::Disjoint;
            }
        }
        FlexIdRelation::Related
    }

    /// [FlexId]同士の重なり合っている部分（交差領域）を返す。
    pub(crate) fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let mut intersect_segments = self.segments.clone();

        for i in 0..4 {
            intersect_segments[i] = match self.segments[i].relation(&other.segments[i]) {
                SegmentRelation::Equal => self.segments[i].clone(),
                SegmentRelation::Ancestor => other.segments[i].clone(),
                SegmentRelation::Descendant => self.segments[i].clone(),
                SegmentRelation::Disjoint => return None,
            };
        }

        Some(FlexId {
            segments: intersect_segments,
        })
    }

    /// [FlexId]から[FlexId]を引き、残った領域を[FlexId]の集合として返す。
    pub fn difference(&self, other: &FlexId) -> Vec<FlexId> {
        let intersection = match self.intersection(other) {
            Some(i) => i,
            None => return vec![self.clone()],
        };

        if *self == intersection {
            return vec![];
        }

        let mut result = Vec::new();
        let mut current_base = self.segments.clone();

        // 次元ごとに差分を抽出し、残りを次の次元のベースにする（再帰的分割）
        for i in 0..4 {
            let diffs = current_base[i].difference(&intersection.segments[i]);
            for seg in diffs {
                let mut new_flex = FlexId {
                    segments: current_base.clone(),
                };
                new_flex.segments[i] = seg;
                result.push(new_flex);
            }
            // 共通部分だけを次の次元の固定土台にする
            current_base[i] = intersection.segments[i].clone();
        }

        result
    }

    /// [FlexId]が相手のFlexIdを完全に含んでいるかどうかを判定する。
    pub fn contains(&self, other: &FlexId) -> bool {
        for i in 0..4 {
            let rel = self.segments[i].relation(&other.segments[i]);
            if rel == SegmentRelation::Disjoint || rel == SegmentRelation::Descendant {
                return false;
            }
        }
        true
    }

    /// 各次元の兄弟[FlexId]を取得するユーティリティ
    pub(crate) fn sibling_at(&self, index: usize) -> FlexId {
        let mut sib = self.clone();
        sib.segments[index] = sib.segments[index].sibling();
        sib
    }

    /// 各次元の親[FlexId]を取得するユーティリティ
    pub(crate) fn parent_at(&self, index: usize) -> Option<FlexId> {
        let mut p = self.clone();
        p.segments[index] = p.segments[index].parent()?;
        Some(p)
    }
}

impl HyperRect for FlexId {
    fn segmentation(&self) -> HyperRectSegments {
        HyperRectSegments {
            segments: [
                vec![self.f().clone()], // 0: F
                vec![self.x().clone()], // 1: X
                vec![self.y().clone()], // 2: Y
                vec![self.t().clone()], // 3: T
            ],
        }
    }
}

impl From<FlexId> for RangeId {
    /// `FlexId` を `RangeId` に変換します。
    /// 空間3次元(F,X,Y)において最も深い解像度に合わせ、範囲を算出します。
    /// 時間(T)については、変換後のRangeIdが持つ時間属性 `t: [u64; 2]` に反映されます。
    fn from(flex_id: FlexId) -> Self {
        let (f_z, f_dim) = flex_id.f().to_f();
        let (x_z, x_dim) = flex_id.x().to_xy();
        let (y_z, y_dim) = flex_id.y().to_xy();
        let (t_z, t_dim) = flex_id.t().to_xy();

        let max_z = f_z.max(x_z).max(y_z);

        // F次元 (i64) スケール
        let f_range = {
            let diff = max_z - f_z;
            let start = f_dim << diff;
            let end = if diff == 0 {
                start
            } else {
                start | ((1_i64 << diff) - 1)
            };
            [start, end]
        };

        // X, Y次元 (u64) スケール
        let scale_u = |val: u64, current_z: u8| -> [u64; 2] {
            let diff = max_z - current_z;
            let start = val << diff;
            let end = if diff == 0 {
                start
            } else {
                start | ((1_u64 << diff) - 1)
            };
            [start, end]
        };

        // T次元 (u64) スケール: MAX_ZOOM_LEVEL基準で絶対時間を算出
        let t_range = {
            let diff = MAX_ZOOM_LEVEL as u8 - t_z;
            let start = t_dim << diff;
            let end = if diff == 0 {
                start
            } else {
                start | ((1_u64 << diff) - 1)
            };
            [start, end]
        };

        let x_range = scale_u(x_dim, x_z);
        let y_range = scale_u(y_dim, y_z);

        let mut rid = unsafe { RangeId::new_unchecked(max_z, f_range, x_range, y_range) };
        rid.set_t(t_range);
        rid
    }
}

/// 内部ビット配列への一括変換トレイト
impl From<FlexId> for [[u8; Segment::ARRAY_LENGTH]; 4] {
    fn from(value: FlexId) -> Self {
        [
            value.segments[0].clone().into(),
            value.segments[1].clone().into(),
            value.segments[2].clone().into(),
            value.segments[3].clone().into(),
        ]
    }
}

impl From<[[u8; Segment::ARRAY_LENGTH]; 4]> for FlexId {
    fn from(value: [[u8; Segment::ARRAY_LENGTH]; 4]) -> Self {
        Self {
            segments: [
                value[0].into(),
                value[1].into(),
                value[2].into(),
                value[3].into(),
            ],
        }
    }
}
