use crate::{
    bit_vec::{HierarchicalKey, relation::HierarchicalKeyRelation},
    space_time_id_set::{ReverseInfo, SpaceTimeIdSet, insert::select_dimensions::DimensionSelect},
};

impl SpaceTimeIdSet {
    pub(crate) fn collect_other_dimension(
        dim: &HierarchicalKey,
        dim_select: &DimensionSelect,
        top_reverse: &Vec<&ReverseInfo>,
        under_reverse: &Vec<&ReverseInfo>,
    ) -> Option<(Vec<HierarchicalKeyRelation>, Vec<HierarchicalKeyRelation>)> {
        let mut top_unrelated = true;
        let mut under_unrelated = true;

        let mut top_relation: Vec<HierarchicalKeyRelation> = Vec::new();
        let mut under_relation: Vec<HierarchicalKeyRelation> = Vec::new();

        for top in top_reverse {
            let target = match dim_select {
                DimensionSelect::F => &top.f,
                DimensionSelect::X => &top.x,
                DimensionSelect::Y => &top.y,
            };

            let relation = dim.relation(target);

            if relation != HierarchicalKeyRelation::Unrelated {
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

            if relation != HierarchicalKeyRelation::Unrelated {
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
