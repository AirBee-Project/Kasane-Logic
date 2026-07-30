//! [`SpatialIdMap`] のバイト列表現（フラットアリーナ）と、その write 側変換。
//!
//! インメモリの作業構造（`Arc` ベースの `FlexTreeCore`）はそのままに、保存時のみ
//! 木を `Vec<ArenaNode>`（子ノードは配列インデックス参照）へ平坦化して rkyv で直列化する。
//! 値は `dictionary: Vec<Vec<u8>>` に集約（重複排除）し、葉は dictionary のインデックス（+1、0 は空）を持つ。
//!
//! - 書き込み（[`SpatialIdMap::to_bytes`]）はここで `Arc` 木からこのアリーナを組み立てる。
//! - 読み込み（[`SpatialIdMap::from_bytes`]）はここで archived 表現から `Arc` 木を再構築する。
//! - バイト列を `Arc` 木に戻さず**直接読む**ゼロコピーの読み取り口は [`super::archived`] を参照。

use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use rkyv::{Archive, Deserialize, Serialize};

use super::SpatialIdMap;
use crate::spatial_id::collection::flex_tree::core::node::Node;
use crate::spatial_id::collection::flex_tree::core::ptr::SharedNode;
use crate::{Error, FlexId};

/// 形式バージョンを検証する。
pub(crate) fn check_version(found: u16) -> Result<(), Error> {
    if found == FORMAT_VERSION {
        Ok(())
    } else {
        Err(Error::UnsupportedFormatVersion {
            expected: FORMAT_VERSION,
            found,
        })
    }
}

/// 空を表す葉インデックス値（`ArenaNode::Leaf { value }` の `value == 0`）。
pub(crate) const EMPTY_LEAF: u32 = 0;

/// 永続化バイト列の形式バージョン。
///
/// `MapArena` / `ArenaNode` のフィールド構成を変更したら**必ず上げる**。
/// 上げ忘れると、古いバイト列が拒否されずに誤って読まれる（`access_unchecked` は
/// レイアウトを検証しないため）。
///
/// なお **`FlexTreeCore` や `Node` にフィールドを足しただけならバージョンは変わらない**。
/// ディスク形式は `MapArena` が単独で決めており、書き込み側は `Node` の
/// キャッシュ用フィールドを読まないため。この不変性は golden テストが担保している。
///
/// # 履歴
///
/// - `1`: 時間軸の導入前。木は F/X/Y の3軸で、`FlexId` は空間3軸ぶんのフィールドだけを持つ。
/// - `2`: 時間軸（T）を加えた4軸（`temporal_id` feature 有効時）。
/// - `3`: 3軸のまま `FlexId` が時間フィールドを持つ形（`temporal_id` feature 無効時）。
///
/// **バージョンは feature によって変わる。** 理由は2つ。
///
/// 1. ノードが保持する `level` から軸を求める式が、3軸なら `level % 3`、4軸なら `level % 4`。
///    `level` はバイト列にそのまま入っているので、取り違えると軸の対応がずれ、
///    エラーにならないまま別の空間 ID として読めてしまう。
/// 2. `shard` として保存される `FlexId` のフィールド構成が `1` とは異なる
///    （時間軸を4番目の軸としてインラインに持つようになったため、feature の有無に関わらず
///    `t_zoomlevel` / `t_index` が並ぶ）。
///
/// バージョンを分けることで、レイアウトの違う書き出しは読み込み時に必ず弾かれる。
#[cfg(feature = "temporal_id")]
pub const FORMAT_VERSION: u16 = 2;
#[cfg(not(feature = "temporal_id"))]
pub const FORMAT_VERSION: u16 = 3;

/// 平坦化された [`SpatialIdMap`] 1枚（1シャード）の書き込み用スキーマ。
///
/// この構造体自体は `to_bytes` の実行中に一度だけ組み立てられる使い捨ての値で、
/// 読み込み側（[`SpatialIdMap::from_bytes`] / [`super::archived::ArchivedSpatialIdMap`]）は
/// これを経由せず archived 表現（[`ArchivedMapArena`]）を直接読む。
#[derive(Archive, Serialize, Deserialize, Debug)]
pub(crate) struct MapArena {
    /// 形式バージョン（[`FORMAT_VERSION`]）。読み出し時に検証する。
    ///
    /// フィールドは `pub(crate)`：archived 表現（`ArchivedMapArena`）の対応フィールドを
    /// 読み取り専用ビュー（[`super::archived`]）から直接読むため。
    pub(crate) version: u16,
    /// 下半分（f < 0）ルートの `nodes` インデックス。
    pub(crate) lower_root: u32,
    /// 上半分（f >= 0）ルートの `nodes` インデックス。
    pub(crate) upper_root: u32,
    /// このマップが閉じているシャード領域（挿入クリップ用）。
    pub(crate) shard: Option<FlexId>,
    /// 後行順（子が親より前）で並んだノード配列。
    pub(crate) nodes: Vec<ArenaNode>,
    /// 値の辞書。葉の `value`（>0）から `value - 1` で参照する。
    pub(crate) dictionary: Vec<Vec<u8>>,
}

