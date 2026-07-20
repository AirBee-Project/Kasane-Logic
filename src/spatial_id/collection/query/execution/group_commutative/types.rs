use core::any::TypeId;

/// 演算子の分類。
/// AST最適化において、可換性を持つ可能性がある分類を定義します。
/// ※ ユーザー側で必要に応じて列挙子を拡張してください。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorClass {
    /// 初期値-f(d)で表される最も基本的な変換式
    Separable,
    /// それ以外
    Other,
}

impl OperatorClass {
    /// この分類の演算子が、そもそも可換性を持ち得るか
    pub fn is_potentially_commutative(&self) -> bool {
        matches!(self, OperatorClass::Separable)
    }
}

/// 衝突解決ポリシーの可換性に関する情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyCommutativity {
    /// 衝突が発生しないため常に可換（Shiftなど）
    CollisionFree,
    /// 衝突解決ポリシーが存在し、そのポリシーが可換である（Maxなど）
    Commutative(TypeId),
    /// 衝突解決ポリシーが存在するが、可換ではない
    NonCommutative,
}

/// 演算子から取得する可換性検証用のメタデータ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommutativityInfo {
    pub operator_class: OperatorClass,
    pub policy: PolicyCommutativity,
}

impl CommutativityInfo {
    /// 自身が可換性を持ち得るか（他とグループ化される可能性があるか）
    pub fn is_potentially_commutative(&self) -> bool {
        self.operator_class.is_potentially_commutative()
            && self.policy != PolicyCommutativity::NonCommutative
    }

    /// 2つのメタデータが「同一かつ可換」であるか判定する
    pub fn can_commute_with(&self, other: &Self) -> bool {
        if self.operator_class != other.operator_class
            || !self.operator_class.is_potentially_commutative()
        {
            return false;
        }
        match (&self.policy, &other.policy) {
            (PolicyCommutativity::CollisionFree, PolicyCommutativity::CollisionFree) => true,
            (PolicyCommutativity::Commutative(id1), PolicyCommutativity::Commutative(id2)) => {
                id1 == id2
            }
            _ => false,
        }
    }
}
