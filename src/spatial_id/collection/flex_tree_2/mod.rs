//! [`FlexTreeCore`](crate::spatial_id::collection::flex_tree::core::FlexTreeCore)の後継案。
//!
//! 旧設計はF→X→Y(→T)を1レベルにつき1軸1ビットずつ分岐する厳密な2分木で、
//! `collapse_equal_children`は両子が完全に等価なときしか畳まない。そのため疎なデータ
//! (例: 深いzoomの点を1つだけ挿入)では、ルート(半球全体, zoom0)から実データまでの間、
//! 片方の子が常に空なだけの単鎖Branchが軸あたり最大30段、3〜4軸ぶん実体化されてしまう。
//!
//! `FlexTreeCore2`はこれを解決するため、すべての`Branch`が自分の担当セル
//! (`cell: FlexId`)を持つ(相対root/パス圧縮)。分岐方式は旧設計と同じ1軸ずつの2分岐を
//! 維持する。`FlexId`は軸ごとに異なるズームを持てる(異方圧縮)ため、複数軸を同時に
//! 分岐するオクツリー方式だと「ある軸だけ解像度が尽きているSegment」が複数の
//! 子スロットにまたがってしまい破綻する。1軸ずつ分岐する旧方式ならこの問題が無い
//! (旧`Node::covers`/`forking`と同じロジックがそのまま使える)。
//!
//! # データ構造
//!
//! - [`Node::Leaf`]は自分の`id`(実際のFlexId)を直接持つ。パス圧縮により祖先からは
//!   復元できないため。
//! - [`Node::Branch`]は`cell`(配下すべてを包む共通祖先)と`level`(実際に分岐する
//!   軸・深さ。旧`Axis::axis(level)`/`Node::depth(level)`と同じ意味)を持つ。
//!
//! 値の無い領域はノード自体が存在しない疎表現なので、旧設計の`Leaf{value: None}`や
//! N2(空Branch禁止)は無い。維持すべき不変条件は2つ:
//!
//! - **必須分岐点**: すべての`Branch`は実際に2つ以上の子孫が分かれる最小のセルで
//!   なければならない([`diverge`]が構成的に保証する)。
//! - **値一様マージ**: 2つの子がどちらも`Leaf`で値が等しく、かつ両者を合わせると
//!   ちょうど親の分岐を過不足なく覆うなら、1つの`Leaf`へ畳む([`mk_branch`])。

use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::node::{Axis, NUM_AXES, Node as OldNode};
use crate::spatial_id::flex_id::ops::common_ancestor_axis;
use crate::{FlexId, Side, SpatialId};

#[cfg(test)]
mod tests;

pub enum Node<V> {
    /// 単独のSegment。`id`はこのLeafが正確に表す領域(異方圧縮された実際の領域)。
    Leaf { id: FlexId, value: V },
    /// 分岐点。`cell`は配下すべてを包む共通祖先セル。`level`は実際に分岐する軸・深さ。
    Branch {
        cell: FlexId,
        level: u8,
        lower: Box<Node<V>>,
        upper: Box<Node<V>>,
        leaf_count: u32,
    },
}

impl<V: Clone> Clone for Node<V> {
    fn clone(&self) -> Self {
        match self {
            Node::Leaf { id, value } => Node::Leaf {
                id: *id,
                value: value.clone(),
            },
            Node::Branch {
                cell,
                level,
                lower,
                upper,
                leaf_count,
            } => Node::Branch {
                cell: *cell,
                level: *level,
                lower: lower.clone(),
                upper: upper.clone(),
                leaf_count: *leaf_count,
            },
        }
    }
}

impl<V> Node<V> {
    fn leaf_count(&self) -> u32 {
        match self {
            Node::Leaf { .. } => 1,
            Node::Branch { leaf_count, .. } => *leaf_count,
        }
    }
}

#[derive(Default)]
pub struct FlexTreeCore2<V> {
    upper: Option<Box<Node<V>>>,
    lower: Option<Box<Node<V>>>,
}