/// 平坦化されたノード。子は `nodes` 配列のインデックス。
#[derive(Archive, Serialize, Deserialize, Debug)]
pub(crate) enum ArenaNode {
    Branch {
        level: u8,
        lower: u32,
        upper: u32,
    },
    /// `value == 0` は空、`value > 0` は `dictionary[value - 1]`。
    Leaf {
        value: u32,
    },
}

impl SpatialIdMap<Vec<u8>> {
    /// この [`SpatialIdMap`] をフラットアリーナ形式のバイト列へ直列化する。
    ///
    /// 先頭に [`FORMAT_VERSION`] を埋め込むので、[`from_bytes`](Self::from_bytes) /
    /// [`ArchivedSpatialIdMap::access`](super::archived::ArchivedSpatialIdMap::access) が形式違いを検出できる。
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut nodes: Vec<ArenaNode> = Vec::new();
        let mut dictionary: Vec<Vec<u8>> = Vec::new();
        let mut value_to_idx: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
        let mut empty_idx: Option<u32> = None;

        let lower_root = build_node(
            &self.inner.lower_root,
            &mut nodes,
            &mut dictionary,
            &mut value_to_idx,
            &mut empty_idx,
        );
        let upper_root = build_node(
            &self.inner.upper_root,
            &mut nodes,
            &mut dictionary,
            &mut value_to_idx,
            &mut empty_idx,
        );

        let arena = MapArena {
            version: FORMAT_VERSION,
            lower_root,
            upper_root,
            shard: self.inner.shard,
            nodes,
            dictionary,
        };
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(&arena)
            .map_err(|e| Error::Persist(alloc::format!("serialize: {e}")))?
            .to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から作業木（`Arc` ベース）を復元する。
    ///
    /// 形式バージョンが [`FORMAT_VERSION`] と異なる場合は
    /// [`Error::UnsupportedFormatVersion`] を返す（誤読させない）。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedMapArena>(bytes) };
        check_version(archived.version.to_native())?;
        // 所有型 `MapArena` へ復元してから木を組むと、ノード配列と値辞書を
        // まるごと複製することになる。archived 表現を直接読んで組めばその一段が省ける。
        let mut core = FlexTreeCore::<Vec<u8>>::new();
        let empty = core.empty_leaf.clone();
        core.lower_root = rebuild_node(
            archived.lower_root.to_native(),
            &archived.nodes,
            &archived.dictionary,
            &empty,
        );
        core.upper_root = rebuild_node(
            archived.upper_root.to_native(),
            &archived.nodes,
            &archived.dictionary,
            &empty,
        );
        core.shard = archived
            .shard
            .as_ref()
            .map(rkyv::deserialize::<FlexId, rkyv::rancor::Error>)
            .transpose()
            .map_err(|e| Error::Persist(alloc::format!("deserialize shard: {e}")))?;

        Ok(Self { inner: core })
    }
}

/// 作業木の 1 ノードを後行順でアリーナへ書き出し、そのインデックスを返す。
fn build_node(
    node: &SharedNode<Node<Vec<u8>>>,
    nodes: &mut Vec<ArenaNode>,
    dictionary: &mut Vec<Vec<u8>>,
    value_to_idx: &mut BTreeMap<Vec<u8>, u32>,
    empty_idx: &mut Option<u32>,
) -> u32 {
    match &**node {
        Node::Leaf { value: None } => {
            if let Some(i) = *empty_idx {
                i
            } else {
                let i = nodes.len() as u32;
                nodes.push(ArenaNode::Leaf { value: EMPTY_LEAF });
                *empty_idx = Some(i);
                i
            }
        }
        Node::Leaf { value: Some(v) } => {
            let dict_idx = match value_to_idx.get(v) {
                Some(idx) => *idx,
                None => {
                    let idx = dictionary.len() as u32;
                    dictionary.push(v.clone());
                    value_to_idx.insert(v.clone(), idx);
                    idx
                }
            };
            let i = nodes.len() as u32;
            nodes.push(ArenaNode::Leaf {
                value: dict_idx + 1,
            });
            i
        }
        Node::Branch {
            level,
            lower_child,
            upper_child,
            ..
        } => {
            let lower = build_node(lower_child, nodes, dictionary, value_to_idx, empty_idx);
            let upper = build_node(upper_child, nodes, dictionary, value_to_idx, empty_idx);
            let i = nodes.len() as u32;
            nodes.push(ArenaNode::Branch {
                level: *level,
                lower,
                upper,
            });
            i
        }
    }
}

