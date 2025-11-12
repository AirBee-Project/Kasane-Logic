use crate::{bit_vec::BitVec, space_time_id_set::SpaceTimeIdSet};

///Me（自身）から見た視点の結果
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Relation {
    Top,
    Under,
    Disjoint,
}

impl SpaceTimeIdSet {
    ///mainの上位IDについて逆引き検索する関数
    pub fn check_relation(me: &BitVec, target: &BitVec) -> Relation {
        if target == me {
            return Relation::Top;
        } else if (me < target) && (target < &me.under_prefix()) {
            return Relation::Top;
        } else if (target < me) && (me < &target.under_prefix()) {
            return Relation::Under;
        } else {
            return Relation::Disjoint;
        }
    }
}
