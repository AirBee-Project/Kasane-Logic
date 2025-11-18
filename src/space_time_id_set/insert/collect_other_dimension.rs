use crate::{
    bit_vec::BitVec,
    space_time_id_set::{
        ReverseInfo, SpaceTimeIdSet,
        insert::{check_relation::Relation, insert_main_dim::DimensionSelect},
    },
};

impl SpaceTimeIdSet {
    pub fn collect_other_dimension(
        dim: &BitVec,
        dim_select: DimensionSelect,
        top_reverse: &Vec<&ReverseInfo>,
        under_reverse: &Vec<&ReverseInfo>,
    ) -> Option<(Vec<Relation>, Vec<Relation>)> {
        let mut top_disjoint = true;
        let mut under_disjoint = true;

        let mut top_relation: Vec<Relation> = Vec::new();
        let mut under_relation: Vec<Relation> = Vec::new();

        //代表次元における上位範囲を調べる

        for top in top_reverse {
            let target = match dim_select {
                DimensionSelect::F => &top.f,
                DimensionSelect::X => &top.x,
                DimensionSelect::Y => &top.y,
            };

            let relation = Self::check_relation(dim, target);

            if relation != Relation::Disjoint {
                top_disjoint = false;
            }

            top_relation.push(relation);
        }

        for under in under_reverse {
            let target = match dim_select {
                DimensionSelect::F => &under.f,
                DimensionSelect::X => &under.x,
                DimensionSelect::Y => &under.y,
            };

            let relation = Self::check_relation(dim, target);

            if relation != Relation::Disjoint {
                under_disjoint = false;
            }

            under_relation.push(relation);
        }

        println!("Top  :{:?}", top_relation);
        println!("Under:{:?}", under_relation);

        if top_disjoint && under_disjoint {
            None
        } else {
            Some((top_relation, under_relation))
        }
    }
}
