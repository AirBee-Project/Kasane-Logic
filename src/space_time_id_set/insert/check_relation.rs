use crate::{bit_vec::BitVec, space_time_id_set::SpaceTimeIdSet};

///相手と自分を比べたときの自分のサイズ
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Relation {
    Top,
    Under,
    Disjoint,
}

impl SpaceTimeIdSet {
    ///mainの上位IDについて逆引き検索する関数
    pub fn check_relation(me: &BitVec, target: &BitVec) -> Relation {
        let me_range = me.under_prefix();
        let target_range = target.under_prefix();

        if target == me {
            println!("EQUAL");
            Relation::Top
        } else if (me_range.0 < *target) && (target < &me_range.1) {
            Relation::Top
        } else if (target_range.0 < *me) && (me < &target_range.1) {
            Relation::Under
        } else {
            Relation::Disjoint
        }
    }
}
