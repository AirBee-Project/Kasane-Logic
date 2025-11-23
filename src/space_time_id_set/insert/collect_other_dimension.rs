use crate::{
    bit_vec::{BitVec, relation::BitVecRelation},
    encode_id::EncodeID,
    space_time_id_set::{SpaceTimeIDSet, insert::select_dimensions::DimensionSelect},
};

impl SpaceTimeIDSet {
    pub(crate) fn collect_other_dimension(
        dim: &BitVec,
        dim_select: &DimensionSelect,
        top_reverse: &Vec<&EncodeID>,
        under_reverse: &Vec<&EncodeID>,
    ) -> Option<(Vec<BitVecRelation>, Vec<BitVecRelation>)> {
        let mut top_unrelated = true;
        let mut under_unrelated = true;

        let mut top_relation: Vec<BitVecRelation> = Vec::new();
        let mut under_relation: Vec<BitVecRelation> = Vec::new();

        for top in top_reverse {
            let target = match dim_select {
                DimensionSelect::F => &top.f,
                DimensionSelect::X => &top.x,
                DimensionSelect::Y => &top.y,
            };

            let relation = dim.relation(target);

            if relation != BitVecRelation::Unrelated {
                top_unrelated = false;
            }

            top_relation.push(relation);
        }

        for under in under_reverse {
            let target = match dim_select {
                DimensionSelect::F => &under.f,
                DimensionSelect::X => &under.x,
                DimensionSelect::Y => &under.y,
            };

            let relation = dim.relation(target);

            if relation != BitVecRelation::Unrelated {
                under_unrelated = false;
            }

            under_relation.push(relation);
        }

        if top_unrelated && under_unrelated {
            return None;
        } else {
            return Some((top_relation, under_relation));
        }
    }
}
