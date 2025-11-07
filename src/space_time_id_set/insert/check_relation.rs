use std::collections::HashSet;

use crate::{
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
    r#type::bit_vec::BitVec,
};

///Me（自身）から見た視点の結果
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Relation {
    Top,
    Bottom,
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
            return Relation::Bottom;
        } else {
            return Relation::Disjoint;
        }
    }
}