impl<V> FlexTreeCore2<V>
where
    V: Clone + PartialEq,
{
    pub fn new() -> Self {
        Self {
            upper: None,
            lower: None,
        }
    }

    /// このツリーが配下すべてを包む領域(相対root)。半球ごとに、`Leaf`ならその`id`、
    /// `Branch`ならその`cell`として導出する(独立フィールドとしては持たない)。
    pub fn upper_root(&self) -> Option<FlexId> {
        self.upper.as_deref().map(Node::region)
    }

    pub fn lower_root(&self) -> Option<FlexId> {
        self.lower.as_deref().map(Node::region)
    }

    pub fn insert<S: SpatialId>(&mut self, target: S, value: V) {
        for flex_id in target.into_iter() {
            let slot = if flex_id.f_index().is_positive() {
                &mut self.upper
            } else {
                &mut self.lower
            };

            *slot = Some(Box::new(match slot.take() {
                None => Node::Leaf {
                    id: flex_id,
                    value: value.clone(),
                },
                Some(existing) => merge(*existing, flex_id, value.clone()),
            }));
        }
    }
}

impl<V> Node<V> {
    /// このノード配下すべてを包む領域。`Leaf`なら`id`、`Branch`なら`cell`。
    fn region(&self) -> FlexId {
        match self {
            Node::Leaf { id, .. } => *id,
            Node::Branch { cell, .. } => *cell,
        }
    }
}

// --- レベル ⇔ (軸, 深さ) の変換・降下判定 ---
//
// 旧`flex_tree::core::node::Node`の同名の純粋関数(`Node<V>`には依存しない)をそのまま
// 呼び出す薄いラッパ。`SafeValue`境界を満たす具体型が要るので`()`を使う(値には無関係)。

fn axis_of(level: u8) -> Axis {
    OldNode::<()>::axis(level)
}

fn depth_of(level: u8) -> u8 {
    OldNode::<()>::depth(level)
}

/// `target`のこのレベルでの分岐先(旧`Node::forking`)。
fn forking(target: &FlexId, level: u8) -> Side {
    OldNode::<()>::forking(target, level)
}

/// 木の巡回順(F→X→Y→T)における軸の位置。`axis_of`の逆写像。
fn axis_position(axis: Axis) -> u8 {
    match axis {
        Axis::F => 0,
        Axis::X => 1,
        Axis::Y => 2,
        Axis::T => 3,
    }
}

fn level_of(axis: Axis, depth: u8) -> u8 {
    depth * NUM_AXES + axis_position(axis)
}

fn axis_zoom(id: &FlexId, axis: Axis) -> u8 {
    match axis {
        Axis::F => id.f_zoomlevel(),
        Axis::X => id.x_zoomlevel(),
        Axis::Y => id.y_zoomlevel(),
        Axis::T => id.t_zoomlevel(),
    }
}

fn split_axis(id: &FlexId, axis: Axis, side: Side) -> Option<FlexId> {
    match axis {
        Axis::F => id.split_f(side),
        Axis::X => id.split_x(side),
        Axis::Y => id.split_y(side),
        Axis::T => id.split_t(side),
    }
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Lower => Side::Upper,
        Side::Upper => Side::Lower,
    }
}

/// `side`が`Lower`/`Upper`のどちらかに応じて`(lower, upper)`を組み立てる。
fn place<V>(side_a: Side, node_a: Node<V>, node_b: Node<V>) -> (Node<V>, Node<V>) {
    match side_a {
        Side::Lower => (node_a, node_b),
        Side::Upper => (node_b, node_a),
    }
}

