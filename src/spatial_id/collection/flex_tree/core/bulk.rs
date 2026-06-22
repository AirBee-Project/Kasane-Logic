use super::FlexTreeCore;
use crate::FlexId;
use crate::spatial_id::collection::flex_tree::core::morton::MortonCode;
use crate::spatial_id::collection::flex_tree::core::node::Node;
use crate::spatial_id::collection::flex_tree::core::ptr::SharedNode;

impl<V> FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    /// Z-Order (Morton Code) でソート済みの配列から、Octree (Binary Trie) をボトムアップで一括構築します。
    ///
    /// 既存の `insert` メソッドはトップダウンで空間を走査するため、ソートされていないデータを挿入すると
    /// 連鎖的なツリー分割と `Arc` アロケーションが発生し O(N log N) の重いオーバーヘッドがかかります。
    ///
    /// 本メソッドは、ソート済みのデータを $O(N \log N)$ (各レベルでの二分探索) で処理し、
    /// 不要なツリー走査や再帰的な分割を完全に排除するため、数百万要素のツリーを数ミリ秒〜数十ミリ秒で構築可能です。
    pub(crate) fn from_sorted_batch(
        slice: &[(MortonCode, FlexId, V)],
        empty_leaf: &SharedNode<Node<V>>,
    ) -> (SharedNode<Node<V>>, SharedNode<Node<V>>) {
        let sign_split = slice.partition_point(|x| (x.0.code >> 90) & 1 == 0);
        let lower = Self::build_tree(&slice[..sign_split], 0, empty_leaf);
        let upper = Self::build_tree(&slice[sign_split..], 0, empty_leaf);
        (lower, upper)
    }

    fn build_tree(
        slice: &[(MortonCode, FlexId, V)],
        level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Node<V>> {
        if slice.is_empty() {
            return empty_leaf.clone();
        }

        if slice.len() == 1 {
            let target = &slice[0].1;
            if Node::<V>::completely_covers_public(target, level) {
                return SharedNode::new(Node::Leaf {
                    value: Some(slice[0].2.clone()),
                });
            }
        }

        if level >= 90 {
            return SharedNode::new(Node::Leaf {
                value: Some(slice[0].2.clone()),
            });
        }

        let bit_idx = 89 - level;
        let split = slice.partition_point(|x| (x.0.code >> bit_idx) & 1 == 0);

        let lower_child = Self::build_tree(&slice[..split], level + 1, empty_leaf);
        let upper_child = Self::build_tree(&slice[split..], level + 1, empty_leaf);

        if let (Node::Leaf { value: Some(v1) }, Node::Leaf { value: Some(v2) }) =
            (&*lower_child, &*upper_child)
            && v1 == v2
        {
            return SharedNode::new(Node::Leaf {
                value: Some(v1.clone()),
            });
        }

        SharedNode::new(Node::Branch {
            level,
            leaf_count: lower_child.leaf_count() + upper_child.leaf_count(),
            max_zoom: Node::<V>::fold_max_zoom(level, &lower_child, &upper_child),
            lower_child,
            upper_child,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlexId;
    use alloc::vec;

    #[test]
    fn test_from_sorted_batch() {
        let ids = unsafe {
            vec![
                (FlexId::new_unchecked(1, 0, 1, 0, 1, 0), 10),
                (FlexId::new_unchecked(1, 1, 1, 0, 1, 0), 20),
                (FlexId::new_unchecked(1, 0, 1, 1, 1, 0), 30),
                (FlexId::new_unchecked(1, 0, 1, 0, 1, 1), 40),
                (FlexId::new_unchecked(1, -1, 1, 0, 1, 0), 50),
                (FlexId::new_unchecked(1, -2, 1, 0, 1, 0), 60),
                (FlexId::new_unchecked(2, 3, 2, 3, 2, 3), 70),
            ]
        };

        // 1. Build sequentially
        let mut sequential_tree = FlexTreeCore::new();
        for (id, val) in &ids {
            sequential_tree.insert(id.clone(), *val);
        }

        // 2. Build via batch
        let mut batch_items: Vec<_> = ids
            .into_iter()
            .map(|(id, val)| (MortonCode::from_flex_id(&id), id, val))
            .collect();
        batch_items.sort_by_key(|x| x.0);

        let mut batch_tree = FlexTreeCore::new();
        let (lower, upper) = FlexTreeCore::from_sorted_batch(&batch_items, &batch_tree.empty_leaf);
        batch_tree.lower_root = lower;
        batch_tree.upper_root = upper;

        // Verify identical structure!
        if sequential_tree != batch_tree {
            println!("Sequential:\n{:#?}", sequential_tree);
            println!("Batch:\n{:#?}", batch_tree);
            panic!("Batch built tree must be structurally identical to sequentially built tree");
        }
    }
}
