//! 木の降下（枝刈り走査）を、ノード表現によらず1か所に集約する。
//!
//! FlexTree の走査は2つの表現に対して行われる:
//!
//! - `Arc` ベースの作業木（[`Node`]）
//! - 永続化バイト列の archived 表現（`ArchivedArenaNode` + アリーナ）
//!
//! 枝刈りの判定そのもの（[`Node::overlapping_children`] / [`Node::axis`] /
//! [`split_child_id`]）は元から共有されていたが、**スタックを回す定型**が両側に
//! 重複していた。ここではノードから「Branch なら子2つ」を取り出す最小の口
//! （[`TreeCursor`]）だけを抽象し、走査本体を共通化する。
//!
//! 葉に到達したときの扱い（値の取り出し方、`target` で切り取るか否か）は
//! 表現ごと・用途ごとに違うため、ここでは**葉のカーソルをそのまま返す**。
//! 切り取りの有無は呼び出し側の責務。

use alloc::vec::Vec;

use super::node::{Node, OverlappingChildren};
use super::split_child_id;
use crate::{FlexId, RangeId, Side};

/// 木を降りるために必要な最小限の操作。
///
/// `Copy` を要求するのは、スタックへ積む際に安価に複製できる軽い参照
/// （`&Node<V>` や `(アリーナ, インデックス)`）だけを想定しているため。
pub(crate) trait TreeCursor: Copy {
    /// Branch なら `(level, 下側の子, 上側の子)`。Leaf なら `None`。
    fn branch(self) -> Option<(u8, Self, Self)>;
}

impl<V> TreeCursor for &Node<V>
where
    V: super::ptr::SafeValue,
{
    fn branch(self) -> Option<(u8, Self, Self)> {
        match self {
            Node::Branch {
                level,
                lower_child,
                upper_child,
                ..
            } => Some((*level, lower_child.as_ref(), upper_child.as_ref())),
            Node::Leaf { .. } => None,
        }
    }
}

/// `target`（単一セル）と交差する葉だけを辿るスタック走査。
///
/// 交差しない子は**積む前に**捨てるため、点クエリでは葉まで一直線に降りる。
pub(crate) struct OverlapWalk<C> {
    target: FlexId,
    stack: Vec<(C, FlexId)>,
}

impl<C: TreeCursor> OverlapWalk<C> {
    /// 走査開始点（すでに `target` と交差することが分かっているルート）から始める。
    pub(crate) fn new(roots: Vec<(C, FlexId)>, target: FlexId) -> Self {
        Self {
            target,
            stack: roots,
        }
    }
}

impl<C: TreeCursor> Iterator for OverlapWalk<C> {
    /// 葉の `(この葉が占める FlexId, 葉のカーソル)`。
    ///
    /// 交差する子しか積まないので、返る葉は必ず `target` と交差する。
    type Item = (FlexId, C);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((cursor, current_id)) = self.stack.pop() {
            match cursor.branch() {
                Some((level, lower, upper)) => {
                    push_children(&mut self.stack, &current_id, level, lower, upper, |level| {
                        Node::<()>::overlapping_children(&self.target, level)
                    });
                }
                None => return Some((current_id, cursor)),
            }
        }
        None
    }
}

/// `target`（範囲）と交差する葉だけを辿るスタック走査。
///
/// F はズーム0で `0`（上半球）/ `-1`（下半球）の2セルしか無いので、範囲を半球ごとに
/// 割ってから該当するルートだけを降りる。`roots` はその割り済みの開始点。
pub(crate) struct RangeOverlapWalk<C> {
    roots: Vec<(C, FlexId, RangeId)>,
    current_target: Option<RangeId>,
    stack: Vec<(C, FlexId)>,
}

impl<C: TreeCursor> RangeOverlapWalk<C> {
    pub(crate) fn new(roots: Vec<(C, FlexId, RangeId)>) -> Self {
        Self {
            roots,
            current_target: None,
            stack: Vec::new(),
        }
    }
}

impl<C: TreeCursor> Iterator for RangeOverlapWalk<C> {
    type Item = (FlexId, C);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // `current_target` の取り出しは**ルート1本につき1回**。内側の while で
            // ノードごとに `Option` を展開すると、小さい木では固定費が支配して
            // 目に見えて遅くなる（実測で +10〜29%）。
            if let Some(target) = self.current_target.as_ref() {
                while let Some((cursor, current_id)) = self.stack.pop() {
                    match cursor.branch() {
                        Some((level, lower, upper)) => {
                            push_children(
                                &mut self.stack,
                                &current_id,
                                level,
                                lower,
                                upper,
                                |level| Node::<()>::overlapping_children_range(target, level),
                            );
                        }
                        None => return Some((current_id, cursor)),
                    }
                }
            }
            let (root, root_id, root_target) = self.roots.pop()?;
            self.current_target = Some(root_target);
            self.stack.push((root, root_id));
        }
    }
}

/// Branch を降りる際、`overlapping` が交差しうると判定した子だけを積む。
///
/// 積んだ後に [`FlexId::intersection`] で捨て直す必要が無いよう、ここで落とし切る。
fn push_children<C: TreeCursor>(
    stack: &mut Vec<(C, FlexId)>,
    current_id: &FlexId,
    level: u8,
    lower: C,
    upper: C,
    overlapping: impl FnOnce(u8) -> OverlappingChildren,
) {
    let axis = Node::<()>::axis(level);
    let mut push = |side: Side, child: C| {
        stack.push((child, split_child_id(current_id, axis, side)));
    };

    match overlapping(level) {
        OverlappingChildren::Both => {
            push(Side::Upper, upper);
            push(Side::Lower, lower);
        }
        OverlappingChildren::Only(Side::Lower) => push(Side::Lower, lower),
        OverlappingChildren::Only(Side::Upper) => push(Side::Upper, upper),
    }
}