/// `a`と`b`が最初に分かれる絶対レベルを返す。`a == b`(全軸で完全に一致)なら`None`。
///
/// 軸ごとに[`common_ancestor_axis`]を呼び、返ってきた共有深さ`z`が
/// `z1.min(z2)`より縮んでいれば、その軸で実際に分岐が起きたことを意味する
/// (縮んでいなければ、一方が他方を覆っているだけで分岐しない)。分岐が起きた軸のうち
/// 絶対レベル(`z * NUM_AXES + 軸の巡回位置`)が最小のものが、木の巡回順で最初に
/// 訪れる分岐点になる。
fn diverge(a: &FlexId, b: &FlexId) -> Option<u8> {
    let mut best: Option<u8> = None;

    macro_rules! consider {
        ($axis:expr, $z1:expr, $i1:expr, $z2:expr, $i2:expr) => {
            let z1 = $z1;
            let z2 = $z2;
            // z1 == z2 == z (縮んでいない) なら、この軸は完全に一致していて情報が無い
            // ("等しい"か、一方がもう一方の中に完全に収まっているだけ)ので分岐点の
            // 候補にならない。z < z1 または z < z2 (どちらか一方でも縮んでいる)なら、
            // 実際にビットが食い違ったか、一方がこの軸で尽きていて他方が続いている
            // (=粗い領域の内側に細かい領域を挿すケース)ので分岐点の候補になる。
            if let Some((z, _)) = common_ancestor_axis(z1, $i1 as i64, z2, $i2 as i64)
                && z < z1.max(z2)
            {
                let level = level_of($axis, z);
                best = Some(match best {
                    Some(cur) => cur.min(level),
                    None => level,
                });
            }
        };
    }

    consider!(
        Axis::F,
        a.f_zoomlevel(),
        a.f_index(),
        b.f_zoomlevel(),
        b.f_index()
    );
    consider!(
        Axis::X,
        a.x_zoomlevel(),
        a.x_index(),
        b.x_zoomlevel(),
        b.x_index()
    );
    consider!(
        Axis::Y,
        a.y_zoomlevel(),
        a.y_index(),
        b.y_zoomlevel(),
        b.y_index()
    );
    #[cfg(feature = "temporal_id")]
    consider!(Axis::T, a.t_zoomlevel(), a.t(), b.t_zoomlevel(), b.t());

    best
}

/// 2つの子から`Branch`(または値一様マージなら`Leaf`)を構成する唯一の入口。
///
/// 両子が`Leaf`で値が等しく、かつそれぞれの`id`がちょうど`(cell, level)`の分岐を
/// 過不足なく覆う(`cell`をその軸のその深さで単純に2分した領域と一致する)なら、
/// 1つの`Leaf{cell, value}`へ畳む。これが無いと、同じ論理内容でも挿入粒度
/// (1回の粗い挿入 vs 2回の細かい挿入)によって物理構造が変わってしまう。
fn mk_branch<V: PartialEq>(cell: FlexId, level: u8, lower: Node<V>, upper: Node<V>) -> Node<V> {
    let axis = axis_of(level);
    if let (
        Node::Leaf {
            id: lo_id,
            value: lo_v,
        },
        Node::Leaf {
            id: up_id,
            value: up_v,
        },
    ) = (&lower, &upper)
        && lo_v == up_v
        && Some(*lo_id) == split_axis(&cell, axis, Side::Lower)
        && Some(*up_id) == split_axis(&cell, axis, Side::Upper)
    {
        let value = match lower {
            Node::Leaf { value, .. } => value,
            Node::Branch { .. } => unreachable!(),
        };
        return Node::Leaf { id: cell, value };
    }

    let leaf_count = lower.leaf_count() + upper.leaf_count();
    Node::Branch {
        cell,
        level,
        lower: Box::new(lower),
        upper: Box::new(upper),
        leaf_count,
    }
}

/// `node`(担当領域`bound`)を軸`axis`の深さ`depth`ちょうどで`side`側だけに絞り込む。
///
/// - `node`がその軸で`depth`より深い実ビットを持つなら、そのビットが`side`と
///   一致する場合だけ`node`をそのまま残す(一致しなければ`None`)。
/// - ちょうど`depth`で尽きている(それ以上の情報を持たない)なら、`side`側へ
///   1段narrowして残す(反対側にも同じ内容が及んでいたことになる)。
/// - `Branch`はまず`cell`だけで判定できないか試し(全体がどちらかの側に確定するなら
///   部分木を丸ごと辿らずに済む)、できなければ両子を再帰的にnarrowして組み直す。
fn narrow_to_side<V: Clone + PartialEq>(
    node: Node<V>,
    axis: Axis,
    depth: u8,
    side: Side,
) -> Option<Node<V>> {
    let bound = node.region();
    let z = axis_zoom(&bound, axis);

    if z > depth {
        let level = level_of(axis, depth);
        return if forking(&bound, level) == side {
            Some(node)
        } else {
            None
        };
    }

    debug_assert_eq!(z, depth, "呼び出し側はcontainsで z >= depth を保証すること");

    match node {
        Node::Leaf { id, value } => Some(Node::Leaf {
            id: split_axis(&id, axis, side).expect("depthちょうどのidは最大ズーム未満のはず"),
            value,
        }),
        Node::Branch {
            cell,
            level,
            lower,
            upper,
            ..
        } => {
            let lo = narrow_to_side(*lower, axis, depth, side);
            let up = narrow_to_side(*upper, axis, depth, side);
            let new_cell =
                split_axis(&cell, axis, side).expect("depthちょうどのcellは最大ズーム未満のはず");
            match (lo, up) {
                (None, None) => None,
                (Some(only), None) | (None, Some(only)) => Some(only),
                (Some(lo), Some(up)) => Some(mk_branch(new_cell, level, lo, up)),
            }
        }
    }
}

