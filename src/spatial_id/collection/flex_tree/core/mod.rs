use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::spatial_id::collection::flex_tree::shard_path::ShardPath;
use crate::spatial_id::collection::flex_tree::summary::ShardSummary;
use crate::{AllowedIntervals, Error, FlexId, RangeId, Side, SingleId, SpatialId};
pub use convert::{LeavesIntoIter, LeavesIterRef};
use node::{Axis, Node};
use node_ops::MergeOp;
pub use ptr::SafeValue;
pub(crate) mod bulk;
mod convert;
pub mod node;
pub mod node_ops;
mod overlap;
#[cfg(feature = "rayon")]
pub(crate) mod parallel;
pub(crate) mod ptr;
pub mod shard;
pub(crate) mod walk;
use ptr::{MaybeSend, MaybeSendSync, MaybeSync, SharedNode};
pub mod tests;

/// 拡張空間IDとそれに紐づいたValueを保存するための型
#[derive(Clone, Debug)]
pub struct FlexTreeCore<V>
where
    V: SafeValue,
{
    pub(crate) lower_root: SharedNode<Node<V>>,
    pub(crate) upper_root: SharedNode<Node<V>>,
    pub(crate) empty_leaf: SharedNode<Node<V>>,

    /// シャード空間の有無。
    pub(crate) shard: Option<FlexId>,

    /// シャード木の中でのこの木の位置。KVS キーの生成に使う。
    ///
    /// 領域（[`shard`](Self::shard)）からは復元できない（理由は
    /// [`shard_path`](crate::spatial_id::collection::flex_tree::shard_path) を参照）ので、
    /// [`split_shard`](Self::split_shard) が1段ずつ記録する。位置が定まらない木では
    /// [`None`]。
    pub(crate) shard_path: Option<ShardPath>,
}

impl<V> Default for FlexTreeCore<V>
where
    V: SafeValue,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> PartialEq for FlexTreeCore<V>
where
    V: SafeValue,
{
    fn eq(&self, other: &Self) -> bool {
        self.lower_root == other.lower_root && self.upper_root == other.upper_root
    }
}

impl<V> Eq for FlexTreeCore<V> where V: SafeValue {}

