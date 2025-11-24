use crate::{
    bit_vec::{BitVec, relation::BitVecRelation},
    encode_id::EncodeID,
    encode_id_set::{EncodeIDSet, insert::select_dimensions::DimensionSelect},
};

impl EncodeIDSet {
    pub(crate) fn collect_other_dimension(
        main: &BitVec,
        main_ancestors_reverse: &Vec<&EncodeID>,
        main_descendants_reverse: &Vec<&EncodeID>,
        dim_select: &DimensionSelect,
    ) -> Option<(Vec<BitVecRelation>, Vec<BitVecRelation>)> {
        let mut ancestors_unrelated = true;
        let mut descendants_unrelated = true;

        let mut main_ancestor_relation: Vec<BitVecRelation> = Vec::new();
        let mut main_descendants_relation: Vec<BitVecRelation> = Vec::new();

        for ancestor in main_ancestors_reverse {
            let target = match dim_select {
                DimensionSelect::F => &ancestor.f,
                DimensionSelect::X => &ancestor.x,
                DimensionSelect::Y => &ancestor.y,
            };

            let relation = main.relation(target);

            if relation != BitVecRelation::Unrelated {
                ancestors_unrelated = false;
            }

            main_ancestor_relation.push(relation);
        }

        for descendant in main_descendants_reverse {
            let target = match dim_select {
                DimensionSelect::F => &descendant.f,
                DimensionSelect::X => &descendant.x,
                DimensionSelect::Y => &descendant.y,
            };

            let relation = main.relation(target);

            if relation != BitVecRelation::Unrelated {
                descendants_unrelated = false;
            }

            main_descendants_relation.push(relation);
        }

        if ancestors_unrelated && descendants_unrelated {
            return None;
        } else {
            return Some((main_ancestor_relation, main_descendants_relation));
        }
    }
}
