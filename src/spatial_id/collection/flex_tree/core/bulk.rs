//! 一様ズームの [`SingleId`](crate::SingleId) 列からの、木のボトムアップ一括構築。
//!
//! # なぜ必要か
//!
//! [`insert`](FlexTreeCore::insert) は [`FlexId`](crate::FlexId) 1 個につき根から葉まで
//! 降りる。実データのズーム 24 では 1 本の経路が 70 段を超えるため、N 件の構築は
//! O(N × 深さ) のノード訪問になる。各段で `make_mut` の参照カウント確認・`leaf_count` /
//! `max_zoom` / `split_mask` の畳み上げ・[`collapse_equal_children`](Node::collapse_equal_children)
//! の判定が走るので、1 段あたりの定数も小さくない。
//!
//! ところが**木のノード総数は高々 O(N)** しかない。300 万葉なら 70 段 × 300 万 =
//! 2 億回のノード訪問を払っているのに、実際に作られるノードは 600 万個ほどしかない。
//! 差は「兄弟同士が共有する接頭辞を、1 件ごとに何度も降り直している」ぶんである。
//!
//! # やっていること
//!
//! 空間 3 軸のズームが揃っていれば、木の降下順（`F→X→Y` を 1 ビットずつ織り込んだ
//! Morton 順）で並べるだけで、**任意の部分木がスライスの連続区間になる**。あとは
//! 区間を再帰的に二分して葉から組み上げればよく、各ノードをちょうど 1 回だけ作れば済む。
//! ノード訪問は O(ノード数) に落ちる。
//!
//! 組み上げは [`Node::mk`] を通すので、正規形（FXY-正規形）は構成的に保証される。
//! 一様ズームで組んでも、`mk` の畳み込みが「その軸が効いていない」枝を畳むため、
//! 結果は逐次 `insert` と同じ異方圧縮された木になる。
//!
//! # 前提
//!
//! - 空間 3 軸のズームが `z` で揃っていること。
//! - すべて全時間（`t_zoomlevel == 0`）であること。時間軸で分割された木は一様性が
//!   崩れるので、呼び出し側が従来経路へフォールバックする。
//! - 位置が一意で、[`sort_and_dedup`] の順に整列済みであること。

use alloc::vec::Vec;

use super::FlexTreeCore;
use super::node::{Axis, LEAF_LEVEL, Node};
use super::node_ops::join_at;
use super::ptr::{MaybeSync, SafeValue, SharedNode};

/// 一様ズームの [`SingleId`](crate::SingleId) と、それに紐づく値。`(f, x, y, 値)`。
///
/// ズームは列全体で共通なので個々には持たない。時間は全時間に固定。
pub(crate) type SingleEntry<V> = (i32, u32, u32, V);

/// 1 バイトのビットを 3 ビット間隔へ広げる表。Morton キーの組み立てに使う。
const SPREAD: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut b = 0usize;
    while b < 256 {
        let mut r = 0u32;
        let mut i = 0;
        while i < 8 {
            r |= (((b as u32) >> i) & 1) << (3 * i);
            i += 1;
        }
        table[b] = r;
        b += 1;
    }
    table
};

/// 30 ビット以下の値のビットを 3 ビット間隔へ広げる。
#[inline]
fn spread3(v: u32) -> u128 {
    (SPREAD[((v >> 24) & 0x3F) as usize] as u128) << 72
        | (SPREAD[((v >> 16) & 0xFF) as usize] as u128) << 48
        | (SPREAD[((v >> 8) & 0xFF) as usize] as u128) << 24
        | (SPREAD[(v & 0xFF) as usize] as u128)
}

/// 木の降下順（浅い深度から `F→X→Y` の順に 1 ビットずつ）に対応する整列キー。
///
/// F は負値を取るので、[`Node::forking`] と同じく 2 の補数のビットをそのまま使う。
/// 上下ルートで符号は分離されるため、根の中では単調（`f + 2^z` と同じ並び）になる。
#[inline]
pub(crate) fn morton_key(f: i32, x: u32, y: u32) -> u128 {
    (spread3(f as u32) << 2) | (spread3(x) << 1) | spread3(y)
}

/// 整列キー全体。上ルート（F ≧ 0）が先、下ルート（F < 0）が後ろに来る。
#[inline]
fn sort_key<V>(entry: &SingleEntry<V>) -> (bool, u128) {
    (entry.0 < 0, morton_key(entry.0, entry.1, entry.2))
}

/// 一様ズーム `z` において、全軸を消化しきる（＝葉になる）レベル。
///
/// [`Node::covers_all_axes`] と同じ判定なので、軸数（`temporal_id` の有無で 3 or 4）が
/// 変わっても追従するよう、式を置かず実際に走査して求める。z ごとに一度だけ呼ぶ。
fn leaf_level_for<V: SafeValue>(z: u8) -> u8 {
    let probe = crate::FlexId::new(z, 0, z, 0, z, 0).expect("z はズーム範囲内");
    let mut level = 0u8;
    while level < LEAF_LEVEL && !Node::<V>::covers_all_axes(&probe, level) {
        level += 1;
    }
    level
}