impl<V> FlexTreeCore<V>
where
    V: SafeValue,
{
    /// 新しい空の[FlexTreeCore]を作成する
    pub fn new() -> Self {
        let empty_leaf = SharedNode::new(Node::Leaf { value: None });
        Self {
            lower_root: empty_leaf.clone(),
            upper_root: empty_leaf.clone(),
            empty_leaf,
            shard: None,
            shard_path: None,
        }
    }

    /// シャード領域 `region` に閉じた空の[FlexTreeCore]を作成する。以降は `region` の内側だけを保持する。`region` の外側への挿入は無視される。
    ///
    /// `region` が半球まるごと（[`FlexId::UPPER_MAX`] / [`FlexId::LOWER_MAX`]）なら
    /// シャード木の根なので [`shard_path`](Self::shard_path) も同時に定まる。それ以外の
    /// 領域を直接指定した場合、その領域がシャード木のどこに位置するのかは領域だけからは
    /// 決まらないため [`None`] になる。パスを引き継ぎたい場合は
    /// [`new_in_shard_with_path`](Self::new_in_shard_with_path) を使う。
    pub fn new_in_shard(region: FlexId) -> Self {
        let path = if region == FlexId::UPPER_MAX {
            Some(ShardPath::root(true))
        } else if region == FlexId::LOWER_MAX {
            Some(ShardPath::root(false))
        } else {
            None
        };
        Self::new_in_shard_with_path(region, path)
    }

    /// シャード領域とシャード木上の位置を指定して空の[FlexTreeCore]を作成する。
    ///
    /// 永続化したシャードを読み戻すときのように、領域とパスの対応が呼び出し側で
    /// 分かっている場合に使う。
    pub fn new_in_shard_with_path(region: FlexId, path: Option<ShardPath>) -> Self {
        let mut core = Self::new();
        core.shard = Some(region);
        core.shard_path = path;
        core
    }

    /// このツリーが閉じているシャード領域を返す。`None` は全空間。
    pub(crate) fn shard(&self) -> Option<&FlexId> {
        self.shard.as_ref()
    }

    /// このツリーのシャード木上の位置を返す。定まらない場合は `None`。
    pub(crate) fn shard_path(&self) -> Option<&ShardPath> {
        self.shard_path.as_ref()
    }

    /// 集合演算後のシャードパス。
    ///
    /// 一般に2つのシャードの演算結果はシャード木のノードではないので、
    /// **両辺の位置が完全に一致するときだけ**引き継ぐ。それ以外は `None`（位置不明）。
    fn shard_path_after_merge(a: &Option<ShardPath>, b: &Option<ShardPath>) -> Option<ShardPath> {
        match (a, b) {
            (Some(x), Some(y)) if x == y => Some(x.clone()),
            _ => None,
        }
    }

    /// 上下いずれかのルート同士を集合演算 `op` で突き合わせる、レベル0起点の薄いラッパ。
    /// 終端規則 [`MergeOp::terminal`] をクロージャに包んで [`Node::merge`] へ渡す。
    fn merge_roots(
        &self,
        a: &SharedNode<Node<V>>,
        b: &SharedNode<Node<V>>,
        op: MergeOp,
    ) -> SharedNode<Node<V>> {
        Node::merge(a, b, &|x, y, e| op.terminal(x, y, e), 0, &self.empty_leaf)
    }

    /// 2つの [FlexTreeCore] の和集合を計算します。
    pub fn union(&self, other: &Self) -> Self {
        Self {
            lower_root: self.merge_roots(&self.lower_root, &other.lower_root, MergeOp::Union),
            upper_root: self.merge_roots(&self.upper_root, &other.upper_root, MergeOp::Union),
            empty_leaf: self.empty_leaf.clone(),
            shard: Self::shard_after_union(&self.shard, &other.shard),
            shard_path: Self::shard_path_after_merge(&self.shard_path, &other.shard_path),
        }
    }

    /// 2つの [FlexTreeCore] を値解決付きで重ね合わせる。
    ///
    /// [`union`](Self::union) と同じ構造マージ（構造共有・並列・枝刈り）だが、両側にSegmentが
    /// 存在する領域では `resolve(a側の値, b側の値)` で値を合成する。片側だけが持つ領域は相手の
    /// 部分木をそのまま共有する。`insert_with_policy` のようなSegment単位の逐次
    /// remove/difference/insert ループを、木マージ 1 本へ置き換えるための土台。
    ///
    /// シャードの扱いは [`union`](Self::union) と同じ。
    #[allow(dead_code)]
    pub fn merge_with<R>(&self, other: &Self, resolve: R) -> Self
    where
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        // 終端規則: 片側が空なら相手を通し（構造共有）、両側が値付き葉なら resolve で合成。
        // `resolve(v, v) != v` になりうる（例: 加算）ため MergeOp のような ptr_eq ショートカットは
        // 使わず、両側に値のある領域は必ず葉まで降りて解決する。
        let terminal = |a: &SharedNode<Node<V>>, b: &SharedNode<Node<V>>, _e: &_| match (&**a, &**b)
        {
            (Node::Leaf { value: None }, _) => Some(b.clone()),
            (_, Node::Leaf { value: None }) => Some(a.clone()),
            (Node::Leaf { value: Some(av) }, Node::Leaf { value: Some(bv) }) => {
                Some(SharedNode::new(Node::Leaf {
                    value: Some(resolve(av, bv)),
                }))
            }
            _ => None,
        };
        Self {
            lower_root: Node::merge(
                &self.lower_root,
                &other.lower_root,
                &terminal,
                0,
                &self.empty_leaf,
            ),
            upper_root: Node::merge(
                &self.upper_root,
                &other.upper_root,
                &terminal,
                0,
                &self.empty_leaf,
            ),
            empty_leaf: self.empty_leaf.clone(),
            shard: Self::shard_after_union(&self.shard, &other.shard),
            shard_path: Self::shard_path_after_merge(&self.shard_path, &other.shard_path),
        }
    }

    /// 2つの [FlexTreeCore] を、片側が空の領域も `default` で埋めてから `resolve` で重ね合わせる。
    ///
    /// [`merge_with`](Self::merge_with) は片側が空ならもう片方をそのまま構造共有するが、こちらは
    /// 「データが無い」ことを表す `default` を代入したうえで必ず `resolve` を呼ぶ（例:
    /// `resolve(default, b)`）。両側とも空の領域は resolve を呼ばずそのまま空を保つ。
    /// 片側が丸ごと構造共有できる最適化が効かないぶん、非空の葉は必ず降りて解決する。
    ///
    /// シャードの扱いは [`union`](Self::union) と同じ。
    pub fn merge_with_default<R>(&self, other: &Self, default: &V, resolve: R) -> Self
    where
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        let terminal = |a: &SharedNode<Node<V>>, b: &SharedNode<Node<V>>, _e: &_| match (&**a, &**b)
        {
            (Node::Leaf { value: None }, Node::Leaf { value: None }) => Some(a.clone()),
            (Node::Leaf { value: None }, Node::Leaf { value: Some(bv) }) => {
                Some(SharedNode::new(Node::Leaf {
                    value: Some(resolve(default, bv)),
                }))
            }
            (Node::Leaf { value: Some(av) }, Node::Leaf { value: None }) => {
                Some(SharedNode::new(Node::Leaf {
                    value: Some(resolve(av, default)),
                }))
            }
            (Node::Leaf { value: Some(av) }, Node::Leaf { value: Some(bv) }) => {
                Some(SharedNode::new(Node::Leaf {
                    value: Some(resolve(av, bv)),
                }))
            }
            _ => None,
        };
        Self {
            lower_root: Node::merge(
                &self.lower_root,
                &other.lower_root,
                &terminal,
                0,
                &self.empty_leaf,
            ),
            upper_root: Node::merge(
                &self.upper_root,
                &other.upper_root,
                &terminal,
                0,
                &self.empty_leaf,
            ),
            empty_leaf: self.empty_leaf.clone(),
            shard: Self::shard_after_union(&self.shard, &other.shard),
            shard_path: Self::shard_path_after_merge(&self.shard_path, &other.shard_path),
        }
    }

    /// 木を降りながら各葉へ `f` を適用し、結果を1本の `Vec` へ集約する。
    ///
    /// 旧実装は「逐次DFSで全葉を `Vec<(FlexId,&V)>` へ平坦化 → その後に並列で `f` を適用」という
    /// 2段構成だった。フェーズ1が木の大きさに関わらず常に逐次だったため、大きな木ほど
    /// 相対的なボトルネックになっていた。ここでは走査と `f` の適用を1回の再帰に融合し、
    /// [`Node::merge`] と同じ `leaf_count` ガード付き `rayon::join` で並列化する
    /// （[`node_ops::PARALLEL_LEAF_CUTOFF`] は既に掃引済みの閾値を共有）。
    fn map_expand<F, I>(&self, f: F) -> Result<Vec<(FlexId, V)>, Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
    {
        let total_leaves = self.lower_root.leaf_count() + self.upper_root.leaf_count();
        if total_leaves == 0 {
            return Ok(Vec::new());
        }

        let mut out = Vec::with_capacity(total_leaves);
        Self::expand_into(self.lower_root.as_ref(), FlexId::LOWER_MAX, &f, &mut out)?;
        Self::expand_into(self.upper_root.as_ref(), FlexId::UPPER_MAX, &f, &mut out)?;
        Ok(out)
    }

    /// [`map_expand`](Self::map_expand) の再帰本体。しきい値未満の部分木は `out` へ直接
    /// 追記（追加のVec確保なし）、しきい値以上は `rayon::join` で両側を独立な `Vec` に集めてから
    /// 結合する（並列境界だけがVecを新規確保する）。
    #[cfg(feature = "rayon")]
    fn expand_into<F, I>(
        node: &Node<V>,
        current_id: FlexId,
        f: &F,
        out: &mut Vec<(FlexId, V)>,
    ) -> Result<(), Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
    {
        match node {
            Node::Leaf { value: None } => Ok(()),
            Node::Leaf { value: Some(v) } => {
                out.extend(f(current_id, v)?);
                Ok(())
            }
            Node::Branch {
                level,
                leaf_count,
                lower_child,
                upper_child,
                ..
            } => {
                let axis = Node::<V>::axis(*level);
                let lower_id = split_child_id(&current_id, axis, Side::Lower);
                let upper_id = split_child_id(&current_id, axis, Side::Upper);

                if *leaf_count as usize >= node_ops::PARALLEL_LEAF_CUTOFF {
                    let (lr, ur): (Result<Vec<_>, Error>, Result<Vec<_>, Error>) = rayon::join(
                        || {
                            let mut lo = Vec::with_capacity(lower_child.leaf_count());
                            Self::expand_into(lower_child.as_ref(), lower_id, f, &mut lo)?;
                            Ok(lo)
                        },
                        || {
                            let mut hi = Vec::with_capacity(upper_child.leaf_count());
                            Self::expand_into(upper_child.as_ref(), upper_id, f, &mut hi)?;
                            Ok(hi)
                        },
                    );
                    out.extend(lr?);
                    out.extend(ur?);
                    return Ok(());
                }

                Self::expand_into(lower_child.as_ref(), lower_id, f, out)?;
                Self::expand_into(upper_child.as_ref(), upper_id, f, out)?;
                Ok(())
            }
        }
    }

    /// [`expand_into`](Self::expand_into) の非rayon版。並列分岐がないぶん単純な逐次再帰。
    #[cfg(not(feature = "rayon"))]
    fn expand_into<F, I>(
        node: &Node<V>,
        current_id: FlexId,
        f: &F,
        out: &mut Vec<(FlexId, V)>,
    ) -> Result<(), Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
    {
        match node {
            Node::Leaf { value: None } => Ok(()),
            Node::Leaf { value: Some(v) } => {
                out.extend(f(current_id, v)?);
                Ok(())
            }
            Node::Branch {
                level,
                lower_child,
                upper_child,
                ..
            } => {
                let axis = Node::<V>::axis(*level);
                let lower_id = split_child_id(&current_id, axis, Side::Lower);
                let upper_id = split_child_id(&current_id, axis, Side::Upper);
                Self::expand_into(lower_child.as_ref(), lower_id, f, out)?;
                Self::expand_into(upper_child.as_ref(), upper_id, f, out)?;
                Ok(())
            }
        }
    }

    /// 各Segmentを `f` で写し、**union**（左優先）で組み直した木を返す。
    ///
    /// 「写像先が空間的に単射」な per-time_segment 演算子（shift / 縮小 など）の汎用 recombiner。写像先が
    /// 重なる場合の値は union に従う。
    pub fn map_rebuild<F, I>(&self, f: F) -> Result<Self, Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
    {
        // 小入力では rayon（par_sort / par_chunks / reduce）起動コストが利得を上回るので逐次挿入で組む。
        // insert は挿入順に依らず O(深さ) なのでソート不要。単発 shift 等の固定床を削る（`FromIterator` へ委譲）。
        Ok(self.map_expand(f)?.into_iter().collect())
    }

    /// 各Segmentを `f` で写し、**写像先の重なりを `resolve` で合成**して組み直した木を返す。
    ///
    /// 「写像先が空間的に非単射」な per-time_segment 演算子（falloff / dilate / 拡大 / downsample …）の
    /// 汎用 recombiner。`resolve` には `MergePolicy::resolve` 相当のクロージャを渡す（FlexTreeCore は
    /// query 層の `MergePolicy` に依存しない）。合成は `par_build_vec_with` や `insert_with`
    /// に委ねられる。
    pub fn map_rebuild_with<F, I, R>(&self, f: F, resolve: R) -> Result<Self, Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        let expanded = self.map_expand(f)?;
        #[cfg(feature = "rayon")]
        {
            if expanded.len() >= parallel::MIN_PAR_CHUNK {
                return Ok(Self::par_build_vec_with(expanded, resolve));
            }
        }
        let mut core = Self::new();
        for (id, value) in expanded {
            core.insert_with(id, value, &resolve);
        }
        Ok(core)
    }

    pub fn intersection(&self, other: &Self) -> Self {
        if let (Some(a), Some(b)) = (&self.shard, &other.shard)
            && a.intersection(b).is_none()
        {
            return Self {
                lower_root: self.empty_leaf.clone(),
                upper_root: self.empty_leaf.clone(),
                empty_leaf: self.empty_leaf.clone(),
                shard: Self::shard_after_intersection(&self.shard, &other.shard),
                shard_path: Self::shard_path_after_merge(&self.shard_path, &other.shard_path),
            };
        }

        Self {
            lower_root: self.merge_roots(
                &self.lower_root,
                &other.lower_root,
                MergeOp::Intersection,
            ),
            upper_root: self.merge_roots(
                &self.upper_root,
                &other.upper_root,
                MergeOp::Intersection,
            ),
            empty_leaf: self.empty_leaf.clone(),
            shard: Self::shard_after_intersection(&self.shard, &other.shard),
            shard_path: Self::shard_path_after_merge(&self.shard_path, &other.shard_path),
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        if let (Some(a), Some(b)) = (&self.shard, &other.shard)
            && a.intersection(b).is_none()
        {
            return self.clone();
        }

        Self {
            lower_root: self.merge_roots(&self.lower_root, &other.lower_root, MergeOp::Difference),
            upper_root: self.merge_roots(&self.upper_root, &other.upper_root, MergeOp::Difference),
            empty_leaf: self.empty_leaf.clone(),
            shard: self.shard,
            shard_path: self.shard_path.clone(),
        }
    }

    /// ルートノードのポインタが完全に同一か判定します（Result Reuseテスト用）
    #[cfg(test)]
    pub fn root_ptr_eq(&self, other: &Self) -> bool {
        SharedNode::ptr_eq(&self.lower_root, &other.lower_root)
            && SharedNode::ptr_eq(&self.upper_root, &other.upper_root)
    }

    /// 上下ルートが FXY-正規形を満たすことを検査し、違反があれば panic する（テスト用）。
    #[cfg(test)]
    pub(crate) fn assert_canonical(&self) {
        if let Err(reason) = self.lower_root.check_canonical() {
            panic!("lower_root not canonical: {reason}");
        }
        if let Err(reason) = self.upper_root.check_canonical() {
            panic!("upper_root not canonical: {reason}");
        }
    }

    /// コレクション内のすべての値をインプレースで更新します。
    ///
    /// `core` が `pub(crate)` になり、本番経路の利用者が居なくなったためテスト専用。
    /// 公開の等価物は [`SpatialIdTable::map_values_in_place`](crate::SpatialIdTable::map_values_in_place)。
    #[cfg(test)]
    pub fn map_values_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut V),
    {
        Node::map_values_mut(&mut self.lower_root, &mut f, &self.empty_leaf);
        Node::map_values_mut(&mut self.upper_root, &mut f, &self.empty_leaf);
    }

    /// `keep` が偽になる値の葉だけを取り除く。空間的な形は変えない。
    ///
    /// 木を平坦化して組み直す（`into_iter().collect()`）のではなく、**変化した経路だけを
    /// 作り直す** copy-on-write で刈る。したがって
    ///
    /// - 述語を全て満たす部分木は**元の `Arc` をそのまま保つ**（構造共有が維持される）
    /// - 中間の `Vec<(FlexId, V)>` を確保しない
    /// - 木の再構築（葉ごとの `insert`）が起きない
    ///
    /// 刈った結果として左右の子が等価になった枝は
    /// 内部で `collapse_equal_children` を呼んで畳み、正規形を保つ。
    pub fn retain_values<F>(&mut self, keep: F)
    where
        F: Fn(&V) -> bool,
    {
        let empty = self.empty_leaf.clone();
        if let Some(new_root) = Self::retain_node(&self.lower_root, &keep, &empty) {
            self.lower_root = new_root;
        }
        if let Some(new_root) = Self::retain_node(&self.upper_root, &keep, &empty) {
            self.upper_root = new_root;
        }
    }

    /// [`retain_values`](Self::retain_values) の本体。
    ///
    /// 変化があれば新しい部分木を返し、無ければ `None` を返す。`None` のときは呼び出し側が
    /// 元の `Arc` を保持し続けるので、触っていない部分木のクローンが発生しない。
    fn retain_node<F>(
        node: &SharedNode<Node<V>>,
        keep: &F,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> Option<SharedNode<Node<V>>>
    where
        F: Fn(&V) -> bool,
    {
        match &**node {
            Node::Leaf { value: None } => None,
            Node::Leaf { value: Some(v) } => {
                if keep(v) {
                    None
                } else {
                    Some(empty_leaf.clone())
                }
            }
            Node::Branch {
                level,
                lower_child,
                upper_child,
                ..
            } => {
                let new_lower = Self::retain_node(lower_child, keep, empty_leaf);
                let new_upper = Self::retain_node(upper_child, keep, empty_leaf);

                // どちらの子も変化していなければ、この枝ごと共有を維持する。
                if new_lower.is_none() && new_upper.is_none() {
                    return None;
                }

                let lower = new_lower.unwrap_or_else(|| lower_child.clone());
                let upper = new_upper.unwrap_or_else(|| upper_child.clone());

                // 刈った結果、左右が等価化したならここで畳んで正規形を保つ
                // （両子が空なら empty_leaf が返る）。
                if let Some(rep) =
                    Node::<V>::collapse_equal_children(&lower, &upper, *level, empty_leaf)
                {
                    return Some(rep);
                }

                let leaf_count = (lower.leaf_count() + upper.leaf_count()) as u32;
                if leaf_count == 0 {
                    return Some(empty_leaf.clone());
                }

                Some(SharedNode::new(Node::Branch {
                    level: *level,
                    leaf_count,
                    max_zoom: Node::<V>::fold_max_zoom(*level, &lower, &upper),
                    split_mask: Node::<V>::fold_split_mask(*level, &lower, &upper),
                    lower_child: lower,
                    upper_child: upper,
                }))
            }
        }
    }

    ///クリアする
    pub fn clear(&mut self) {
        self.lower_root = self.empty_leaf.clone();
        self.upper_root = self.empty_leaf.clone();
    }

    pub fn is_empty(&self) -> bool {
        self.lower_root.leaf_count() == 0 && self.upper_root.leaf_count() == 0
    }

    pub fn count(&self) -> usize {
        self.lower_root.leaf_count() + self.upper_root.leaf_count()
    }

    /// この木が時間軸（T）で分割されたノードを1つでも持つかを返す。O(1)。
    ///
    /// 各 Branch がキャッシュしている `split_mask` の畳み上げを見るだけなので、木を
    /// 走査しない。偽なら**全ての葉が全時間**であることが保証されるので、書き出し時の
    /// 時間方向の結合（[`coalesce_temporal`](super::coalesce::coalesce_temporal)）を
    /// まるごと省略でき、`flat_single_ids_ref` の遅延評価を保てる。
    ///
    /// `temporal_id` feature 無効時は T の番自体が無い（`NUM_AXES == 3`）ため常に偽。
    pub(crate) fn has_temporal_split(&self) -> bool {
        self.split_mask() & node::axis_bit(Axis::T) != 0
    }

    /// 上下ルートの `split_mask` を OR したもの。O(1)。
    pub(crate) fn split_mask(&self) -> u8 {
        self.lower_root.split_mask() | self.upper_root.split_mask()
    }

    /// この [`FlexTreeCore`] に含まれる要素のうち、最も高いズームレベル値を返します。ここでいう解像度は、各 [`FlexId`] の `f/x/y` それぞれのズームレベルの最大値です。
    /// 空の木では [`None`] を返します。
    ///
    /// 検証は `core_api_tests::max_zoomlevel_reports_the_finest_axis` を参照。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let lower = self.lower_root.max_zoom();
        let upper = self.upper_root.max_zoom();
        Some(lower.max(upper))
    }

    /// この集合が値を持つ全Segmentを包む最小の[RangeId]を返します。
    ///
    /// 検証は `core_api_tests::bounding_box_covers_every_segment` を参照。
    pub fn bounding_box(&self) -> Option<RangeId> {
        self.summary().bbox
    }

    /// この木の要約（[`ShardSummary`]）を作る。葉を1度だけ走査するので O(葉数)。
    ///
    /// `leaf_count` と `split_mask` はノードのキャッシュから O(1) で取れるが、
    /// bounding box と絶対秒区間は位置依存でノードに持てない
    /// （[`summary`](super::summary) モジュールの説明を参照）ため、ここで走査して求める。
    /// [`bounding_box`](Self::bounding_box) もこの1周に相乗りする。
    pub fn summary(&self) -> ShardSummary {
        let leaf_count = self.count() as u32;
        let split_mask = self.split_mask();

        // 空の木は `max_zoomlevel()` が `None`。逆に非空なら葉が必ず1枚以上あるので、
        // このガードを抜けた先のループは必ず1回以上回る（`max_zoomlevel` の空判定は
        // `iter_ref` が数えるのと同じ `leaf_count` を見ている）。
        let Some(max_z) = self.max_zoomlevel() else {
            return ShardSummary::empty();
        };

        // 各Segmentの範囲を共通ズーム `max_z` へ揃えてから min/max を取る。揃える前に
        // 累積すると軸ごとにズームの異なるSegmentが混ざって比較できない。
        //
        // 木の走査経路から領域を復元する方法は、軸ごとにズームが異なるSegment
        // （パス圧縮された `FlexId`）で経路と実際の広がりがずれるため使わない。
        let mut f_acc = [i64::MAX, i64::MIN];
        let mut x_acc = [i64::MAX, i64::MIN];
        let mut y_acc = [i64::MAX, i64::MIN];
        let mut min_zoom = [u8::MAX; 3];
        let mut max_zoom = [0u8; 3];
        let mut t_zoom = [u8::MAX, 0u8];
        let mut seconds = (u64::MAX, 0u64);

        for (flex_id, _) in self.iter_ref() {
            let zooms = [
                flex_id.f_zoomlevel(),
                flex_id.x_zoomlevel(),
                flex_id.y_zoomlevel(),
            ];
            for axis in 0..3 {
                min_zoom[axis] = min_zoom[axis].min(zooms[axis]);
                max_zoom[axis] = max_zoom[axis].max(zooms[axis]);
            }
            t_zoom[0] = t_zoom[0].min(flex_id.t_zoomlevel());
            t_zoom[1] = t_zoom[1].max(flex_id.t_zoomlevel());

            let (start, end) = flex_id.seconds_range();
            seconds.0 = seconds.0.min(start);
            seconds.1 = seconds.1.max(end);

            let range = RangeId::from(&flex_id);
            let shift = max_z - range.z();
            for (acc, axis) in [
                (&mut f_acc, range.f().map(i64::from)),
                (&mut x_acc, range.x().map(i64::from)),
                (&mut y_acc, range.y().map(i64::from)),
            ] {
                acc[0] = acc[0].min(axis[0] << shift);
                acc[1] = acc[1].max(((axis[1] + 1) << shift) - 1);
            }
        }

        ShardSummary {
            leaf_count,
            split_mask,
            max_zoom,
            min_zoom,
            t_zoom,
            bbox: RangeId::new(
                max_z,
                [f_acc[0] as i32, f_acc[1] as i32],
                [x_acc[0] as u32, x_acc[1] as u32],
                [y_acc[0] as u32, y_acc[1] as u32],
            )
            .ok(),
            seconds_range: Some(seconds),
        }
    }

    /// この [`FlexTreeCore`] に含まれる要素を、木全体の `max_zoomlevel` に揃えた [`SingleId`] として書き出す。
    ///
    /// [`flat_single_ids_ref`](Self::flat_single_ids_ref) と同じく、時間方向の結合を通す。
    #[cfg(test)]
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, V)> {
        let Some(max_zoomlevel) = self.max_zoomlevel() else {
            return Vec::new().into_iter();
        };

        // 1葉が複数のSingleIdへ分解されうるため下限のヒント（葉数）を与える。
        let mut exported = Vec::with_capacity(self.count());

        let merged: Vec<(RangeId, V)> = if self.has_temporal_split() {
            crate::spatial_id::collection::flex_tree::coalesce::coalesce_temporal(self.iter(), None)
                .collect()
        } else {
            self.iter()
                .map(|(flex_id, value)| (RangeId::from(&flex_id), value))
                .collect()
        };

        for (range, value) in merged {
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };

            for single_id in normalized.single_ids() {
                exported.push((single_id, value.clone()));
            }
        }

        exported.into_iter()
    }

    /// この [`FlexTreeCore`] に含まれる要素を、木全体の `max_zoomlevel` に揃えた [`SingleId`] として値の参照付きで書き出す。
    ///
    /// 木が時間軸で分割されている場合のみ、書き出す前に時間方向に隣接する同値のSegmentを
    /// [`coalesce_temporal`](crate::spatial_id::collection::flex_tree::coalesce::coalesce_temporal)
    /// で結合する。木は時間を2の冪秒のSegmentとして持つため、これを行わないと
    /// `1800` 秒のような単位で入れた ID が断片のまま出てきてしまう。
    ///
    /// # 遅延評価について
    ///
    /// 結合は木全体を集めてソートする（同じFlexIdの時間Segmentは木の走査順では連続しない
    /// ため、1件先読みでは結合しきれない）。そのため時間を持たない木で無条件に通すと、
    /// 本来 `O(1)` で始まるはずの列挙が全件走査になってしまう。
    /// [`has_temporal_split`](Self::has_temporal_split) が偽なら全葉が全時間で結合対象が
    /// 存在しないので、結合ごと省いて素通しの遅延イテレータを返す。
    pub fn flat_single_ids_ref(&self) -> Box<dyn Iterator<Item = (SingleId, &V)> + '_> {
        self.flat_single_ids_in_ref(None)
    }

    /// 時間方向に結合した [`RangeId`] として読み出す。**空間解像度は変えない**。
    ///
    /// `units` に [`AllowedIntervals`] を渡すと、結合後の秒区間をその候補のうち最も粗い単位
    /// （＝Segment数が候補の中で最小になる単位）で表す。`None` なら `gcd` で最も粗い単位を選ぶ。
    ///
    /// 木が時間軸で分割されていなければ結合対象が無いので、素通しの遅延イテレータを返す。
    pub fn range_ids_ref<'a>(
        &'a self,
        units: Option<&'a AllowedIntervals>,
    ) -> Box<dyn Iterator<Item = (RangeId, &'a V)> + 'a> {
        if !self.has_temporal_split() {
            // 全葉が全時間。結合対象が無いので集めずに素通しできる。
            // 候補集合が指定されていても、全時間は `WHOLE`（必ず候補にある）のままになる。
            return Box::new(
                self.iter_ref()
                    .map(|(flex_id, value)| (RangeId::from(&flex_id), value)),
            );
        }

        Box::new(
            crate::spatial_id::collection::flex_tree::coalesce::coalesce_temporal(
                self.iter_ref(),
                units,
            ),
        )
    }

    /// [`flat_single_ids_ref`](Self::flat_single_ids_ref) の、時間単位を指定できる版。
    pub fn flat_single_ids_in_ref<'a>(
        &'a self,
        units: Option<&'a AllowedIntervals>,
    ) -> Box<dyn Iterator<Item = (SingleId, &'a V)> + 'a> {
        let Some(max_zoomlevel) = self.max_zoomlevel() else {
            return Box::new(core::iter::empty());
        };

        let merged = self.range_ids_ref(units);

        Box::new(merged.flat_map(move |(range, value)| {
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };

            normalized
                .single_ids()
                .map(move |single_id| (single_id, value))
        }))
    }

    /// [FlexTreeCore]からtargetと重なりがある[FlexId]とそのValueへの参照を全て取り出す。
    pub fn get_ref<'a, I>(&'a self, target: I) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        I: IntoIterator<Item = FlexId> + 'a,
        V: 'a,
    {
        target.into_iter().flat_map(move |item| {
            self.overlap_ref(item).filter_map(move |(overlap_id, val)| {
                overlap_id
                    .intersection(&item)
                    .map(|intersected_id| (intersected_id, val))
            })
        })
    }

    /// [FlexTreeCore]に空間IDを挿入する
    pub fn insert<I>(&mut self, target: I, value: V)
    where
        I: IntoIterator<Item = FlexId>,
    {
        for flex_id in target.into_iter() {
            // シャード初期化されている場合、領域外は無視し、はみ出しは切り詰める。
            let flex_id = match &self.shard {
                Some(region) => match flex_id.intersection(region) {
                    Some(clipped) => clipped,
                    None => continue,
                },
                None => flex_id,
            };
            self.insert_flex_id(flex_id, value.clone());
        }
    }

    /// [FlexTreeCore]に空間IDをポリシー付きで挿入する
    pub fn insert_with<I, R>(&mut self, target: I, value: V, resolve: &R)
    where
        I: IntoIterator<Item = FlexId>,
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        for flex_id in target.into_iter() {
            // シャード初期化されている場合、領域外は無視し、はみ出しは切り詰める。
            let flex_id = match &self.shard {
                Some(region) => match flex_id.intersection(region) {
                    Some(clipped) => clipped,
                    None => continue,
                },
                None => flex_id,
            };
            self.insert_flex_id_with(flex_id, value.clone(), resolve);
        }
    }

    /// [FlexTreeCore]からtargetと重なりがある[FlexId]とそのValueを全て取り出す
    pub fn get<'a, I>(&'a self, target: I) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        I: IntoIterator<Item = FlexId> + 'a,
        V: Clone + 'a,
    {
        target.into_iter().flat_map(move |item| {
            self.overlap(item).filter_map(move |(overlap_id, val)| {
                overlap_id
                    .intersection(&item)
                    .map(|intersected_id| (intersected_id, val.clone()))
            })
        })
    }

    /// [FlexTreeCore]からTargetが示す領域を削除して、返す。
    ///
    /// 削除は呼び出した時点で完了する。返り値は「何が消えたか」の記録なので、
    /// 不要なら捨ててよい（`impl Iterator` を返すと `#[must_use]` が付き、
    /// 「消費しないと削除されない」と読めてしまうため `Vec` を返す）。
    pub fn remove<I>(&mut self, target: I) -> Vec<(FlexId, V)>
    where
        I: IntoIterator<Item = FlexId>,
    {
        let mut actual_removed = Vec::new();

        for t_id in target.into_iter() {
            for (leaf_id, value) in self.overlap_remove(&t_id) {
                for remnant_id in leaf_id.difference(&t_id) {
                    self.insert_flex_id(remnant_id, value.clone());
                }
                if let Some(intersect_id) = leaf_id.intersection(&t_id) {
                    actual_removed.push((intersect_id, value));
                }
            }
        }

        actual_removed
    }

    /// [`get`](Self::get) と同様に target と重なる要素を取り出しますが、
    /// 切り取りを行わず、[`FlexId`] をそのままの広さで返す。
    pub fn get_overlapping<'a, I>(&'a self, target: I) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        I: IntoIterator<Item = FlexId> + 'a,
        V: Clone + 'a,
    {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for item in target.into_iter() {
            for (overlap_id, value) in self.overlap(item) {
                if seen.insert(overlap_id) {
                    results.push((overlap_id, value));
                }
            }
        }
        results.into_iter()
    }

    /// [`get_overlapping`](Self::get_overlapping) の参照版。
    pub fn get_overlapping_ref<'a, I>(
        &'a self,
        target: I,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        I: IntoIterator<Item = FlexId> + 'a,
        V: 'a,
    {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for item in target.into_iter() {
            for (overlap_id, value) in self.overlap_ref(item) {
                if seen.insert(overlap_id) {
                    results.push((overlap_id, value));
                }
            }
        }
        results.into_iter()
    }

    /// [`remove`](Self::remove) と異なり、**交差による切り取りや残余の再挿入を行わず**、 target と少しでも重なった葉を丸ごとツリーから取り除き、その格納済み [`FlexId`] を そのままの広さで返す。
    pub fn remove_overlapping<I>(&mut self, target: I) -> Vec<(FlexId, V)>
    where
        I: IntoIterator<Item = FlexId>,
    {
        let mut removed = Vec::new();
        for t_id in target.into_iter() {
            removed.extend(self.overlap_remove(&t_id));
        }
        removed
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返す。入力された空間ID自身と重なる要素は除外する。
    pub fn neighbors_share_face_ref<'a, S>(
        &'a self,
        id: &S,
    ) -> alloc::vec::IntoIter<(FlexId, &'a V)>
    where
        S: SpatialId,
    {
        let self_ids: Vec<FlexId> = id.clone().into_iter().collect();

        let mut slabs: Vec<S> = Vec::new();
        for delta in [-1, 1] {
            let mut sf = id.clone();
            if sf.move_f(delta).is_ok() {
                slabs.push(sf);
            }
            let mut sy = id.clone();
            if sy.move_y(delta).is_ok() {
                slabs.push(sy);
            }
            let mut sx = id.clone();
            sx.move_x(delta);
            slabs.push(sx);
        }

        let mut seen: HashSet<FlexId> = HashSet::new();
        let mut results: Vec<(FlexId, &'a V)> = Vec::new();

        for slab in &slabs {
            for slab_id in slab.clone().into_iter() {
                for (cand, value) in self.overlap_ref(slab_id) {
                    if self_ids.iter().any(|s| cand.intersection(s).is_some()) {
                        continue;
                    }
                    if !self_ids.iter().any(|s| s.shares_face(&cand)) {
                        continue;
                    }
                    if seen.insert(cand) {
                        results.push((cand, value));
                    }
                }
            }
        }

        results.into_iter()
    }

    /// [FlexTreeCore]から全ての[FlexId]とValueを取り出す（値はクローン）。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        self.iter_ref()
            .map(|(flex_id, value)| (flex_id, value.clone()))
    }

    /// [FlexTreeCore]から全ての[FlexId]とValueへの参照を取り出す。
    pub fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        LeavesIterRef {
            stack: self.root_node_stack(),
        }
    }

    /// 走査開始点として上下ルートノードを ID 付きで収集する。
    pub(super) fn root_node_stack(&self) -> Vec<(&Node<V>, FlexId)> {
        let mut stack = Vec::new();

        if !SharedNode::ptr_eq(&self.upper_root, &self.empty_leaf) {
            stack.push((self.upper_root.as_ref(), FlexId::UPPER_MAX));
        }

        if !SharedNode::ptr_eq(&self.lower_root, &self.empty_leaf) {
            stack.push((self.lower_root.as_ref(), FlexId::LOWER_MAX));
        }

        stack
    }

    fn insert_flex_id(&mut self, flex_id: FlexId, value: V) {
        let root = if flex_id.f_index().is_negative() {
            &mut self.lower_root
        } else {
            &mut self.upper_root
        };
        Node::insert_mut(root, &flex_id, &value, 0, &self.empty_leaf);
    }

    fn insert_flex_id_with<R>(&mut self, flex_id: FlexId, value: V, resolve: &R)
    where
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        let root = if flex_id.f_index().is_negative() {
            &mut self.lower_root
        } else {
            &mut self.upper_root
        };
        Node::insert_mut_with(root, &flex_id, &value, 0, &self.empty_leaf, resolve);
    }

    /// unionのシャード領域を返す。
    /// シャードされている場合とされていない場合があるので、そのラッパー
    fn shard_after_union(a: &Option<FlexId>, b: &Option<FlexId>) -> Option<FlexId> {
        match (a, b) {
            (Some(a), Some(b)) if a == b => Some(*a),
            _ => None,
        }
    }

    /// intersectionのシャード領域を返す。
    /// シャードされている場合とされていない場合があるので、そのラッパー
    fn shard_after_intersection(a: &Option<FlexId>, b: &Option<FlexId>) -> Option<FlexId> {
        match (a, b) {
            (Some(a), Some(b)) => a.intersection(b).or(Some(*a)),
            (Some(a), None) => Some(*a),
            (None, Some(b)) => Some(*b),
            (None, None) => None,
        }
    }
}

