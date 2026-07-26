use super::ptr::{SafeValue, SharedNode};
use crate::{Dimension, FlexId, Side};

/// 葉ノードの仮想ツリーレベル（ズーム30相当）。木を降りる操作で「葉に達した」ことを表す
/// 番兵として使う。実在の Branch はこれ未満のレベルしか持たない。
pub(crate) const LEAF_LEVEL: u8 = 93;

/// Branch の両子（下・上）への参照ペア。
type ChildRefs<'a, V> = (&'a SharedNode<Node<V>>, &'a SharedNode<Node<V>>);

#[derive(Debug, PartialEq, Clone, Eq)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "persist", rkyv(archive_bounds(V: 'static)))]
#[cfg_attr(
    feature = "persist",
    rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::ser::Sharing,
        <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(deserialize_bounds(
        __D: rkyv::de::Pooling,
        <__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext + rkyv::validation::SharedContext,
        <__C as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    )))
)]
pub enum Node<V>
where
    V: SafeValue,
{
    Branch {
        level: u8,
        leaf_count: u32,
        max_zoom: u8,
        /// この部分木が分割している軸の集合（F=0b001 / X=0b010 / Y=0b100 の OR）。
        /// `leaf_count` / `max_zoom` と同じく子から畳み上げてキャッシュする。
        /// `collapse_equal_children` の畳み込みガード
        /// （「この軸をそれ以深で分割していないか」）を O(1) で判定するために持つ。
        split_mask: u8,
        #[cfg_attr(feature = "persist", rkyv(omit_bounds))]
        lower_child: SharedNode<Node<V>>,
        #[cfg_attr(feature = "persist", rkyv(omit_bounds))]
        upper_child: SharedNode<Node<V>>,
    },
    Leaf {
        value: Option<V>,
    },
}

/// ある Branch を降りるとき、target と交差しうる子。
///
/// [`Node::overlapping_children`] が返す。反対側の子は交差しえないので、
/// [`Only`](Self::Only) なら部分木を丸ごと枝刈りできる。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum OverlappingChildren {
    /// target がこの軸を丸ごと覆うため、両方の子が交差しうる。
    Both,
    /// この側の子だけが交差する。
    Only(Side),
}