impl<V> FlexTreeCore<V>
where
    V: SafeValue,
{
    /// 空間 3 軸のズームが `z` で揃った全時間の [`SingleEntry`] 列から木を構築する。
    ///
    /// `entries` は位置が一意で [`sort_and_dedup`] の順に整列済みでなければならない
    /// （デバッグビルドでは検査する）。上下ルートの振り分けは F の符号で行う。
    pub(crate) fn from_uniform_single_ids(z: u8, entries: &[SingleEntry<V>]) -> Self {
        let empty_leaf = SharedNode::new(Node::Leaf { value: None });

        debug_assert!(
            entries
                .windows(2)
                .all(|w| sort_key(&w[0]) < sort_key(&w[1])),
            "SingleId が昇順・一意で並んでいない"
        );

        // 整列キーの先頭要素が `f < 0` なので、上ルート（F ≧ 0）が前半に来る。
        let split = entries.partition_point(|e| e.0 >= 0);
        let (upper_entries, lower_entries) = entries.split_at(split);

        let leaf_level = leaf_level_for::<V>(z);

        let (lower_root, upper_root) = join_at!(
            super::parallel::PAR_SLICE_CUTOFF,
            entries.len(),
            || build_slice(lower_entries, 0, z, leaf_level, &empty_leaf),
            || build_slice(upper_entries, 0, z, leaf_level, &empty_leaf)
        );

        Self {
            lower_root,
            upper_root,
            empty_leaf,
            shard: None,
            shard_path: None,
        }
    }
}

/// `entries`（同じ接頭辞を共有する連続区間）から、レベル `level` 以下の部分木を組む。
fn build_slice<V: SafeValue>(
    entries: &[SingleEntry<V>],
    level: u8,
    z: u8,
    leaf_level: u8,
    empty_leaf: &SharedNode<Node<V>>,
) -> SharedNode<Node<V>> {
    if entries.is_empty() {
        return empty_leaf.clone();
    }

    // 消化済みの軸（そのズームまで分割し終えた F/X/Y、および全時間の T）は
    // ノードを作らず読み飛ばす。insert 側の「無関係な次元をスキップ」と同じ扱い。
    let mut level = level;
    let split_level = loop {
        if level >= leaf_level {
            debug_assert_eq!(entries.len(), 1, "全軸消化点に複数件ある（位置が重複）");
            return SharedNode::new(Node::Leaf {
                value: Some(entries[0].3.clone()),
            });
        }
        // T は全時間（ズーム 0）なので常に消化済み。
        if !matches!(Node::<V>::axis(level), Axis::T) && Node::<V>::depth(level) < z {
            break level;
        }
        level += 1;
    };

    let axis = Node::<V>::axis(split_level);
    let shift = z - 1 - Node::<V>::depth(split_level);
    let split = entries.partition_point(|e| axis_bit(e, axis, shift) == 0);
    let (lo, hi) = entries.split_at(split);

    let (new_lower, new_upper) = join_at!(
        super::parallel::PAR_SLICE_CUTOFF,
        entries.len(),
        || build_slice(lo, split_level + 1, z, leaf_level, empty_leaf),
        || build_slice(hi, split_level + 1, z, leaf_level, empty_leaf)
    );

    Node::mk(split_level, new_lower, new_upper, empty_leaf)
}

/// `axis` 方向のインデックスから、深度に対応する 1 ビットを取り出す。
/// [`Node::forking`] と同じ式（F は 2 の補数のビットをそのまま使う）。
#[inline]
fn axis_bit<V>(entry: &SingleEntry<V>, axis: Axis, shift: u8) -> u32 {
    let index = match axis {
        Axis::F => entry.0 as u32,
        Axis::X => entry.1,
        Axis::Y => entry.2,
        Axis::T => unreachable!("T は消化済みとして読み飛ばしている"),
    };
    (index >> shift) & 1
}

/// 葉 `id` をズーム `z` の一様な [`SingleEntry`] へ細分して `out` へ追記する。
///
/// 葉は軸ごとにズームが違いうる（異方圧縮）ので、各軸を `z` まで割った直方体になる。
pub(crate) fn expand_leaf<V: Clone>(
    id: &crate::FlexId,
    z: u8,
    value: &V,
    out: &mut Vec<SingleEntry<V>>,
) {
    let nf = 1i32 << (z - id.f_zoomlevel());
    let nx = 1u32 << (z - id.x_zoomlevel());
    let ny = 1u32 << (z - id.y_zoomlevel());
    let f0 = id.f_index() * nf;
    let x0 = id.x_index() * nx;
    let y0 = id.y_index() * ny;
    for df in 0..nf {
        for dx in 0..nx {
            for dy in 0..ny {
                out.push((f0 + df, x0 + dx, y0 + dy, value.clone()));
            }
        }
    }
}

/// 木の降下順へ整列し、同じ位置のものを `resolve` で畳んで一意にする。
///
/// `resolve` は整列後の並び（＝空間順）で左から畳み込まれる。畳み込み順に結果が
/// 依存しない可換な [`MergePolicy`](crate::merge_policy::MergePolicy) でのみ使うこと。
pub(crate) fn sort_and_dedup<V, R>(entries: &mut Vec<SingleEntry<V>>, resolve: &R)
where
    V: SafeValue,
    R: Fn(&V, &V) -> V + MaybeSync + ?Sized,
{
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        if entries.len() >= super::parallel::PAR_SLICE_CUTOFF {
            entries.par_sort_unstable_by_key(sort_key);
        } else {
            entries.sort_unstable_by_key(sort_key);
        }
    }
    #[cfg(not(feature = "rayon"))]
    entries.sort_unstable_by_key(sort_key);

    // `dedup_by` のクロージャには (後ろの要素, 残っている前の要素) の順で渡される。
    // true を返すと後ろの要素が捨てられるので、値は前の要素へ畳み込む。
    entries.dedup_by(|later, kept| {
        if kept.0 == later.0 && kept.1 == later.1 && kept.2 == later.2 {
            kept.3 = resolve(&kept.3, &later.3);
            true
        } else {
            false
        }
    });
}

#[cfg(test)]
mod test;