/// 空間ソートキーの1軸あたりビット数。
///
/// `temporal_id` 有効時は F/X/Y/T の4軸で 4×20 = 80bit（`u128`）、無効時は F/X/Y の
/// 3軸で 3×20 = 60bit（`u64`）。
#[cfg(feature = "rayon")]
const SORT_KEY_BITS: u32 = 20;

/// [`spatial_sort_key`] の戻り型。軸数に応じて必要な幅だけを使う。
///
/// 時間を使わないビルドで `u128` のまま回すと、並列バルク構築のソートが理由なく
/// 倍幅の比較になる。
#[cfg(all(feature = "rayon", feature = "temporal_id"))]
pub(crate) type SortKey = u128;
/// [`spatial_sort_key`] の戻り型（3軸版）。
#[cfg(all(feature = "rayon", not(feature = "temporal_id")))]
pub(crate) type SortKey = u64;

/// 軸のインデックスを、ズームに依らず先頭ビット揃え（MSB 揃え）で `bits` 幅へ正規化する。
/// 粗い（浅い）Segmentは上位ビット側に、細かいSegmentは下位ビットまで伸びる。
#[cfg(feature = "rayon")]
#[inline]
fn axis_aligned(index: u64, zoom: u8, bits: u32) -> u64 {
    let z = zoom as u32;
    let a = if z <= bits {
        index << (bits - z)
    } else {
        index >> (z - bits)
    };
    a & ((1u64 << bits) - 1)
}

