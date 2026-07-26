use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::{FlexId, SingleId, SpatialIdTable};

#[cfg(feature = "rayon")]
use crate::CellValue;

impl<V> SpatialIdTable<V>
where
    V: SafeValue + Ord,
{
    pub fn flex_ids(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.inner.single_ids()
    }
}

/// `(FlexId, V)` 列から [`SpatialIdTable`] を並列に構築する（`feature = "rayon"`）。
///
/// テーブルは値をランク（`usize`）へ内部符号化してから空間ツリーへ格納する。並列構築では
/// (1) 出現値を並列に集めて重複排除・ソートしランクを決定的に割り当て、(2) 各セルを
/// ランクへ写し、(3) ランクの木を [`FlexTreeCore::par_build_vec`](crate::FlexTreeCore::par_build_vec)
/// で並列構築する。
///
/// 同じ空間へ異なる値が重なった場合の勝者は `union` の左優先で決まり、逐次 `insert` の
/// 後勝ちとは一致しない（値が衝突しない使い方なら結果は一意）。
#[cfg(feature = "rayon")]
impl<V> rayon::iter::FromParallelIterator<(FlexId, V)> for SpatialIdTable<V>
where
    V: CellValue,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use alloc::collections::BTreeMap;
        use alloc::vec::Vec;
        use rayon::prelude::*;

        let items: Vec<(FlexId, V)> = par_iter.into_par_iter().collect();
        if items.is_empty() {
            return Self::new();
        }

        // 1. 出現値を並列に集め、ソート＋重複排除して決定的なランク順を得る。
        let mut values: Vec<V> = items.par_iter().map(|(_, v)| v.clone()).collect();
        values.par_sort_unstable();
        values.dedup();

        // 2. 値 ⇄ ランク（1始まり）の双方向辞書を作る。
        let mut dictionary = BTreeMap::new();
        let mut reverse_dictionary = BTreeMap::new();
        for (i, v) in values.iter().enumerate() {
            let rank = i + 1;
            dictionary.insert(v.clone(), rank);
            reverse_dictionary.insert(rank, v.clone());
        }
        let current_rank = values.len();

        // 3. 各セルをランクへ写す（ソート済み values への二分探索で引く）。
        let rank_items: Vec<(FlexId, usize)> = items
            .into_par_iter()
            .map(|(id, v)| (id, values.binary_search(&v).unwrap() + 1))
            .collect();

        // 4. ランクの木を並列構築。値インデックスは未構築（`insert` 後と同じ状態）。
        let mut table = Self::new();
        table.inner = crate::FlexTreeCore::par_build_vec(rank_items);
        table.dictionary = dictionary;
        table.reverse_dictionary = reverse_dictionary;
        table.current_rank = current_rank;
        table.value_index_built = false;
        table
    }
}

/// 既存の [`SpatialIdTable`] へ `(FlexId, V)` 列を並列にマージする（`feature = "rayon"`）。
///
/// 別テーブルを並列構築したのち、ランク空間が異なるため逐次に再挿入して統合する。
#[cfg(feature = "rayon")]
impl<V> rayon::iter::ParallelExtend<(FlexId, V)> for SpatialIdTable<V>
where
    V: CellValue,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::iter::FromParallelIterator;
        let other = Self::from_par_iter(par_iter);
        for (id, value) in other {
            self.insert(id, value);
        }
    }
}
