use super::ptr::SafeValue;
use super::{
    FlexTreeCore,
    node::{Axis, NUM_AXES, Node},
    ptr::SharedNode,
    split_child_id,
};
use crate::{FlexId, Side};

/// この `FlexId` ちょうどを表すノードが、木のルート（レベル0）から見て位置する絶対レベル。
///
/// `Node::passed` より、サイクル内の位置 `p` の軸を `z` 回割り終えているには
/// `passed(L, p) = (L + (NUM_AXES - 1 - p)) / NUM_AXES >= z`、すなわち
/// `L >= NUM_AXES * z - (NUM_AXES - 1 - p)` が必要なので、全軸のこの下限の最大値がレベルになる。
/// 3軸のときは `f_zoom + x_zoom + y_zoom` に一致する。
fn node_level_of(id: &FlexId) -> u8 {
    const fn need(zoom: u8, position: u8) -> i32 {
        NUM_AXES as i32 * zoom as i32 - (NUM_AXES as i32 - 1 - position as i32)
    }

    #[cfg_attr(not(feature = "temporal_id"), allow(unused_mut))]
    let mut level = need(id.f_zoomlevel(), 0)
        .max(need(id.x_zoomlevel(), 1))
        .max(need(id.y_zoomlevel(), 2));

    // 3軸版にTの番は無い（時間は常に全時間）。
    #[cfg(feature = "temporal_id")]
    {
        level = level.max(need(id.t_zoomlevel(), 3));
    }

    level.max(0) as u8
}

/// [`split_child_id`] と同じだが、その軸が最大ズームに達していれば [`None`] を返す。
///
/// [`split_shard`](FlexTreeCore::split_shard) は木を降りるのではなくシャード領域を自分で
/// 割るため、これ以上割れない軸に当たりうる。木の降下側（[`split_child_id`]）は
/// 「分割が必要＝まだ割れる」ことが保証されているので `unwrap` のままでよい。
fn split_child_id_checked(current_id: &FlexId, axis: Axis, side: Side) -> Option<FlexId> {
    match axis {
        Axis::F => current_id.split_f(side),
        Axis::X => current_id.split_x(side),
        Axis::Y => current_id.split_y(side),
        Axis::T => current_id.split_t(side),
    }
}