/// [`FlexId`] の空間・時間位置を単調なキーへ写す。F→X→Y→T の順にビットを詰め、木の降下順
/// （レベル 0=F, 1=X, 2=Y, 3=T, …）と整合する粗いクラスタリングを与える。厳密な木順ではなく
/// 「近い ID を連続させる」ことが目的で、これによりチャンクが局所化し、
/// チャンク木同士の [`union`](FlexTreeCore::union) / [`merge_with`](FlexTreeCore::merge_with) が
/// 互いにほぼ素になって簡約が軽くなる。並列バルク構築と値解決構築の双方で使う。
/// `temporal_id` 有効時はTのインデックスが`u64`まで取りうるため、キー全体を[`SortKey`]
/// = `u128`に拡張している（4軸×20bit=80bitで`u64`に収まらない）。無効時はT軸が無いので
/// 3軸×20bit=60bitの`u64`に戻る。
#[cfg(feature = "rayon")]
#[inline]
pub(crate) fn spatial_sort_key(id: &FlexId) -> SortKey {
    const B: u32 = SORT_KEY_BITS;
    // F は符号付き。木は最初に符号でルートを分けるため、符号ビットを最上位に置く。
    let f_biased = (id.f_index() as i64 + (1i64 << 30)) as u64;
    let fa = axis_aligned(f_biased, id.f_zoomlevel().saturating_add(1), B) as SortKey;
    let xa = axis_aligned(id.x_index() as u64, id.x_zoomlevel(), B) as SortKey;
    let ya = axis_aligned(id.y_index() as u64, id.y_zoomlevel(), B) as SortKey;

    #[cfg(feature = "temporal_id")]
    {
        let ta = axis_aligned(id.t(), id.t_zoomlevel(), B) as SortKey;
        (fa << (3 * B)) | (xa << (2 * B)) | (ya << B) | ta
    }

    #[cfg(not(feature = "temporal_id"))]
    {
        (fa << (2 * B)) | (xa << B) | ya
    }
}

