//! [`SpatialIdMap`] の永続化（フラットアリーナ直列化）と ZeroCopy 読み出し。
//!
//! インメモリの作業構造（`Arc` ベースの [`FlexTree`]）はそのままに、保存時のみ
//! 木を `Vec<PersistedNode>`（子ノードは配列インデックス参照）へ平坦化して rkyv で直列化する。
//! 値は `dictionary: Vec<Vec<u8>>` に集約（重複排除）し、葉は dictionary のインデックス（+1、0 は空）を持つ。
//!
//! - 書き込み（[`SpatialIdMap::to_bytes`] / [`SpatialIdMap::from_bytes`]）は `Arc` 木との相互変換。
//! - 読み出し（[`ArchivedMap`]）は archived バイト列を**直接走査**し、`Arc` 木を再構築せず
//!   `&[u8]` を ZeroCopy で返す。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use rkyv::{Archive, Deserialize, Serialize};

use super::SpatialIdMap;
use crate::spatial_id::collection::flex_tree::node::Node;
use crate::spatial_id::collection::flex_tree::ptr::SharedNode;
use crate::spatial_id::collection::flex_tree::split_child_id;
use crate::{FlexId, FlexTree, Side, SpatialId, TemporalId, TemporalMap};

/// 平坦化された [`SpatialIdMap`] 1枚（1シャード）。
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct PersistedMap {
    /// 下半分（f < 0）ルートの `nodes` インデックス。
    lower_root: u32,
    /// 上半分（f >= 0）ルートの `nodes` インデックス。
    upper_root: u32,
    /// このマップが閉じているシャード領域（挿入クリップ用）。
    shard: Option<FlexId>,
    /// 後行順（子が親より前）で並んだノード配列。
    nodes: Vec<PersistedNode>,
    /// 値の辞書。葉の `value`（>0）から `value - 1` で参照する。
    dictionary: Vec<Vec<u8>>,
}

/// 平坦化されたノード。子は `nodes` 配列のインデックス。
#[derive(Archive, Serialize, Deserialize, Debug)]
pub enum PersistedNode {
    Branch {
        level: u8,
        lower: u32,
        upper: u32,
    },
    /// 葉の時間セグメント列 `(start, end, dictionary index)`。空葉は `segments` が空。
    Leaf {
        segments: Vec<(u64, u64, u32)>,
    },
}

impl SpatialIdMap<Vec<u8>> {
    /// この [`SpatialIdMap`] をフラットアリーナ形式の rkyv バイト列へ直列化する。
    pub fn to_bytes(&self) -> Result<Vec<u8>, rkyv::rancor::Error> {
        let mut nodes: Vec<PersistedNode> = Vec::new();
        let mut dictionary: Vec<Vec<u8>> = Vec::new();
        let mut value_to_idx: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
        let mut empty_idx: Option<u32> = None;

        let lower_root = build_node(
            &self.inner.inner.lower_root,
            &mut nodes,
            &mut dictionary,
            &mut value_to_idx,
            &mut empty_idx,
        );
        let upper_root = build_node(
            &self.inner.inner.upper_root,
            &mut nodes,
            &mut dictionary,
            &mut value_to_idx,
            &mut empty_idx,
        );

        let persisted = PersistedMap {
            lower_root,
            upper_root,
            shard: self.inner.shard().cloned(),
            nodes,
            dictionary,
        };
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(&persisted)?.to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から作業木（`Arc` ベース）を復元する。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedPersistedMap>(bytes) };
        let persisted: PersistedMap =
            rkyv::deserialize::<PersistedMap, rkyv::rancor::Error>(archived)?;

        let mut core = FlexTree::<TemporalMap<Vec<u8>>>::new();
        let empty = core.empty_leaf.clone();
        core.lower_root = rebuild_node(
            persisted.lower_root,
            &persisted.nodes,
            &persisted.dictionary,
            &empty,
        );
        core.upper_root = rebuild_node(
            persisted.upper_root,
            &persisted.nodes,
            &persisted.dictionary,
            &empty,
        );
        core.shard = persisted.shard;

        Ok(Self {
            inner: crate::spatial_id::collection::temporal::SpatioTemporalCore { inner: core },
        })
    }
}

/// 作業木の 1 ノードを後行順でアリーナへ書き出し、そのインデックスを返す。
fn build_node(
    node: &SharedNode<Node<TemporalMap<Vec<u8>>>>,
    nodes: &mut Vec<PersistedNode>,
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
                nodes.push(PersistedNode::Leaf {
                    segments: Vec::new(),
                });
                *empty_idx = Some(i);
                i
            }
        }
        Node::Leaf { value: Some(tmap) } => {
            let segments = tmap
                .segments_ref()
                .into_iter()
                .map(|(start, end, v)| {
                    let dict_idx = match value_to_idx.get(v) {
                        Some(idx) => *idx,
                        None => {
                            let idx = dictionary.len() as u32;
                            dictionary.push(v.clone());
                            value_to_idx.insert(v.clone(), idx);
                            idx
                        }
                    };
                    (start, end, dict_idx)
                })
                .collect();
            let i = nodes.len() as u32;
            nodes.push(PersistedNode::Leaf { segments });
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
            nodes.push(PersistedNode::Branch {
                level: *level,
                lower,
                upper,
            });
            i
        }
    }
}

