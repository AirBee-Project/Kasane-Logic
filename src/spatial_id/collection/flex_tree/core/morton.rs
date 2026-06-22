use crate::FlexId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MortonCode {
    pub code: u128,
    pub depth: u8,
}

impl MortonCode {
    /// FlexIdからモートン符号を計算します。
    ///
    /// このモートン符号は `FlexTreeCore` (F, X, Y の交互の分岐によるBinary Tree)
    /// の走査経路と完全に一致する Z-Order 符号を生成します。
    pub fn from_flex_id(id: &FlexId) -> Self {
        let align_index = |index: i64, z: u8| -> u64 {
            if z == 0 {
                return 0;
            }
            let mask = (1u64 << z) - 1;
            ((index as u64) & mask) << (30 - z)
        };

        let f_bits = align_index(id.f_index() as i64, id.f_zoomlevel());
        let x_bits = align_index(id.x_index() as i64, id.x_zoomlevel());
        let y_bits = align_index(id.y_index() as i64, id.y_zoomlevel());

        let mut morton: u128 = 0;
        for i in 0..30 {
            let bit_idx = 29 - i;
            let f_bit = ((f_bits >> bit_idx) & 1) as u128;
            let x_bit = ((x_bits >> bit_idx) & 1) as u128;
            let y_bit = ((y_bits >> bit_idx) & 1) as u128;

            morton = (morton << 3) | (f_bit << 2) | (x_bit << 1) | y_bit;
        }

        let f_sign: u128 = if id.f_index() >= 0 { 1 } else { 0 };
        let code = (f_sign << 90) | morton;

        let depth = id.f_zoomlevel() + id.x_zoomlevel() + id.y_zoomlevel();

        Self { code, depth }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
    use crate::spatial_id::flex_id::FlexId;

    #[test]
    fn test_morton_code_matches_tree_order() {
        // Create random or systematic FlexIds
        let mut ids = unsafe {
            vec![
                FlexId::new_unchecked(1, 0, 1, 0, 1, 0), // (F>=0, F=0, X=0, Y=0)
                FlexId::new_unchecked(1, 1, 1, 0, 1, 0), // (F>=0, F=1, X=0, Y=0)
                FlexId::new_unchecked(1, 0, 1, 1, 1, 0), // (F>=0, F=0, X=1, Y=0)
                FlexId::new_unchecked(1, 0, 1, 0, 1, 1), // (F>=0, F=0, X=0, Y=1)
                FlexId::new_unchecked(1, -1, 1, 0, 1, 0), // (F<0, F=-1)
                FlexId::new_unchecked(1, -2, 1, 0, 1, 0), // (F<0, F=-2)
                FlexId::new_unchecked(2, 3, 2, 3, 2, 3), // (F=3, X=3, Y=3)
            ]
        };

        // Insert into FlexTreeCore to get the tree's natural traversal order
        let mut tree = FlexTreeCore::new();
        for (i, id) in ids.iter().enumerate() {
            tree.insert(id.clone(), i);
        }

        let tree_order: Vec<FlexId> = tree.iter().map(|(id, _)| id).collect();

        // Sort by MortonCode
        ids.sort_by_key(MortonCode::from_flex_id);

        // The two orders must perfectly match
        assert_eq!(
            ids, tree_order,
            "Morton Code sorting must perfectly match FlexTreeCore DFS traversal order"
        );
    }
}