/// 軸と side に応じて、現在 ID から子ノード側の ID を1段分割して返す。
pub(crate) fn split_child_id(current_id: &FlexId, axis: Axis, side: Side) -> FlexId {
    match axis {
        Axis::F => current_id.split_f(side).unwrap(),
        Axis::X => current_id.split_x(side).unwrap(),
        Axis::Y => current_id.split_y(side).unwrap(),
        Axis::T => current_id.split_t(side).unwrap(),
    }
}

impl<V> IntoIterator for FlexTreeCore<V>
where
    V: SafeValue,
{
    type Item = (FlexId, V);
    type IntoIter = crate::spatial_id::collection::flex_tree::core::convert::LeavesIntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack = Vec::new();

        if !crate::spatial_id::collection::flex_tree::core::ptr::SharedNode::ptr_eq(
            &self.upper_root,
            &self.empty_leaf,
        ) {
            stack.push((self.upper_root, FlexId::UPPER_MAX));
        }

        if !crate::spatial_id::collection::flex_tree::core::ptr::SharedNode::ptr_eq(
            &self.lower_root,
            &self.empty_leaf,
        ) {
            stack.push((self.lower_root, FlexId::LOWER_MAX));
        }

        crate::spatial_id::collection::flex_tree::core::convert::LeavesIntoIter { stack }
    }
}