impl<V> Node<V>
where
    V: SafeValue,
{
    /// 各ノード以下の (値が Some の) Leaf の合計数を返す。O(1)で取得可能。
    pub fn leaf_count(&self) -> usize {
        match self {
            Node::Branch { leaf_count, .. } => *leaf_count as usize,
            Node::Leaf { value: Some(_) } => 1,
            Node::Leaf { value: None } => 0,
        }
    }

    /// このノードを `node_level`（自身の絶対ツリーレベル）に置いたときの、配下の値付き Leaf の
    /// FlexId ズームレベルの最大値を返す。Branch はキャッシュ済みの値を返すため O(1)。
    ///
    /// ツリーレベル `L` の Leaf が持つ FlexId のズームは `max(f, x, y) = ceil(L / 3)` に等しい
    /// （[`covers_all_axes`](Self::covers_all_axes) の式と整合）。値の無い Leaf は 0 を返す。
    pub(crate) fn max_zoom_at(&self, node_level: u8) -> u8 {
        match self {
            Node::Branch { max_zoom, .. } => *max_zoom,
            Node::Leaf { value: Some(_) } => node_level.div_ceil(3),
            Node::Leaf { value: None } => 0,
        }
    }

    /// レベル `level` の Branch を構築・更新する際の `max_zoom` を、両子の部分木から畳み上げる。
    /// 子はいずれもレベル `level + 1` に位置する。
    pub(crate) fn fold_max_zoom(level: u8, lower: &Node<V>, upper: &Node<V>) -> u8 {
        let child_level = level + 1;
        lower
            .max_zoom_at(child_level)
            .max(upper.max_zoom_at(child_level))
    }

    /// 軸に対応する `split_mask` の1ビット（F=0b001 / X=0b010 / Y=0b100）。
    pub(crate) fn axis_bit(axis: Dimension) -> u8 {
        match axis {
            Dimension::F => 0b001,
            Dimension::X => 0b010,
            Dimension::Y => 0b100,
        }
    }

    /// このノード配下が分割している軸の集合。Branch はキャッシュ済みで O(1)、Leaf は 0。
    pub(crate) fn split_mask(&self) -> u8 {
        match self {
            Node::Branch { split_mask, .. } => *split_mask,
            Node::Leaf { .. } => 0,
        }
    }

    /// このノードのツリーレベル。葉は仮想的に [`LEAF_LEVEL`] を返す（降下の番兵）。
    pub(crate) fn node_level(&self) -> u8 {
        match self {
            Node::Branch { level, .. } => *level,
            Node::Leaf { .. } => LEAF_LEVEL,
        }
    }

    /// Branch なら両子（下・上）への参照を返す。葉なら `None`。
    pub(crate) fn children(&self) -> Option<ChildRefs<'_, V>> {
        match self {
            Node::Branch {
                lower_child,
                upper_child,
                ..
            } => Some((lower_child, upper_child)),
            Node::Leaf { .. } => None,
        }
    }

    /// レベル `level` の Branch を構築・更新する際の `split_mask` を、両子から畳み上げる。
    /// 自身が分割する軸 `axis(level)` に、両子の分割軸を OR する。
    pub(crate) fn fold_split_mask(level: u8, lower: &Node<V>, upper: &Node<V>) -> u8 {
        Self::axis_bit(Self::axis(level)) | lower.split_mask() | upper.split_mask()
    }

    /// Branch を構築する唯一の入口（smart constructor）。
    ///
    /// 常に正規形（FXY-正規形）を構成的に保証する。
    /// - N1: 両子が等価で、かつ畳む軸をそれ以深で分割していなければ、片方の子へ畳む。
    /// - N2: 両子が空（`Leaf{None}`）なら `empty_leaf` シングルトンへ畳む（上記に含まれる）。
    /// - キャッシュ（`leaf_count` / `max_zoom` / `split_mask`）を畳み上げて格納する。
    ///
    /// 木を構築・更新するすべての経路（挿入・集合演算・削除・値変換・シャード）は、
    /// 巻き戻しでこの関数を通すことで正規形を保つ。
    pub(crate) fn mk(
        level: u8,
        lower: SharedNode<Node<V>>,
        upper: SharedNode<Node<V>>,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> SharedNode<Node<V>> {
        if let Some(rep) = Self::collapse_equal_children(&lower, &upper, level, empty_leaf) {
            return rep;
        }
        SharedNode::new(Node::Branch {
            level,
            leaf_count: (lower.leaf_count() + upper.leaf_count()) as u32,
            max_zoom: Self::fold_max_zoom(level, &lower, &upper),
            split_mask: Self::fold_split_mask(level, &lower, &upper),
            lower_child: lower,
            upper_child: upper,
        })
    }

    /// level から対象とする軸(F, X, Y) を返す
    pub fn axis(level: u8) -> Dimension {
        match level % 3 {
            0 => Dimension::F,
            1 => Dimension::X,
            2 => Dimension::Y,
            _ => unreachable!(),
        }
    }

    /// level から各軸の深度を返す
    pub fn depth(level: u8) -> u8 {
        level / 3
    }

    /// FlexId の指定次元に対するズームレベルを返す
    fn target_zoom(axis: Dimension, target: &FlexId) -> u8 {
        match axis {
            Dimension::F => target.f_zoomlevel(),
            Dimension::X => target.x_zoomlevel(),
            Dimension::Y => target.y_zoomlevel(),
        }
    }

    /// レベル `level` の Branch を降りるとき、target と交差しうる子を返す。
    ///
    /// 木を降りる操作（挿入・走査・削除）は、どれもこの判断を起点にする。
    /// 交差判定を [`FlexId::intersection`] に委ねると、真偽値が欲しいだけの場所で
    /// FlexId を構築してしまう。ここは [`covers`](Self::covers) /
    /// [`forking`](Self::forking) のビット演算だけで同じ結論を出す。
    pub(crate) fn overlapping_children(target: &FlexId, level: u8) -> OverlappingChildren {
        if Self::covers(target, level) {
            OverlappingChildren::Both
        } else {
            OverlappingChildren::Only(Self::forking(target, level))
        }
    }

    pub(crate) fn overlapping_children_range(
        target: &crate::RangeId,
        level: u8,
    ) -> OverlappingChildren {
        let axis = Self::axis(level);
        let depth = Self::depth(level);
        let target_z = target.z();

        if depth >= target_z {
            return OverlappingChildren::Both;
        }

        let shift = target_z - 1 - depth;
        let (min_idx, max_idx) = match axis {
            Dimension::F => (target.f()[0] as u32, target.f()[1] as u32),
            Dimension::X => (target.x()[0], target.x()[1]),
            Dimension::Y => (target.y()[0], target.y()[1]),
        };

        let min_path = min_idx >> shift;
        let max_path = max_idx >> shift;

        if min_path == max_path {
            if (min_path & 1) == 0 {
                OverlappingChildren::Only(Side::Lower)
            } else {
                OverlappingChildren::Only(Side::Upper)
            }
        } else {
            OverlappingChildren::Both
        }
    }

    /// target が、この `level` が担当する**1軸**について現在の空間境界を完全に覆うか判定する。
    /// 全軸をまとめて見るのは [`covers_all_axes`](Self::covers_all_axes)。
    fn covers(target: &FlexId, level: u8) -> bool {
        let axis = Self::axis(level);
        let depth = Self::depth(level);
        Self::target_zoom(axis, target) <= depth
    }

    /// target が現在の空間境界を**全軸（F/X/Y）**で完全に覆うか判定する。
    /// 1軸だけ見るのは [`covers`](Self::covers)。
    pub(crate) fn covers_all_axes(target: &FlexId, level: u8) -> bool {
        let passed_f = level.div_ceil(3);
        let passed_x = (level + 1) / 3;
        let passed_y = level / 3;

        target.f_zoomlevel() <= passed_f
            && target.x_zoomlevel() <= passed_x
            && target.y_zoomlevel() <= passed_y
    }

    /// 持続的データ構造に挿入します。
    /// 参照カウントが1の場合は `SharedNode::make_mut` を使用してインプレースで更新します (Copy-on-Write最適化)。
    pub fn insert_mut(
        node: &mut SharedNode<Self>,
        target: &FlexId,
        value: &V,
        level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        // 現在のノードがすでに Leaf であり、値が同一ならそのまま終了(Result Reuse)
        if let Node::Leaf {
            value: Some(ref existing),
        } = **node
            && existing == value
        {
            return;
        }

        // 完全にターゲットが現在の空間全体を覆う場合、O(1)でLeafに置換する
        if Self::covers_all_axes(target, level) {
            *node = SharedNode::new(Node::Leaf {
                value: Some(value.clone()),
            });
            return;
        }

        let node_level = node.node_level();

        let mut current_level = level;

        // 無関係な次元をスキップ
        while current_level < node_level && Self::covers(target, current_level) {
            current_level += 1;
        }

        // 完全に target に覆い尽くされた場合、全体を塗りつぶす
        if current_level >= LEAF_LEVEL {
            *node = SharedNode::new(Node::Leaf {
                value: Some(value.clone()),
            });
            return;
        }

        // 既存のツリーに欠けている階層 (Prepend missing level) を補うために Branch を作成する
        if current_level < node_level {
            let side = Self::forking(target, current_level);

            let (new_lower, new_upper) = match side {
                Side::Lower => {
                    let mut lo = node.clone();
                    Self::insert_mut(&mut lo, target, value, current_level + 1, empty_leaf);
                    (lo, node.clone())
                }
                Side::Upper => {
                    let mut hi = node.clone();
                    Self::insert_mut(&mut hi, target, value, current_level + 1, empty_leaf);
                    (node.clone(), hi)
                }
            };

            *node = Self::mk(current_level, new_lower, new_upper, empty_leaf);
            return;
        }

        // current_level == node_level の場合
        // ここが Copy-on-Write のコア。可能なら node の中身を mutable に取得する。
        let replacement = {
            let mut_node = SharedNode::make_mut(node);

            if let Node::Branch {
                level: l,
                lower_child,
                upper_child,
                leaf_count,
                max_zoom,
                split_mask,
            } = mut_node
            {
                match Self::overlapping_children(target, *l) {
                    OverlappingChildren::Both => {
                        Self::insert_mut(lower_child, target, value, *l + 1, empty_leaf);
                        Self::insert_mut(upper_child, target, value, *l + 1, empty_leaf);
                    }
                    OverlappingChildren::Only(Side::Lower) => {
                        Self::insert_mut(lower_child, target, value, *l + 1, empty_leaf)
                    }
                    OverlappingChildren::Only(Side::Upper) => {
                        Self::insert_mut(upper_child, target, value, *l + 1, empty_leaf)
                    }
                }

                *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
                *max_zoom = Self::fold_max_zoom(*l, lower_child, upper_child);
                *split_mask = Self::fold_split_mask(*l, lower_child, upper_child);

                Self::collapse_equal_children(lower_child, upper_child, *l, empty_leaf)
            } else {
                unreachable!()
            }
        };

        if let Some(rep) = replacement {
            *node = rep;
        }
    }

    /// 持続的データ構造にポリシー付きで挿入します。
    /// 重なりが生じた場合は、指定された resolve ポリシーで値を解決します。
    pub fn insert_mut_with<R>(
        node: &mut SharedNode<Self>,
        target: &FlexId,
        value: &V,
        level: u8,
        empty_leaf: &SharedNode<Node<V>>,
        resolve: &R,
    ) where
        R: Fn(&V, &V) -> V,
    {
        // 完全にターゲットが現在の空間全体を覆う場合
        if Self::covers_all_axes(target, level) {
            Self::fill_or_merge_mut(node, value, empty_leaf, resolve);
            return;
        }

        let node_level = node.node_level();
        let mut current_level = level;

        // 無関係な次元をスキップ
        while current_level < node_level && Self::covers(target, current_level) {
            current_level += 1;
        }

        // 完全に target に覆い尽くされた場合、全体を塗りつぶす/マージする
        if current_level >= LEAF_LEVEL {
            Self::fill_or_merge_mut(node, value, empty_leaf, resolve);
            return;
        }

        // 既存のツリーに欠けている階層を補うために Branch を作成する
        if current_level < node_level {
            let side = Self::forking(target, current_level);

            let (new_lower, new_upper) = match side {
                Side::Lower => {
                    let mut lo = node.clone();
                    Self::insert_mut_with(
                        &mut lo,
                        target,
                        value,
                        current_level + 1,
                        empty_leaf,
                        resolve,
                    );
                    (lo, node.clone())
                }
                Side::Upper => {
                    let mut hi = node.clone();
                    Self::insert_mut_with(
                        &mut hi,
                        target,
                        value,
                        current_level + 1,
                        empty_leaf,
                        resolve,
                    );
                    (node.clone(), hi)
                }
            };

            *node = Self::mk(current_level, new_lower, new_upper, empty_leaf);
            return;
        }

        // current_level == node_level の場合 (Copy-on-Write)
        let replacement = {
            let mut_node = SharedNode::make_mut(node);

            if let Node::Branch {
                level: l,
                lower_child,
                upper_child,
                leaf_count,
                max_zoom,
                split_mask,
            } = mut_node
            {
                match Self::overlapping_children(target, *l) {
                    OverlappingChildren::Both => {
                        Self::insert_mut_with(
                            lower_child,
                            target,
                            value,
                            *l + 1,
                            empty_leaf,
                            resolve,
                        );
                        Self::insert_mut_with(
                            upper_child,
                            target,
                            value,
                            *l + 1,
                            empty_leaf,
                            resolve,
                        );
                    }
                    OverlappingChildren::Only(Side::Lower) => Self::insert_mut_with(
                        lower_child,
                        target,
                        value,
                        *l + 1,
                        empty_leaf,
                        resolve,
                    ),
                    OverlappingChildren::Only(Side::Upper) => Self::insert_mut_with(
                        upper_child,
                        target,
                        value,
                        *l + 1,
                        empty_leaf,
                        resolve,
                    ),
                }

                *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
                *max_zoom = Self::fold_max_zoom(*l, lower_child, upper_child);
                *split_mask = Self::fold_split_mask(*l, lower_child, upper_child);

                Self::collapse_equal_children(lower_child, upper_child, *l, empty_leaf)
            } else {
                unreachable!()
            }
        };

        if let Some(rep) = replacement {
            *node = rep;
        }
    }

    /// node 配下のすべての Leaf について、値があれば resolve(v, value) を適用し、
    /// 空 (None) であれば value で埋める。
    fn fill_or_merge_mut<R>(
        node: &mut SharedNode<Self>,
        value: &V,
        empty_leaf: &SharedNode<Node<V>>,
        resolve: &R,
    ) where
        R: Fn(&V, &V) -> V,
    {
        let replacement = {
            let mut_node = SharedNode::make_mut(node);
            match mut_node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    leaf_count,
                    max_zoom,
                    split_mask,
                } => {
                    Self::fill_or_merge_mut(lower_child, value, empty_leaf, resolve);
                    Self::fill_or_merge_mut(upper_child, value, empty_leaf, resolve);
                    *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
                    *max_zoom = Self::fold_max_zoom(*level, lower_child, upper_child);
                    *split_mask = Self::fold_split_mask(*level, lower_child, upper_child);
                    Self::collapse_equal_children(lower_child, upper_child, *level, empty_leaf)
                }
                Node::Leaf { value: existing } => {
                    if let Some(v) = existing {
                        *v = resolve(v, value);
                    } else {
                        *existing = Some(value.clone());
                    }
                    None
                }
            }
        };
        if let Some(rep) = replacement {
            *node = rep;
        }
    }

    /// 2つの子が値として等価なら、この軸の分割は冗長なので片方へ畳む置換ノードを返す。
    ///
    /// 葉同士（同値）の uniform-fill だけでなく、**等価な非葉サブツリー**（＝その軸が
    /// 効かない＝1段粗くできる異方セル）も畳む。Node の derived `PartialEq` は
    /// `level`/`leaf_count`/`max_zoom`（O(1) キャッシュ）を先に比較して短絡するため、
    /// 等価でない大半の枝は深い比較に入らず弾かれる。
    pub(crate) fn collapse_equal_children(
        lower_child: &SharedNode<Node<V>>,
        upper_child: &SharedNode<Node<V>>,
        level: u8,
        empty_leaf: &SharedNode<Node<V>>,
    ) -> Option<SharedNode<Node<V>>> {
        // 2つの子が値として等価なら、この軸の分割は冗長なので片方へ畳める。
        // 葉同士（uniform-fill）だけでなく、等価な非葉サブツリー（＝その軸が効かない
        // 1段粗い異方セル）も畳む＝FlexId の異方圧縮を効かせる。
        //
        // ただし畳めるのは「畳む軸 axis(level) を子側がこれ以上分割しない」＝この分割が
        // その軸の最深分割のときに限る。さもないと軸の分割深さに途中ギャップができ、
        // 階層細分として表現不能になって座標再構成（[`LeavesIter`] / [`split_child_id`]）が
        // 壊れる（接頭辞のみ＝接尾辞トリムだけが許される不変条件）。
        //
        // 「畳む軸を子がそれ以深で分割しているか」はキャッシュ済み `split_mask` の
        // ビットテストで O(1) に判定する（旧 subtree_splits_axis の O(部分木) 走査を排除）。
        // 子が等価なら両子の split_mask は一致するため、lower 側だけ見れば足りる。
        // ここを先に弾くことで、非等価な大半の枝は深い比較（O(部分木)）に入らない。
        let axis_bit = Self::axis_bit(Self::axis(level));
        if (lower_child.split_mask() & axis_bit) != 0 {
            return None;
        }

        // ptr_eq なら値も必ず等価なので、深い等価比較（O(部分木)）を省く。
        // union 後など構造共有された左右の子で効く。
        if SharedNode::ptr_eq(lower_child, upper_child) || **lower_child == **upper_child {
            Some(if matches!(&**lower_child, Node::Leaf { value: None }) {
                empty_leaf.clone()
            } else {
                lower_child.clone()
            })
        } else {
            None
        }
    }

    /// target の次元ごとのインデックスビットを取得し、Lower / Upper を判定する。
    fn forking(target: &FlexId, level: u8) -> Side {
        let axis = Self::axis(level);
        let depth = Self::depth(level);

        let (target_z, index) = match axis {
            Dimension::F => (target.f_zoomlevel(), target.f_index() as u32),
            Dimension::X => (target.x_zoomlevel(), target.x_index()),
            Dimension::Y => (target.y_zoomlevel(), target.y_index()),
        };

        if depth >= target_z {
            return Side::Lower;
        }

        let shift = target_z - 1 - depth;
        let bit = (index >> shift) & 1;

        if bit == 0 { Side::Lower } else { Side::Upper }
    }

    /// ノード以下すべての値をインプレースで更新します。
    ///
    /// 値の書き換えで隣接領域が同値になった場合は巻き戻しで畳み込む（[`mk`](Self::mk) と
    /// 同じ規則）ため、変換後も正規形を保つ。子が畳み込みで縮むと `leaf_count` /
    /// `max_zoom` も変わりうるため、全キャッシュを再計算する。
    pub(crate) fn map_values_mut<F>(
        node: &mut SharedNode<Node<V>>,
        f: &mut F,
        empty_leaf: &SharedNode<Node<V>>,
    ) where
        F: FnMut(&mut V),
    {
        let replacement = {
            let mut_node = SharedNode::make_mut(node);
            match mut_node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    leaf_count,
                    max_zoom,
                    split_mask,
                } => {
                    Self::map_values_mut(lower_child, f, empty_leaf);
                    Self::map_values_mut(upper_child, f, empty_leaf);
                    *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
                    *max_zoom = Self::fold_max_zoom(*level, lower_child, upper_child);
                    *split_mask = Self::fold_split_mask(*level, lower_child, upper_child);
                    Self::collapse_equal_children(lower_child, upper_child, *level, empty_leaf)
                }
                Node::Leaf { value: Some(v) } => {
                    f(v);
                    None
                }
                Node::Leaf { value: None } => None,
            }
        };
        if let Some(rep) = replacement {
            *node = rep;
        }
    }
}