/// アリーナから作業木（`Arc` ベース）を再帰的に復元する。
fn rebuild_node(
    idx: u32,
    nodes: &[PersistedNode],
    dictionary: &[Vec<u8>],
    empty: &SharedNode<Node<TemporalMap<Vec<u8>>>>,
) -> SharedNode<Node<TemporalMap<Vec<u8>>>> {
    match &nodes[idx as usize] {
        PersistedNode::Leaf { segments } if segments.is_empty() => empty.clone(),
        PersistedNode::Leaf { segments } => SharedNode::new(Node::Leaf {
            value: Some(TemporalMap::from_raw_segments(
                segments
                    .iter()
                    .map(|&(start, end, dict_idx)| {
                        (start, end, dictionary[dict_idx as usize].clone())
                    })
                    .collect(),
            )),
        }),
        PersistedNode::Branch {
            level,
            lower,
            upper,
        } => {
            let lower_child = rebuild_node(*lower, nodes, dictionary, empty);
            let upper_child = rebuild_node(*upper, nodes, dictionary, empty);
            let leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
            let max_zoom =
                Node::<TemporalMap<Vec<u8>>>::fold_max_zoom(*level, &lower_child, &upper_child);
            SharedNode::new(Node::Branch {
                level: *level,
                leaf_count,
                max_zoom,
                lower_child,
                upper_child,
            })
        }
    }
}

/// archived バイト列を直接走査する ZeroCopy リーダ。`Arc` 木を再構築しない。
pub struct ArchivedMap<'a> {
    inner: &'a ArchivedPersistedMap,
}

impl<'a> ArchivedMap<'a> {
    /// archived バイト列上にリーダを開く。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn access(bytes: &'a [u8]) -> Self {
        Self {
            inner: unsafe { rkyv::access_unchecked::<ArchivedPersistedMap>(bytes) },
        }
    }

    /// `target` と重なる (FlexId, 値) を、`target` で切り取って返す（インメモリ `get` と同義）。
    pub fn get(&self, target: &FlexId) -> Vec<(FlexId, &'a [u8])> {
        let mut out = Vec::new();
        let mut stack = alloc::vec![
            (self.inner.upper_root.to_native(), FlexId::UPPER_MAX),
            (self.inner.lower_root.to_native(), FlexId::LOWER_MAX),
        ];
        while let Some((idx, current_id)) = stack.pop() {
            if current_id.intersection(target).is_none() {
                continue;
            }
            match &self.inner.nodes[idx as usize] {
                ArchivedPersistedNode::Branch {
                    level,
                    lower,
                    upper,
                } => {
                    let axis = Node::<TemporalMap<Vec<u8>>>::axis(*level);
                    stack.push((
                        upper.to_native(),
                        split_child_id(&current_id, axis, Side::Upper),
                    ));
                    stack.push((
                        lower.to_native(),
                        split_child_id(&current_id, axis, Side::Lower),
                    ));
                }
                ArchivedPersistedNode::Leaf { segments } => {
                    if let Some(clipped) = current_id.intersection(target) {
                        // 時間もクエリ（clipped.temporal()）で切り取って返す。
                        let (w0, w1) = (
                            clipped.temporal().start_unixtime(),
                            clipped.temporal().end_unixtime_exclusive(),
                        );
                        for seg in segments.iter() {
                            let (s, e, dict_idx) =
                                (seg.0.to_native(), seg.1.to_native(), seg.2.to_native());
                            let cs = s.max(w0);
                            let ce = e.min(w1);
                            if cs < ce {
                                for t in TemporalId::from_range(cs, ce).unwrap() {
                                    out.push((
                                        clipped.with_temporal(t),
                                        self.inner.dictionary[dict_idx as usize].as_slice(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// 保持している全ての (FlexId, 値) を ZeroCopy で列挙する。
    pub fn iter(&self) -> Vec<(FlexId, &'a [u8])> {
        let mut out = Vec::new();
        let mut stack = alloc::vec![
            (self.inner.upper_root.to_native(), FlexId::UPPER_MAX),
            (self.inner.lower_root.to_native(), FlexId::LOWER_MAX),
        ];
        while let Some((idx, current_id)) = stack.pop() {
            match &self.inner.nodes[idx as usize] {
                ArchivedPersistedNode::Branch {
                    level,
                    lower,
                    upper,
                } => {
                    let axis = Node::<TemporalMap<Vec<u8>>>::axis(*level);
                    stack.push((
                        upper.to_native(),
                        split_child_id(&current_id, axis, Side::Upper),
                    ));
                    stack.push((
                        lower.to_native(),
                        split_child_id(&current_id, axis, Side::Lower),
                    ));
                }
                ArchivedPersistedNode::Leaf { segments } => {
                    for seg in segments.iter() {
                        let (s, e, dict_idx) =
                            (seg.0.to_native(), seg.1.to_native(), seg.2.to_native());
                        for t in TemporalId::from_range(s, e).unwrap() {
                            out.push((
                                current_id.with_temporal(t),
                                self.inner.dictionary[dict_idx as usize].as_slice(),
                            ));
                        }
                    }
                }
            }
        }
        out
    }
}