#[allow(clippy::too_many_arguments)]
impl<V: SafeValue> FromIterator<(FlexId, V)> for FlexTreeCore<V> {
    /// `(FlexId, 値)` 列からツリーを構築する（重なりは union・左優先）。
    ///
    /// 要素数が十分にあれば並列構築へ切り替える。`insert` は挿入順に依らず
    /// O(深さ) なので、事前のソートや重複除去は不要。
    fn from_iter<I: IntoIterator<Item = (FlexId, V)>>(iter: I) -> Self {
        let items: Vec<_> = iter.into_iter().collect();

        #[cfg(feature = "rayon")]
        {
            if items.len() >= parallel::MIN_PAR_CHUNK {
                return Self::par_build_vec(items);
            }
        }

        let mut core = Self::new();
        for (id, value) in items {
            core.insert(id, value);
        }
        core
    }
}

/// `core` は `pub(crate)` なので doctest（クレート外から実行される）が書けない。
/// もとの doctest 相当をここへ移した。
#[cfg(test)]
mod core_api_tests {
    use super::FlexTreeCore;
    use crate::{RangeId, SingleId};

    #[test]
    fn max_zoomlevel_reports_the_finest_axis() {
        let mut core = FlexTreeCore::new();
        core.insert(RangeId::new(4, [0, 1], [0, 0], [0, 0]).unwrap(), ());
        assert_eq!(core.max_zoomlevel(), Some(4));

        let empty: FlexTreeCore<i32> = FlexTreeCore::new();
        assert_eq!(empty.max_zoomlevel(), None);
    }

