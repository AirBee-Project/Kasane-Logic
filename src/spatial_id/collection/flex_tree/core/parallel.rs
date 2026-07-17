//! [`FlexTreeCore`] の並列バルク構築。
//!
//! 逐次 [`insert`](FlexTreeCore::insert) は 1 件ずつ木を降りるため、大量ロードでは
//! シングルコアに律速される。ここでは入力をチャンクに分けて各チャンクを別スレッドで
//! 部分木化し、[`union`](FlexTreeCore::union) のツリー簡約で畳み込む。
//! union の実体 [`Node::merge`](super::node::Node::merge) 自体も内部で `rayon::join`
//! するため、簡約段も並列化される。
//!
//! FXY-正規形が一意なので、結果は逐次 `insert` と構造的に一致する（union の可換性・
//! 結合性による）。重なりのある値は `union` と同じ左優先で解決されるため、値の衝突が
//! ある `V` では挿入順に依存しうる。集合（`V = ()`）では順序不問で一意に定まる。

use alloc::vec::Vec;
use rayon::prelude::*;

use super::FlexTreeCore;
use super::ptr::SafeValue;
use crate::FlexId;

/// 1 チャンクの最小サイズ。これ未満に刻むと union 簡約の回数がかさんで逆効果になる。
const MIN_PAR_CHUNK: usize = 512;

/// 空間ソートキーの1軸あたりビット数（F/X/Y の3軸で 3×20 = 60bit、u64 に収まる）。
const SORT_KEY_BITS: u32 = 20;

/// 軸のインデックスを、ズームに依らず先頭ビット揃え（MSB 揃え）で `bits` 幅へ正規化する。
/// 粗い（浅い）セルは上位ビット側に、細かいセルは下位ビットまで伸びる。
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

/// [`FlexId`] の空間位置を単調なキーへ写す。F→X→Y の順にビットを詰め、木の降下順
/// （レベル 0=F, 1=X, 2=Y, …）と整合する粗いクラスタリングを与える。厳密な木順ではなく
/// 「空間的に近い ID を連続させる」ことが目的で、これによりチャンクが空間的に局所化し、
/// チャンク木同士の [`union`] が互いにほぼ素になって簡約が軽くなる。
#[inline]
fn spatial_sort_key(id: &FlexId) -> u64 {
    const B: u32 = SORT_KEY_BITS;
    // F は符号付き。木は最初に符号でルートを分けるため、符号ビットを最上位に置く。
    let f_biased = (id.f_index() as i64 + (1i64 << 30)) as u64;
    let fa = axis_aligned(f_biased, id.f_zoomlevel().saturating_add(1), B);
    let xa = axis_aligned(id.x_index() as u64, id.x_zoomlevel(), B);
    let ya = axis_aligned(id.y_index() as u64, id.y_zoomlevel(), B);
    (fa << (2 * B)) | (xa << B) | ya
}

impl<V> FlexTreeCore<V>
where
    V: SafeValue,
{
    /// `(FlexId, V)` 列から木を並列に構築する。逐次 `insert` と同じ正規形を返す。
    ///
    /// 手順は次の3段でいずれも並列化される:
    /// 1. 空間ソートキーで並べ替え、空間的に近い ID を連続させる（`par_sort`）。
    /// 2. 連続チャンクごとに部分木を構築する（各チャンクは空間的に局所）。
    /// 3. 部分木を `union` のツリー簡約で畳み込む。局所化により隣接チャンクはほぼ素で、
    ///    union は分岐の浅い段で枝刈りされて軽い。
    ///
    /// 入力規模がスレッド数に対して小さい場合はチャンクが 1 個に畳まれ、実質逐次で動く。
    pub fn par_build_vec(mut items: Vec<(FlexId, V)>) -> Self {
        if items.is_empty() {
            return Self::new();
        }

        // 空間局所化。union 簡約のコストを大きく左右する。
        items.par_sort_unstable_by_key(|(id, _)| spatial_sort_key(id));

        let threads = rayon::current_num_threads().max(1);
        // 1 スレッドあたり数チャンクに割って負荷を均しつつ、下限で刻み過ぎを防ぐ。
        let chunk_size = (items.len() / (threads * 4)).max(MIN_PAR_CHUNK);

        items
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut core = Self::new();
                for (id, value) in chunk {
                    core.insert(id.clone(), value.clone());
                }
                core
            })
            .reduce(Self::new, |a, b| a.union(&b))
    }
}