#[cfg(test)]
impl<V> Node<V>
where
    V: SafeValue,
{
    /// FXY-正規形（N1〜N4）をこのノード配下について再帰検査する。違反時は理由を返す。
    ///
    /// - N1: 畳める Branch（両子が等価かつその軸をそれ以深で分割しない）が存在しない。
    /// - N2: `leaf_count == 0` の Branch が存在しない（空は葉で表現される）。
    /// - N4: 子も正規形。
    /// - あわせてキャッシュ（`leaf_count` / `max_zoom` / `split_mask`）の整合も検査する。
    pub(crate) fn check_canonical(&self) -> Result<(), alloc::string::String> {
        let Node::Branch {
            level,
            leaf_count,
            max_zoom,
            split_mask,
            lower_child,
            upper_child,
        } = self
        else {
            return Ok(());
        };

        lower_child.check_canonical()?;
        upper_child.check_canonical()?;

        let expected_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
        if *leaf_count != expected_count {
            return Err(alloc::format!(
                "level {level}: stale leaf_count {leaf_count} != {expected_count}"
            ));
        }
        let expected_zoom = Self::fold_max_zoom(*level, lower_child, upper_child);
        if *max_zoom != expected_zoom {
            return Err(alloc::format!(
                "level {level}: stale max_zoom {max_zoom} != {expected_zoom}"
            ));
        }
        let expected_mask = Self::fold_split_mask(*level, lower_child, upper_child);
        if *split_mask != expected_mask {
            return Err(alloc::format!(
                "level {level}: stale split_mask {split_mask:#05b} != {expected_mask:#05b}"
            ));
        }

        // N2: 空の Branch は存在してはならない。
        if *leaf_count == 0 {
            return Err(alloc::format!("level {level}: empty branch violates N2"));
        }

        // N1: 畳めるのに畳んでいない Branch は存在してはならない。
        let axis_bit = Self::axis_bit(Self::axis(*level));
        if (lower_child.split_mask() & axis_bit) == 0 && **lower_child == **upper_child {
            return Err(alloc::format!(
                "level {level}: collapsible branch violates N1"
            ));
        }

        Ok(())
    }
}
