use core::any::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetAxis {
    X,
    Y,
    F,
    FXY,
}

/// 演算子から取得する可換性メタデータ。同じメタデータを持つ演算子同士だけが並べ替え候補となる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommutativityInfo {
    /// 並べ替え不可（他のどの演算子ともグループ化しない）
    None,

    Separable {
        /// このパターンが `MergePolicy` を使う場合の型ID。衝突が発生しない操作（単射）では `None`。
        policy: Option<TypeId>,
    },

    /// 絶対座標の固定範囲へ値を写す変換
    AbsoluteTarget {
        axis: TargetAxis,
        /// このパターンが `MergePolicy` を使う場合の型ID。衝突が発生しない操作では `None`。
        policy: Option<TypeId>,
    },
}

impl CommutativityInfo {
    /// 自身が可換性を持ち得るか（他とグループ化される可能性があるか）。
    pub fn is_potentially_commutative(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn can_commute_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Separable { policy: p1 }, Self::Separable { policy: p2 }) => {
                // policyが一致していれば常に可換。
                // 一致していなくても、どちらかが単射(policyがNone)なら可換。
                p1 == p2 || p1.is_none() || p2.is_none()
            }
            (
                Self::AbsoluteTarget {
                    axis: axis1,
                    policy: p1,
                },
                Self::AbsoluteTarget {
                    axis: axis2,
                    policy: p2,
                },
            ) => {
                // 異なる軸への Extrude 同士は可換（FXYはいずれとも干渉するため不可換）
                // ただし、policyが一致している必要がある。
                p1 == p2 && axis1 != axis2 && *axis1 != TargetAxis::FXY && *axis2 != TargetAxis::FXY
            }
            _ => false,
        }
    }
}
