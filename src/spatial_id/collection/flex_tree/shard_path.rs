//! シャード木の位置（[`ShardPath`]）と、その辞書順が木の構造と一致する KVS キー。
//!
//! # なぜ領域 [`FlexId`](crate::FlexId) をそのままキーにできないのか
//!
//! [`FlexId::encode`](crate::FlexId::encode) のバイト並びは
//! `[zf][zx][zy][f][x][y][tz][t]` で、**ズームレベルが先頭**に
//! ある。したがって辞書順は「ズーム順」であって空間の入れ子順ではなく、親領域のキーが
//! 子領域のキーの接頭辞にならない。KVS のプレフィックススキャンで「ある領域の配下の
//! シャードを全部取る」ことができない。
//!
//! # なぜ領域から分割パスを復元できないのか
//!
//! [`split_shard`](crate::SpatialIdSet::split_shard) が選ぶ軸は
//! `region_split_axis` が決めるが、その規則は木が時間方向の構造を持つかどうかで
//! F→X→Y の3巡回と F→X→Y→T の4巡回に切り替わる。同じ木でも、時間を持たない状態で
//! 何段か割ってから時間を持つデータが入ると、両方の規則が混ざった経路になる。
//! 領域の `(zf, zx, zy, zt)` からは、どちらの規則で何段目に割られたのかを一意に
//! 決められない。**分割パスは復元するものではなく記録するもの**である。
//!
//! # キーの形式
//!
//! ```text
//! byte 0    : 半球（0 = 下半球 f < 0 / 1 = 上半球 f >= 0）
//! byte 1..N : 分割1段につき1バイト（0 = Lower / 1 = Upper）
//! ```
//!
//! この形は次の3つを同時に満たす。
//!
//! - **親のキーは子のキーの接頭辞**（1段につき必ず1バイト伸びるため）
//! - **兄弟は Lower → Upper の順**（`0 < 1`）
//! - **半球が違えば接頭辞を共有しない**（先頭バイトが違う）
//!
//! 1段1バイトは詰めれば1ビットにできるが、ビット詰めするとバイト境界に揃わない長さで
//! 接頭辞性が壊れる。シャードの段数はたかだか数十なのでキー長を優先しない。

use alloc::vec::Vec;

use crate::Side;

/// シャード木の中での位置。[`ShardPath::key`] で KVS キーへ符号化する。
///
/// [`split_shard`](crate::SpatialIdSet::split_shard) が親から子へ1段ずつ伸ばす。
/// 集合演算（union / intersection / difference）の結果はシャード木のノードとは限らないため、
/// 両辺のパスが完全に一致する場合を除いて失われる。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShardPath {
    /// KVS キーそのもの。先頭が半球、以降が1段1バイトの側。
    ///
    /// 半球と分割列を別フィールドに分けると、キーを組み立て直す・突き合わせるたびに
    /// 2つの表現を行き来することになる。キーの並び自体が木の位置を表す設計なので、
    /// キーを唯一の表現にしておくほうが `key` / `contains` が素直になる。
    ///
    /// 不変条件: 非空、かつ全バイトが 0 か 1。
    key: Vec<u8>,
}

impl ShardPath {
    /// 半球まるごと（分割0段）を表すパス。
    ///
    /// `upper_hemisphere` が真なら [`FlexId::UPPER_MAX`](crate::FlexId::UPPER_MAX)、
    /// 偽なら [`FlexId::LOWER_MAX`](crate::FlexId::LOWER_MAX) に対応する。
    pub fn root(upper_hemisphere: bool) -> Self {
        Self {
            key: alloc::vec![u8::from(upper_hemisphere)],
        }
    }

    /// このパスの子（`side` 側へ1段降りた位置）。
    pub fn child(&self, side: Side) -> Self {
        let mut key = Vec::with_capacity(self.key.len() + 1);
        key.extend_from_slice(&self.key);
        key.push(side as u8);
        Self { key }
    }