impl<V> FlexTreeCore<V>
where
    V: SafeValue,
{
    /// この[`FlexTreeCore`]をシャード分割すべきかを判定する。保持する[FlexId]数が `max_flex_id_count` を超えていれば `true`を返す。[FlexId]の個数はキャッシュされているため高速に動作する。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.count() > max_flex_id_count
    }

    /// このシャード（[`shard`](Self::shard) 領域）を、現在のrootの軸で2分割し、切り取った部分木を `((下のシャード領域, 下の実体), (上のシャード領域, 上の実体))` で返す。
    /// シャード領域が未設定なら `None`を返す。
    /// 分割した2枚には、親の [`shard_path`](Self::shard_path) を1段伸ばしたパスが入る
    /// （親のパスが不明なら子も不明）。領域からパスを復元することはできないので、
    /// **分割の瞬間に記録するこの経路が唯一の生成点**である。
    pub(crate) fn split_shard(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let region = *self.shard()?;
        let axis = Self::region_split_axis(&region, self.has_temporal_split());
        let lower = split_child_id_checked(&region, axis, Side::Lower)?;
        let upper = split_child_id_checked(&region, axis, Side::Upper)?;

        let mut lower_piece = self.extract_region(lower);
        let mut upper_piece = self.extract_region(upper);
        lower_piece.shard_path = self.shard_path.as_ref().map(|p| p.child(Side::Lower));
        upper_piece.shard_path = self.shard_path.as_ref().map(|p| p.child(Side::Upper));

        Some(((lower, lower_piece), (upper, upper_piece)))
    }

    /// シャード領域を次に分割すべき軸を返す。
    ///
    /// 対象の軸をこの順（F→X→Y→T）に巡回し、「まだ最も浅い軸」を選ぶ。各軸のズームの和を
    /// 軸数で割った余りがちょうどそれになる（各段で最も浅い軸を割る限り、軸間のズームは
    /// 高々1しか開かないため）。全軸ズーム0（＝全空間）は和が0なのでF軸から始まる。
    ///
    /// 「覆っていない最初のレベル」を返してはならない。それだとFが1段でも深いと常に
    /// F軸が選ばれ続け、シャードがF方向の薄いスライスに退化する。
    ///
    /// # `has_temporal` でT軸を出し入れする理由
    ///
    /// 木がT軸で分割されていない＝全ての葉が全時間のとき、T軸で割ると
    /// 「前半に全部の葉」「後半に全部の葉」という**両方が元と同じ葉数を持つ**2枚になる。
    /// [`should_split_shard`](Self::should_split_shard) は `count()` で判定するので、
    /// これを選ぶと分割しても件数が減らずシャーディングが収束しない。
    /// 時間方向に実際の構造がある木でだけTを巡回に加える。
    fn region_split_axis(region: &FlexId, has_temporal: bool) -> Axis {
        let spatial =
            region.f_zoomlevel() as u32 + region.x_zoomlevel() as u32 + region.y_zoomlevel() as u32;

        if !has_temporal {
            return match spatial % 3 {
                0 => Axis::F,
                1 => Axis::X,
                _ => Axis::Y,
            };
        }

        match (spatial + region.t_zoomlevel() as u32) % 4 {
            0 => Axis::F,
            1 => Axis::X,
            2 => Axis::Y,
            _ => Axis::T,
        }
    }

    pub(crate) fn extract_region(&self, region: FlexId) -> Self {
        let in_lower = region.f_index() < 0;

        // 任意領域での切り出しはシャード木の1段分の分割とは限らないので、位置は引き継がない
        // （正しい子のパスは [`split_shard`](Self::split_shard) が入れ直す）。
        // `self.clone()` だと `shard_path` の `Vec` を確保してすぐ捨てることになるので組み立てる。
        let mut piece = Self {
            lower_root: self.lower_root.clone(),
            upper_root: self.upper_root.clone(),
            empty_leaf: self.empty_leaf.clone(),
            shard: self.shard,
            shard_path: None,
        };
        {
            let (root, root_id) = if in_lower {
                (&mut piece.lower_root, FlexId::LOWER_MAX)
            } else {
                (&mut piece.upper_root, FlexId::UPPER_MAX)
            };
            Self::prune_path(root, root_id, &region, &self.empty_leaf);
        }
        if in_lower {
            piece.upper_root = self.empty_leaf.clone();
        } else {
            piece.lower_root = self.empty_leaf.clone();
        }
        piece.shard = Some(region);
        piece
    }

    /// `node`（`current_id` の領域を占める部分木）を `region` の内側だけに刈り込む。
    ///
    /// # `region` が木の中に実在しない場合
    ///
    /// **`region` がノードとして実体化されていることを前提にしてはならない。** 木は「対象が
    /// その軸を覆っている」レベルを実体化せず読み飛ばすため、`region` に対応するノードは
    /// 存在しないことがある。特に、全Segmentが全時間のデータではT軸の分岐が1つも無いので、
    /// `t_zoomlevel > 0` の領域はどこにも現れない。
    ///
    /// そのため停止条件は「`current_id == region`」ではなく「`current_id` が `region` に
    /// 収まったか」であり、降下も片側だけを選ぶのではなく**両方の子へ降りる**
    /// （`region` が今の軸より浅ければ、`region` は両方の子にまたがる）。
    /// 葉が `region` より粗ければ、その葉を `region` でクリップし直す。
    fn prune_path(
        node: &mut SharedNode<Node<V>>,
        current_id: FlexId,
        region: &FlexId,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        let overlap = current_id.intersection(region);

        // `region` の内側に入りきった → この部分木はまるごと残す。
        if overlap == Some(current_id) {
            return;
        }
        // `region` と交差しない → 空にする。
        if overlap.is_none() {
            *node = empty_leaf.clone();
            return;
        }

        // ここは「交差はするが `current_id` が `region` に収まっていない」＝ `region` が内側。
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
                    let axis = Node::<V>::axis(*level);
                    // `region` は両方の子にまたがりうる。交差しない子は再帰の入口で空になる。
                    let lower_id = split_child_id(&current_id, axis, Side::Lower);
                    let upper_id = split_child_id(&current_id, axis, Side::Upper);
                    Self::prune_path(lower_child, lower_id, region, empty_leaf);
                    Self::prune_path(upper_child, upper_id, region, empty_leaf);

                    *leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
                    *max_zoom = Node::<V>::fold_max_zoom(*level, lower_child, upper_child);
                    *split_mask = Node::<V>::fold_split_mask(*level, lower_child, upper_child);

                    // 片側を空にした結果、左右が等価化／両側空になったら畳んで正規形を保つ。
                    Node::<V>::collapse_equal_children(lower_child, upper_child, *level, empty_leaf)
                }
                Node::Leaf { value: None } => None,
                Node::Leaf { value: Some(value) } => {
                    // `region` より粗い葉。`region` の内側にだけ値を残す。
                    //
                    // 枝を自前で組まず `insert_mut` に任せる。あちらは「既存の木に欠けている
                    // 階層を補う」経路（`current_level < node_level`）を持っているので、
                    // 空葉から挿し直せば必要な枝が正規形のまま生える。
                    let value = value.clone();
                    let mut clipped = empty_leaf.clone();
                    Node::insert_mut(
                        &mut clipped,
                        region,
                        &value,
                        node_level_of(&current_id),
                        empty_leaf,
                    );
                    Some(clipped)
                }
            }
        };

        if let Some(rep) = replacement {
            *node = rep;
        }
    }
}