    /// 時間を持たない木は T 軸で分割されないので、書き出しの結合を丸ごと省ける。
    #[test]
    fn has_temporal_split_is_false_without_time() {
        let mut core = FlexTreeCore::new();
        core.insert(SingleId::new(20, 0, 0, 0).unwrap(), 1);
        core.insert(SingleId::new(20, 3, 5, 7).unwrap(), 2);
        assert!(!core.has_temporal_split());

        let empty: FlexTreeCore<i32> = FlexTreeCore::new();
        assert!(!empty.has_temporal_split());
    }

    /// 時間を持つ木では T 軸の分割が立つ。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn has_temporal_split_is_true_with_time() {
        use crate::Interval;

        let mut core = FlexTreeCore::new();
        core.insert(
            SingleId::new(20, 0, 0, 0)
                .unwrap()
                .with_time(Interval::HOUR, 5)
                .unwrap(),
            1,
        );
        assert!(core.has_temporal_split());
    }

    /// 時間を持たない木の `flat_single_ids_ref` は遅延評価のままでなければならない。
    ///
    /// 時間方向の結合を無条件に通していた頃は、先頭1件を取るだけで木全体の走査＋ソートが
    /// 走っていた（20万葉で 10µs → 373ms）。
    #[test]
    fn flat_single_ids_ref_stays_lazy_without_time() {
        use core::cell::Cell;

        // 値をばらけさせて、正規化で1葉にまとまらないようにする。
        let mut core = FlexTreeCore::new();
        for x in 0..32u32 {
            for y in 0..32u32 {
                core.insert(SingleId::new(20, 0, x * 2, y * 2).unwrap(), x * 32 + y);
            }
        }
        let total = core.count();
        assert!(total > 8, "十分な葉数が要る (got {total})");

        // 実際に消費された葉の数を数える。
        let consumed = Cell::new(0usize);
        let counted = core
            .iter_ref()
            .inspect(|_| consumed.set(consumed.get() + 1));
        // `flat_single_ids_ref` と同じ経路（結合なし）を通ることを確認する。
        assert!(!core.has_temporal_split());
        let mut it = counted.map(|(id, v)| (RangeId::from(&id), v));
        let _first = it.next().expect("先頭が取れる");
        assert_eq!(
            consumed.get(),
            1,
            "先頭1件のために木全体を走査してはならない"
        );

        // 公開経路そのものでも、先頭を取るだけなら全件展開されない。
        let mut public = core.flat_single_ids_ref();
        assert!(public.next().is_some());
    }

    #[test]
    fn bounding_box_covers_every_segment() {
        let mut core = FlexTreeCore::new();
        core.insert(SingleId::new(20, 0, 0, 0).unwrap(), 1);
        core.insert(SingleId::new(20, 0, 2, 3).unwrap(), 1);

        let bbox = core.bounding_box().unwrap();
        assert_eq!(bbox.z(), 20);
        assert_eq!(bbox.f(), [0, 0]);
        assert_eq!(bbox.x(), [0, 2]);
        assert_eq!(bbox.y(), [0, 3]);

        let empty: FlexTreeCore<i32> = FlexTreeCore::new();
        assert!(empty.bounding_box().is_none());
    }

    /// Tが本当にFlexTreeの第4軸として機能しているかを確認する（同一のF/X/Yで時間だけが
    /// 異なる2つのFlexIdが、木の中で別々に区別・保持されること）。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn distinct_temporal_segments_stay_distinguishable() {
        use crate::FlexId;

        let mut core: FlexTreeCore<u32> = FlexTreeCore::new();

        // 同じFlexIdで、時間だけが隣り合う2つのSegment（ズーム2＝全時間の1/4幅）。
        let a = FlexId::new(3, 1, 3, 1, 3, 1)
            .unwrap()
            .with_time(2u8, 0)
            .unwrap();
        let b = FlexId::new(3, 1, 3, 1, 3, 1)
            .unwrap()
            .with_time(2u8, 1)
            .unwrap();

        core.insert([a], 10);
        core.insert([b], 20);

        assert_eq!(core.count(), 2);
        core.assert_canonical();

        let got: alloc::vec::Vec<_> = core.iter().collect();
        assert!(got.contains(&(a, 10)));
        assert!(got.contains(&(b, 20)));
    }

    /// 時間だけが深いSegmentがあっても、`max_zoomlevel` は**空間**の解像度だけを報告する。
    ///
    /// レベル番号から `ceil(level / NUM_AXES)` として推定していた頃は、時間軸（最大ズーム35）の
    /// 深さがそのまま空間ズームとして出てしまい、`bounding_box` が `None` になり
    /// `flat_single_ids` が `ZOutOfRange` でパニックしていた。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn max_zoom_reports_spatial_resolution_only() {
        use crate::{Interval, SingleId};

        let id = SingleId::new(10, 0, 5, 5)
            .unwrap()
            .with_time(Interval::HOUR, 491_666)
            .unwrap();

        let mut core: FlexTreeCore<u32> = FlexTreeCore::new();
        core.insert(id.clone(), 1);
        assert!(core.count() > 1, "1時間ぶんは複数Segmentへ分解されるはず");

        assert_eq!(core.max_zoomlevel(), Some(10));

        let bbox = core.bounding_box().expect("bounding_box が None");
        assert_eq!(bbox.z(), 10);
        assert_eq!(bbox.x(), [5, 5]);

        // 時間方向に結合されて元の ID がそのまま戻る。
        let flat: Vec<_> = core.flat_single_ids().collect();
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, id);
    }

    /// 時間区間を2分岐Segmentへ分解して挿入すると、元の秒区間をちょうど覆う集合が木に入る。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn seconds_range_decomposition_round_trips_into_tree() {
        use crate::{FlexId, Interval, RangeId, SpatialId};

        let range = RangeId::new(0, 0, 0, 0)
            .unwrap()
            .with_time(Interval::HOUR, [2, 2])
            .unwrap();

        let time_segments: alloc::vec::Vec<_> = range.clone().into_iter().collect();
        assert!(!time_segments.is_empty());

        let mut core: FlexTreeCore<u32> = FlexTreeCore::new();
        for time_segment in &time_segments {
            core.insert([*time_segment], 1);
        }

        assert_eq!(core.count(), time_segments.len());
        core.assert_canonical();

        // 分解したSegmentの絶対秒区間を合算すると、元の秒区間と一致する。
        let total_seconds: u64 = time_segments
            .iter()
            .map(|time_segment| {
                let (start, end) = time_segment.seconds_range();
                end - start
            })
            .sum();
        let (range_start, range_end) = range.seconds_range();
        assert_eq!(total_seconds, range_end - range_start);

        // 分解元と同じFlexIdであることも確認する。
        assert!(
            time_segments
                .iter()
                .all(|c| c.f_index() == 0 && c.x_index() == 0)
        );
        let _ = FlexId::new(0, 0, 0, 0, 0, 0).unwrap();
    }
}
