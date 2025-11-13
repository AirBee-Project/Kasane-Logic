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
        let (me_start, me_end) = match *me < me.under_prefix() {
            true => (me.clone(), me.under_prefix()),
            false => (me.under_prefix(), me.clone()),
        };

        let (target_start, target_end) = match *target < target.under_prefix() {
            true => (target.clone(), target.under_prefix()),
            false => (target.under_prefix(), target.clone()),
        };

        if target == me {
            return Relation::Top;
        } else if (me_start < *target) && (target < &me_end) {
            return Relation::Top;
        } else if (target_start < *me) && (me < &target_end) {
            return Relation::Under;
        } else {
            return Relation::Disjoint;
        }
    }
}
