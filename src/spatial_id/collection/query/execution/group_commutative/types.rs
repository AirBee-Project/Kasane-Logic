use core::any::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetAxis {
    X,
    Y,
    F,
    FXY,
}

/// 単項演算子の数式的なパターン。可換性判定はこのパターンが一致するかどうかで決まる。
/// 新しい数式パターンの演算子を追加する場合はここに列挙子を追加する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorPattern {
    /// 「初期値 - f(d)」の形式をとる、空間操作が独立して行える変換。
    /// `need_merge_policy` が `false` なら単射（移動のみ）、`true` なら値の伝播など衝突が発生しポリシー解決が必要な操作。
    Separable { need_merge_policy: bool },
    /// 絶対座標の固定範囲へ値を写す変換（シフト同変ではない。ソースを平行移動しても出力範囲は追従しない）。`MergePolicy` で衝突解決する。異なる軸への適用同士は可換だが、同一軸への適用は不可換。
    AbsoluteTarget { axis: TargetAxis },

    /// 上記のいずれにも当てはまらない・可換性を主張しない。例: ZoomOut, FillEmpty。
    Other,
}

/// 可換グループのキー。同じキーを持つ演算子同士だけ**が自由に並べ替え候補となる。
///
/// `pattern` が異なれば（例: `AbsoluteTarget` と `Separable`）、たとえ `MergePolicy` の一致していても可換とはみなさない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CommutativityKey {
    pattern: OperatorPattern,
    /// このパターンが `MergePolicy` を使う場合の型ID。衝突が発生しない操作では `None`。
    policy: Option<TypeId>,
}

/// 演算子から取得する可換性メタデータ。`None` は「並べ替え不可（他のどの演算子ともグループ化しない）」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommutativityInfo(Option<CommutativityKey>);

impl CommutativityInfo {
    /// 並べ替え不可を表す。例: ZoomOut, FillEmpty。
    pub fn none() -> Self {
        Self(None)
    }

    /// 「初期値 - f(d)」の形式で、衝突が発生しない（単射である）操作（Shiftなど）。
    pub fn separable_injective() -> Self {
        Self(Some(CommutativityKey {
            pattern: OperatorPattern::Separable {
                need_merge_policy: false,
            },
            policy: None,
        }))
    }

    /// 「初期値 - f(d)」の形式で、衝突が発生し集約が必要な操作（FalloffLinearなど）。
    pub fn separable_with_policy<P: 'static>(policy_is_commutative: bool) -> Self {
        if !policy_is_commutative {
            return Self::none();
        }
        Self(Some(CommutativityKey {
            pattern: OperatorPattern::Separable {
                need_merge_policy: true,
            },
            policy: Some(TypeId::of::<P>()),
        }))
    }

    /// 絶対座標の固定範囲への変換（Extrude等）を表す。`policy_is_commutative` が `false` なら
    /// 並べ替え不可を返す。
    pub fn absolute_target<P: 'static>(axis: TargetAxis, policy_is_commutative: bool) -> Self {
        Self::with_policy::<P>(
            OperatorPattern::AbsoluteTarget { axis },
            policy_is_commutative,
        )
    }

    fn with_policy<P: 'static>(pattern: OperatorPattern, policy_is_commutative: bool) -> Self {
        if !policy_is_commutative {
            return Self::none();
        }
        Self(Some(CommutativityKey {
            pattern,
            policy: Some(TypeId::of::<P>()),
        }))
    }

    /// 自身が可換性を持ち得るか（他とグループ化される可能性があるか）。
    pub fn is_potentially_commutative(&self) -> bool {
        self.0.is_some()
    }

    pub fn can_commute_with(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => {
                match (a.pattern, b.pattern) {
                    (
                        OperatorPattern::Separable {
                            need_merge_policy: c1,
                        },
                        OperatorPattern::Separable {
                            need_merge_policy: c2,
                        },
                    ) => {
                        // policyが一致していれば常に可換。
                        // 一致していなくても、どちらかが単射(need_merge_policy=false)なら可換。
                        a.policy == b.policy || !c1 || !c2
                    }
                    (
                        OperatorPattern::AbsoluteTarget { axis: axis1 },
                        OperatorPattern::AbsoluteTarget { axis: axis2 },
                    ) => {
                        // 異なる軸への Extrude 同士は可換（FXYはいずれとも干渉するため不可換）
                        // ただし、policyが一致している必要がある。
                        a.policy == b.policy
                            && axis1 != axis2
                            && axis1 != TargetAxis::FXY
                            && axis2 != TargetAxis::FXY
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