/// archived 表現の `idx` を根とする部分木を `Arc` 木へ復元する。
///
/// 所有型 `MapArena` を経由しないので、ノード配列と値辞書の複製が丸ごと省ける。
/// 保存していない導出値（`leaf_count` / `max_zoom` / `split_mask`）はここで畳み直す。
fn rebuild_node(
    idx: u32,
    nodes: &rkyv::vec::ArchivedVec<ArchivedArenaNode>,
    dictionary: &rkyv::vec::ArchivedVec<rkyv::vec::ArchivedVec<u8>>,
    empty: &SharedNode<Node<Vec<u8>>>,
) -> SharedNode<Node<Vec<u8>>> {
    match &nodes[idx as usize] {
        ArchivedArenaNode::Leaf { value } if value.to_native() == EMPTY_LEAF => empty.clone(),
        ArchivedArenaNode::Leaf { value } => SharedNode::new(Node::Leaf {
            value: Some(dictionary[(value.to_native() - 1) as usize].to_vec()),
        }),
        ArchivedArenaNode::Branch {
            level,
            lower,
            upper,
        } => {
            let level = *level;
            let lower_child = rebuild_node(lower.to_native(), nodes, dictionary, empty);
            let upper_child = rebuild_node(upper.to_native(), nodes, dictionary, empty);
            let leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
            let max_zoom = Node::<Vec<u8>>::fold_max_zoom(level, &lower_child, &upper_child);
            let split_mask = Node::<Vec<u8>>::fold_split_mask(level, &lower_child, &upper_child);
            SharedNode::new(Node::Branch {
                level,
                leaf_count,
                max_zoom,
                split_mask,
                lower_child,
                upper_child,
            })
        }
    }
}

#[cfg(test)]
mod version_tests {
    //! 形式バージョンの検証。
    //!
    //! `MapArena` の私有フィールドへ触る必要があるので、モジュール内に置く。

    use super::{ArchivedMapArena, ArenaNode, FORMAT_VERSION, MapArena};
    use crate::spatial_id::collection::flex_tree::map::archived::ArchivedSpatialIdMap;
    use crate::{Error, SingleId, SpatialIdMap};
    use alloc::vec::Vec;

    fn sample_bytes() -> Vec<u8> {
        let mut m = SpatialIdMap::new();
        m.insert(SingleId::new(4, 0, 1, 1).unwrap(), alloc::vec![7u8]);
        m.to_bytes().unwrap()
    }

    /// 正しく書かれたバイト列には現行バージョンが入っている。
    #[test]
    fn written_bytes_carry_the_current_version() {
        let bytes = sample_bytes();
        let arch = unsafe { ArchivedSpatialIdMap::access(&bytes) }.unwrap();
        assert_eq!(arch.format_version(), FORMAT_VERSION);
    }

    /// 別バージョンのバイト列を作り、両方の読み出し口が拒否することを確認する。
    ///
    /// これが効いていないと、将来 `MapArena` を変更したとき古いバイト列が
    /// 「拒否されず誤って読まれる」ことになる。
    #[test]
    fn foreign_version_is_rejected() {
        let foreign = MapArena {
            version: FORMAT_VERSION.wrapping_add(1),
            lower_root: 0,
            upper_root: 0,
            shard: None,
            nodes: alloc::vec![ArenaNode::Leaf { value: 0 }],
            dictionary: Vec::new(),
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&foreign)
            .unwrap()
            .to_vec();

        // 念のため、書いたバージョンが読めていることを確認
        let arch = unsafe { rkyv::access_unchecked::<ArchivedMapArena>(&bytes) };
        assert_eq!(arch.version.to_native(), FORMAT_VERSION.wrapping_add(1));

        // ゼロコピー読み出し口
        let Err(err) = (unsafe { ArchivedSpatialIdMap::access(&bytes) }) else {
            panic!("ArchivedSpatialIdMap::access がバージョン違いを通した");
        };
        assert_eq!(
            err,
            Error::UnsupportedFormatVersion {
                expected: FORMAT_VERSION,
                found: FORMAT_VERSION.wrapping_add(1),
            }
        );

        // 全復元の読み出し口
        let err = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes) }.unwrap_err();
        assert!(
            matches!(err, Error::UnsupportedFormatVersion { .. }),
            "from_bytes がバージョン違いを通した: {err:?}"
        );
    }
}