    /// 上半球（`f >= 0`）側か。
    pub fn is_upper_hemisphere(&self) -> bool {
        self.key[0] == 1
    }

    /// 半球から何段割ったか。
    pub fn depth(&self) -> usize {
        self.key.len() - 1
    }

    /// KVS キー。辞書順がシャード木の前置順（親が先、兄弟は Lower → Upper）と一致する。
    ///
    /// ```
    /// # use kasane_logic::{ShardPath, Side};
    /// let root = ShardPath::root(true);
    /// let lower = root.child(Side::Lower);
    /// let upper = root.child(Side::Upper);
    ///
    /// // 親のキーは子のキーの接頭辞
    /// assert!(lower.key().starts_with(root.key()));
    /// // 兄弟は Lower が先
    /// assert!(lower.key() < upper.key());
    /// ```
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// [`key`](Self::key) が返したバイト列からパスを復元する。
    ///
    /// 空、または 0 / 1 以外のバイトを含む場合は [`None`]。
    pub fn from_key(key: &[u8]) -> Option<Self> {
        if key.is_empty() || key.iter().any(|b| *b > 1) {
            return None;
        }
        Some(Self { key: key.to_vec() })
    }

    /// `self` が `other` と同じか、`other` を含む祖先か。
    ///
    /// キーの接頭辞関係と一致する（[`key`](Self::key) の性質）。半球バイトが先頭にあるので、
    /// 別半球同士は接頭辞にならない。
    pub fn contains(&self, other: &Self) -> bool {
        other.key.starts_with(&self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// 親のキーが子のキーの接頭辞であること（プレフィックススキャンの前提）。
    #[test]
    fn parent_key_is_prefix_of_descendant_keys() {
        let root = ShardPath::root(true);
        let a = root.child(Side::Upper);
        let b = a.child(Side::Lower);
        let c = b.child(Side::Upper);

        for descendant in [&a, &b, &c] {
            assert!(
                descendant.key().starts_with(root.key()),
                "{descendant:?} のキーが root のキーで始まっていない"
            );
            assert!(root.contains(descendant));
        }
        assert!(a.contains(&c));
        assert!(!c.contains(&a));
    }

    /// 兄弟が Lower → Upper の順に並ぶこと。
    #[test]
    fn siblings_sort_lower_before_upper() {
        let root = ShardPath::root(false);
        assert!(root.child(Side::Lower).key() < root.child(Side::Upper).key());
    }

    /// 半球が違えば接頭辞を共有しないこと。
    #[test]
    fn hemispheres_do_not_share_a_prefix() {
        let lower = ShardPath::root(false);
        let upper = ShardPath::root(true);
        assert!(!lower.contains(&upper));
        assert!(!upper.contains(&lower));
        assert!(!upper.key().starts_with(lower.key()));
    }

    /// 深さ順に並べたキーが、木の前置順（親が子より先）になること。
    #[test]
    fn keys_sort_in_preorder() {
        let root = ShardPath::root(true);
        let l = root.child(Side::Lower);
        let u = root.child(Side::Upper);
        let ll = l.child(Side::Lower);
        let lu = l.child(Side::Upper);

        let mut keys = vec![u.key(), lu.key(), root.key(), ll.key(), l.key()];
        keys.sort();
        assert_eq!(
            keys,
            vec![root.key(), l.key(), ll.key(), lu.key(), u.key()],
            "辞書順が前置順になっていない"
        );
    }

    /// キーを往復できること。
    #[test]
    fn key_round_trips() {
        let path = ShardPath::root(false)
            .child(Side::Upper)
            .child(Side::Upper)
            .child(Side::Lower);
        assert_eq!(ShardPath::from_key(path.key()), Some(path));
    }

    /// 不正なキーを拒否すること。
    #[test]
    fn malformed_keys_are_rejected() {
        assert_eq!(ShardPath::from_key(&[]), None);
        assert_eq!(ShardPath::from_key(&[2]), None);
        assert_eq!(ShardPath::from_key(&[1, 0, 7]), None);
    }
}