/// 担当領域`bound`を持つ既存の内容`existing`へ、それとは異なる`bound`を持つ
/// (=[`diverge`]で分岐点が見つかる)`target`を挿入する。
///
/// `existing`は`Leaf`(単独のSegment)にも`Branch`(部分木)にもなりうる。どちらでも
/// 「`bound`が分岐軸で実ビットを持つか、ちょうど尽きているか」で場合分けするロジックは
/// 同じ: 前者なら`existing`はまるごと片側へ、後者なら`existing`を両側へnarrowして
/// 反対側は再帰的に`target`とマージし続ける(旧設計の「既存ノードを両側へcloneし、
/// target側だけさらに挿入する」処理を、パス圧縮のためnarrowを挟んで一般化したもの)。
fn promote<V: Clone + PartialEq>(
    bound: FlexId,
    existing: Node<V>,
    target: FlexId,
    value: V,
) -> Node<V> {
    let level = diverge(&bound, &target).expect("呼び出し側は bound != target を保証すること");
    let axis = axis_of(level);
    let depth = depth_of(level);
    let cell = bound
        .common_ancestor(&target)
        .expect("同じ半球内のFlexId同士なのでcommon_ancestorは必ず成功する");

    if axis_zoom(&bound, axis) > depth {
        // 実分岐: bound はこの軸ですでに実ビットを持つ。existing はまるごと片側へ、
        // target は新規Leafとしてもう片側へ置く。
        let existing_side = forking(&bound, level);
        let (lower, upper) = place(existing_side, existing, Node::Leaf { id: target, value });
        mk_branch(cell, level, lower, upper)
    } else {
        // ちょうど尽きている: existing は両側に及んでいたので、両側へnarrowする。
        // target側はnarrowした内容へさらに挿入を続ける(narrowできなければ空だったので
        // 新規Leafで十分)。
        let target_side = forking(&target, level);
        let existing_side = opposite(target_side);

        let existing_child = narrow_to_side(existing.clone(), axis, depth, existing_side)
            .expect("boundがdepthちょうどなら少なくとも一方の側には内容が残るはず");
        let target_child = match narrow_to_side(existing, axis, depth, target_side) {
            Some(narrowed) => merge(narrowed, target, value),
            None => Node::Leaf { id: target, value },
        };

        let (lower, upper) = place(existing_side, existing_child, target_child);
        mk_branch(cell, level, lower, upper)
    }
}

/// `existing`へ`(target, value)`を挿入した結果を返す。
fn merge<V: Clone + PartialEq>(existing: Node<V>, target: FlexId, value: V) -> Node<V> {
    match existing {
        Node::Leaf { id, value: old } => {
            if id == target {
                Node::Leaf { id, value }
            } else if target.contains(&id) {
                // targetがidを完全に包含(より粗い上書き) → idの値は消える。
                Node::Leaf { id: target, value }
            } else {
                promote(id, Node::Leaf { id, value: old }, target, value)
            }
        }
        Node::Branch {
            cell,
            level,
            lower,
            upper,
            leaf_count,
        } => {
            if target.contains(&cell) {
                // targetがこのBranch全体を完全に包含 → まるごと上書き。
                Node::Leaf { id: target, value }
            } else if cell.contains(&target) {
                let side = forking(&target, level);
                let (lower, upper) = match side {
                    Side::Lower => (merge(*lower, target, value), *upper),
                    Side::Upper => (*lower, merge(*upper, target, value)),
                };
                mk_branch(cell, level, lower, upper)
            } else {
                promote(
                    cell,
                    Node::Branch {
                        cell,
                        level,
                        lower,
                        upper,
                        leaf_count,
                    },
                    target,
                    value,
                )
            }
        }
    }
}
