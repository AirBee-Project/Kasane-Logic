//! 分離可能な単項演算のための、一様ズームの平坦表現 `UniformGrid`。
//!
//! # 動機
//!
//! `shift` / `falloff_linear` は「1 軸方向の平行移動」しかしない分離可能な演算だが、
//! 木の上で実行すると 1 演算ごとに
//!
//! 1. 全葉を `Vec<(FlexId, V)>` へ展開し（falloff なら 2r+1 倍に膨らむ）、
//! 2. 展開した [`FlexId`](crate::FlexId) を 1 個ずつ木へ挿入して中間木を作り直す
//!
//! という往復が入る。実測ではこの 2 の木構築が全体の 9 割を占めていた（ズーム 24 の
//! 実データで 5 万葉 → 306 万葉の falloff 3 段が 9.9 秒、うち最終木の構築だけで 3.5 秒）。
//! しかも展開した 2000 万件はほとんどが重なって消えるので、木は「重なりを解決するための
//! 高価なハッシュテーブル」として使われていただけだった。
//!
//! # やっていること
//!
//! 分離可能演算が連なっている間だけ木を捨て、**一様ズーム Z の
//! [`SingleId`](crate::SingleId) の平坦な配列**で持つ。
//!
//! - ズームが揃っているので、**軸方向の移動が単なる整数加算**になる。木のときのように
//!   粗いSegmentが移動のたびに複数の正規Segmentへ砕ける（＝データが増える）ことがない。
//! - 重なりの解決は「同じ位置」の一致判定で済むので、整列と隣接畳み込みで終わる。
//! - falloff は、対象軸を最下位キーにして並べると**レーン（他 2 軸が同じものの列）ごとの
//!   1 次元走査**になる。出力位置ごとに半径内の入力を集める gather 型にすれば、
//!   2r+1 倍の中間データを作らずに出力そのものを整列済みで直接生成できる。
//! - 最後に一度だけ `FlexTreeCore::from_uniform_single_ids` でボトムアップに木を組む。
//!
//! # 適用条件
//!
//! 平坦化は葉をズーム Z まで細分するので、粗い葉が多いと件数が爆発する。先に見積もり、
//! 予算を超えるなら従来の木経路へフォールバックする。時間軸で分割された木も
//! （ズームが揃わないので）フォールバックする。

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::RangeInclusive;

use crate::spatial_id::collection::flex_tree::core::bulk::{
    SingleEntry, expand_leaf, sort_and_dedup,
};
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSync;
use crate::spatial_id::collection::flex_tree::core::{FlexTreeCore, SafeValue};
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, SpatialIdError, ZoomLevel};

#[cfg(feature = "rayon")]
use crate::spatial_id::collection::flex_tree::core::parallel::PAR_SLICE_CUTOFF;

#[cfg(test)]
mod test;

/// 分離可能演算が作用する軸。
///
/// 木の [`Axis`](crate::spatial_id::collection::flex_tree::core::node::Axis) と違い、
/// 時間軸を持たない。分離可能演算は空間 3 軸にしか作用しないので、時間の分だけ
/// 到達しない分岐を書かずに済む。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GridAxis {
    F,
    X,
    Y,
}

/// 値の減衰。`(元の値, 演算ズーム単位での距離) -> 減衰後の値`。
///
/// `Send + Sync` を直に書いているのは、クレートの慣習である
/// `MaybeSync` が自動トレイトでなくトレイトオブジェクトの境界に書けないため。
pub(crate) type AttenFn<'a, V> = dyn Fn(&V, u32) -> V + Send + Sync + 'a;
/// 重なった値の解決。可換なポリシーであること。`Send + Sync` の理由は [`AttenFn`] と同じ。
pub(crate) type ResolveFn<'a, V> = dyn Fn(&V, &V) -> V + Send + Sync + 'a;

/// `UniformGrid` 上で直接実行できる演算の記述。
///
/// 演算子側（`ShiftX` など）は
/// `UnaryOperator::grid_op` でこれを返すだけでよく、グリッドの走査を演算子ごとに書かなくて済む。
/// 値型 `V` に依存する部分（減衰・衝突解決）だけをクロージャで受け取る。
///
/// 中身はクエリエンジンの実装詳細なので公開していない。クレート外から新しい演算子を
/// 定義する場合は `grid_op` を実装せず（既定の `None`）、木経路の `run` だけを書く。
pub struct GridOp<V> {
    axis: GridAxis,
    /// 移動量・半径を数えるズームレベル。グリッドのズームはこれ以上でなければならない。
    z: ZoomLevel,
    kind: Kind<V>,
}

enum Kind<V> {
    /// 軸方向の平行移動。`z` のインデックス単位での移動量。
    Shift(i32),
    /// 軸方向への線形減衰つき伝播。
    Falloff {
        radius: u32,
        atten: Box<AttenFn<'static, V>>,
        resolve: Box<ResolveFn<'static, V>>,
    },
}

impl<V> GridOp<V> {
    /// 軸方向の平行移動。
    pub(crate) fn shift(axis: GridAxis, z: ZoomLevel, delta: i32) -> Self {
        Self {
            axis,
            z,
            kind: Kind::Shift(delta),
        }
    }

    /// 軸方向への線形減衰つき伝播。
    pub(crate) fn falloff(
        axis: GridAxis,
        z: ZoomLevel,
        radius: u32,
        atten: Box<AttenFn<'static, V>>,
        resolve: Box<ResolveFn<'static, V>>,
    ) -> Self {
        Self {
            axis,
            z,
            kind: Kind::Falloff {
                radius,
                atten,
                resolve,
            },
        }
    }

    /// この演算が単位とするズームレベル。
    pub(crate) fn zoom(&self) -> ZoomLevel {
        self.z
    }
}

/// 整列順。`None` は不定を表す（[`UniformGrid::order`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Order {
    /// `axis` を最下位キーにした整列。`axis` 方向のレーン走査ができる。
    Lane(GridAxis),
    /// 木の降下順。木を組む直前だけこの順にする。
    Morton,
}

/// 平坦化に使ってよいメモリの絶対上限。
const MAX_BYTES: u64 = 512 * 1024 * 1024;

/// 一様ズーム `z` の [`SingleId`](crate::SingleId) と値の平坦な配列。
///
/// すべて全時間（`t_zoomlevel == 0`）で、空間 3 軸のズームは `z` に揃っている。
/// 位置は重複しない（各演算がこの不変条件を保つ）。
pub(crate) struct UniformGrid<V: SafeValue> {
    z: ZoomLevel,
    entries: Vec<SingleEntry<V>>,
    order: Option<Order>,
}

impl<V: SafeValue> UniformGrid<V> {
    /// 木をズーム `z` の一様な [`SingleEntry`] 列へ展開する。
    ///
    /// 次のいずれかなら [`None`] を返し、呼び出し側は従来の木経路へ落ちる。
    /// - 木が時間軸で分割されている（ズームが揃わない）
    /// - 件数が `budget`（および [`MAX_BYTES`] 相当）を超える
    pub(crate) fn from_tree(tree: &WorkingTree<V>, z: ZoomLevel, budget: u64) -> Option<Self> {
        let core = tree.core();
        if core.has_temporal_split() {
            return None;
        }

        // 先に総件数を見積もる。1 葉あたり 2^(z-zf) × 2^(z-zx) × 2^(z-zy) 件。
        // 上限はバイト数で決めたいので、1 件あたりの実サイズで割って件数に直す。
        let limit = budget.min(MAX_BYTES / core::mem::size_of::<SingleEntry<V>>() as u64);
        let mut total: u64 = 0;
        for (id, _) in core.iter_ref() {
            let bits = (z.get() - id.f_zoomlevel())
                + (z.get() - id.x_zoomlevel())
                + (z.get() - id.y_zoomlevel());
            // 1 葉だけで上限を超えるようならシフトの前に打ち切る。
            if bits >= 63 {
                return None;
            }
            total += 1u64 << bits;
            if total > limit {
                return None;
            }
        }

        let mut entries: Vec<SingleEntry<V>> = Vec::with_capacity(total as usize);
        for (id, value) in core.iter_ref() {
            expand_leaf(&id, z.get(), value, &mut entries);
        }

        Some(Self {
            z,
            entries,
            order: None,
        })
    }

    /// 木を組み直す。
    pub(crate) fn into_tree(mut self) -> WorkingTree<V> {
        self.sort_morton(&|a: &V, _b: &V| a.clone());
        WorkingTree::from_core(FlexTreeCore::from_uniform_single_ids(
            self.z.get(),
            &self.entries,
        ))
    }

    /// 演算を 1 つ適用する。
    pub(crate) fn apply(&mut self, op: &GridOp<V>) -> Result<Applied, Error> {
        match &op.kind {
            Kind::Shift(delta) => self.shift(op.axis, op.z, *delta).map(|()| Applied::Done),
            Kind::Falloff {
                radius,
                atten,
                resolve,
            } => {
                if *radius == 0 {
                    return Ok(Applied::Done);
                }
                Ok(self.falloff(op.axis, op.z, *radius, atten, resolve))
            }
        }
    }

    /// 対象軸を最下位キーにした並びにする。すでにその順ならなにもしない。
    fn sort_lanes(&mut self, axis: GridAxis) {
        if self.order == Some(Order::Lane(axis)) {
            return;
        }
        sort_by_lane(&mut self.entries, axis);
        self.order = Some(Order::Lane(axis));
    }

    /// 木の降下順にし、同じ位置を `resolve` で畳む。すでにその順ならなにもしない。
    fn sort_morton<R>(&mut self, resolve: &R)
    where
        R: Fn(&V, &V) -> V + MaybeSync + ?Sized,
    {
        if self.order == Some(Order::Morton) {
            return;
        }
        sort_and_dedup(&mut self.entries, resolve);
        self.order = Some(Order::Morton);
    }

    /// 対象軸の位置の範囲 `[min, max]`。空なら [`None`]。
    fn axis_span(&self, axis: GridAxis) -> Option<RangeInclusive<i64>> {
        let mut it = self.entries.iter().map(|e| axis_pos(e, axis));
        let first = it.next()?;
        let (min, max) = it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p)));
        Some(min..=max)
    }

    /// 軸方向の平行移動。
    ///
    /// F / Y で範囲外へ出るものがあれば、`FlexId::shift_f` / `shift_y` が `Err` を返して
    /// `map_rebuild` がそれを伝播するのと同じく、演算全体をエラーにする。X は巡回する。
    fn shift(&mut self, axis: GridAxis, op_z: ZoomLevel, delta: i32) -> Result<(), Error> {
        if delta == 0 {
            return Ok(());
        }
        // グリッドのズームは op_z 以上（呼び出し側が保証）。
        let d = delta as i64 * (1i64 << (self.z.get() - op_z.get()));
        let span = 1i64 << self.z.get();

        if axis == GridAxis::X {
            for e in &mut self.entries {
                e.1 = (e.1 as i64 + d).rem_euclid(span) as u32;
            }
            // 巡回で並びが崩れる（位置の一意性は保たれる）。
            self.order = None;
            return Ok(());
        }

        // 移動先が軸の範囲を外れる葉があれば、木経路と同じく演算全体をエラーにする。
        if let Some(moved) = self.axis_span(axis).map(|s| s.start() + d..=s.end() + d) {
            match axis {
                GridAxis::F => {
                    self.z.check_f(*moved.start() as i32)?;
                    self.z.check_f(*moved.end() as i32)?;
                }
                GridAxis::Y => {
                    if *moved.start() < 0 {
                        return Err(SpatialIdError::YOutOfRange {
                            z: self.z.get(),
                            y: 0,
                        }
                        .into());
                    }
                    self.z.check_y(*moved.end() as u32)?;
                }
                GridAxis::X => unreachable!("X は上で返している"),
            }
        }

        for e in &mut self.entries {
            match axis {
                GridAxis::F => e.0 = (e.0 as i64 + d) as i32,
                GridAxis::Y => e.2 = (e.2 as i64 + d) as u32,
                GridAxis::X => unreachable!("X は上で返している"),
            }
        }

        // 一様な平行移動は、その軸を単調に写すだけなのでレーン順（軸ごとの辞書式）は
        // 保たれる。一方 Morton 順は軸のビットを織り交ぜた順なので、定数を足すと上位の
        // 分岐が入れ替わりうる。保たれないほうは捨てる。
        if self.order == Some(Order::Morton) {
            self.order = None;
        }
        Ok(())
    }

    /// 線形減衰つきの伝播。
    ///
    /// 対象軸を最下位キーに並べ替えてレーンへ切り、レーンごとの 1 次元走査で
    /// 出力を直接作る。
    ///
    /// 伝播先が F / Y の軸範囲からはみ出す場合は [`Applied::Unsupported`] を返して
    /// 木経路へ譲る。理由は [`Applied`] を参照。
    fn falloff(
        &mut self,
        axis: GridAxis,
        op_z: ZoomLevel,
        radius: u32,
        atten: &AttenFn<'_, V>,
        resolve: &ResolveFn<'_, V>,
    ) -> Applied {
        self.sort_lanes(axis);

        let span = 1i64 << self.z.get();
        let stride = 1i64 << (self.z.get() - op_z.get());
        let reach = radius as i64 * stride;

        // 伝播が軸の外へ届くか。届かないなら、はみ出しも巡回も起こらない。
        let overflows = self.axis_span(axis).is_some_and(|s| {
            let (lo, hi) = if axis == GridAxis::F {
                (-span, span - 1)
            } else {
                (0, span - 1)
            };
            s.start() - reach < lo || s.end() + reach > hi
        });

        let wrap = match (axis, overflows) {
            // はみ出さないなら 3 軸とも同じ扱いでよい。X の巡回による整列やり直しも省ける。
            (_, false) => None,
            // X だけは巡回できる。はみ出す先は反対側へ回り込む。
            (GridAxis::X, true) => Some(span),
            // F / Y は葉単位の全か無かを再現できないので木経路へ譲る。
            (GridAxis::F | GridAxis::Y, true) => return Applied::Unsupported,
        };

        // 移動量は「ズーム op_z のインデックス」として有効な範囲に限られる
        // （`FlexId::shift_f` の `check_f`）。外れる分は元実装でも `Err` として捨てられる。
        let r = radius as i64;
        let df_range = match axis {
            GridAxis::F => (-r).max(op_z.f_min() as i64)..=r.min(op_z.f_max() as i64),
            _ => -r..=r,
        };

        let params = FalloffParams {
            axis,
            radius,
            stride,
            df_range,
            wrap,
        };
        self.entries = run_lanes(&self.entries, &params, atten, resolve);

        // 巡回（X）と飛び飛び格子（stride > 1）は出力が昇順・一意にならない。
        if params.stride != 1 || params.wrap.is_some() {
            self.order = None;
            self.sort_morton(&|a: &V, b: &V| resolve(a, b));
        }
        Applied::Done
    }
}

/// グリッド経路で実行しきれたか。
pub(crate) enum Applied {
    Done,
    /// グリッドでは元の意味を再現できないので木経路に任せる。
    ///
    /// F / Y の falloff が軸範囲からはみ出す場合がこれにあたる。
    /// [`FlexId::shift_f`](crate::FlexId::shift_f) / `shift_y` は**葉の占有区間を丸ごと**
    /// 見て範囲外なら `Err` を返し、`falloff_linear_*` はその `Err` を握り潰す。つまり
    /// 「1 マスでもはみ出したら、その距離ぶんの寄与は葉ごと落ちる」という全か無かの
    /// 挙動になっている。平坦化すると葉の切れ目が消えるので、この単位を再現できない。
    ///
    /// これは平坦化の原理的な限界ではなく、`falloff_linear_*` 側の粒度の粗さ
    /// （同じ空間でも木の圧縮のされ方で結果が変わる）を保存しているだけである。
    /// 将来 `falloff_linear_*` をマス単位のクリップへ直せば、この分岐ごと不要になる。
    Unsupported,
}

struct FalloffParams {
    axis: GridAxis,
    radius: u32,
    /// グリッド単位での 1 ステップ幅（`2^(グリッドズーム - 演算ズーム)`）。
    stride: i64,
    /// 適用してよい移動量の範囲（演算ズーム単位）。
    df_range: RangeInclusive<i64>,
    /// 出力位置を巡回させる幅。`None` なら軸からはみ出さないことが確認済み。
    wrap: Option<i64>,
}

/// 対象軸の位置を取り出す。
#[inline]
fn axis_pos<V>(entry: &SingleEntry<V>, axis: GridAxis) -> i64 {
    match axis {
        GridAxis::F => entry.0 as i64,
        GridAxis::X => entry.1 as i64,
        GridAxis::Y => entry.2 as i64,
    }
}

/// 対象軸の位置と値を差し替えたものを作る。
#[inline]
fn with_axis_pos<V>(entry: &SingleEntry<V>, axis: GridAxis, pos: i64, value: V) -> SingleEntry<V> {
    match axis {
        GridAxis::F => (pos as i32, entry.1, entry.2, value),
        GridAxis::X => (entry.0, pos as u32, entry.2, value),
        GridAxis::Y => (entry.0, entry.1, pos as u32, value),
    }
}

/// 同じレーン（対象軸以外の 2 軸が等しい）か。
#[inline]
fn same_lane<V>(a: &SingleEntry<V>, b: &SingleEntry<V>, axis: GridAxis) -> bool {
    match axis {
        GridAxis::F => a.1 == b.1 && a.2 == b.2,
        GridAxis::X => a.0 == b.0 && a.2 == b.2,
        GridAxis::Y => a.0 == b.0 && a.1 == b.1,
    }
}

/// レーン走査の作業領域。レーンをまたいで使い回す。
struct Scratch<V> {
    /// `atten[k * (radius+1) + d]` = 入力 k の距離 d での減衰値。
    atten: Vec<V>,
    /// 入力 k の対象軸の位置。`SingleEntry` から毎回引き直さないための写し。
    pos: Vec<i64>,
}

impl<V> Default for Scratch<V> {
    /// `derive(Default)` は `V: Default` を要求してしまうので手で書く。
    fn default() -> Self {
        Self {
            atten: Vec::new(),
            pos: Vec::new(),
        }
    }
}

/// 出力 `Vec` の見積もり容量。
fn estimated_output(entries: usize, params: &FalloffParams) -> usize {
    if params.stride == 1 {
        // 密なら入力 1 個につき前後 r マスぶん増える程度。
        entries.saturating_mul(2)
    } else {
        // 飛び飛び格子は素直に 2r+1 倍書き出す。
        entries.saturating_mul(2 * params.radius as usize + 1)
    }
}

/// 全レーンを走査して出力を作る。レーン同士は独立なので並列に処理できる。
fn run_lanes<V: SafeValue>(
    entries: &[SingleEntry<V>],
    params: &FalloffParams,
    atten: &AttenFn<'_, V>,
    resolve: &ResolveFn<'_, V>,
) -> Vec<SingleEntry<V>> {
    let axis = params.axis;

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        if entries.len() >= PAR_SLICE_CUTOFF {
            // 作業領域は fold のタスクごとに使い回す。レーン数は数百万になりうるので、
            // レーン単位で確保すると確保コストが支配的になる。
            let chunks: Vec<Vec<SingleEntry<V>>> = entries
                .par_chunk_by(|a, b| same_lane(a, b, axis))
                .fold(
                    || (Vec::new(), Scratch::default()),
                    |(mut out, mut scratch), lane| {
                        falloff_lane(lane, params, atten, resolve, &mut out, &mut scratch);
                        (out, scratch)
                    },
                )
                .map(|(out, _)| out)
                .collect();

            let mut out = Vec::with_capacity(chunks.iter().map(Vec::len).sum());
            for c in chunks {
                out.extend(c);
            }
            return out;
        }
    }

    let mut out = Vec::with_capacity(estimated_output(entries.len(), params));
    let mut scratch = Scratch::default();
    for lane in entries.chunk_by(|a, b| same_lane(a, b, axis)) {
        falloff_lane(lane, params, atten, resolve, &mut out, &mut scratch);
    }
    out
}

/// 1 レーンぶんの走査。`lane` は対象軸の昇順・位置一意。
///
/// `stride == 1`（＝演算ズームとグリッドズームが等しい）なら、出力位置ごとに半径内の
/// 入力を集める gather 型で走る。出力は昇順・一意で出るので、後段の整列も重なり解決も
/// 要らない。`stride > 1` なら出力が飛び飛びの格子になるので、素直に 2r+1 個ずつ
/// 書き出して呼び出し側の整列に任せる。
fn falloff_lane<V: SafeValue>(
    lane: &[SingleEntry<V>],
    params: &FalloffParams,
    atten: &AttenFn<'_, V>,
    resolve: &ResolveFn<'_, V>,
    out: &mut Vec<SingleEntry<V>>,
    scratch: &mut Scratch<V>,
) {
    if lane.is_empty() {
        return;
    }

    // 距離ごとの減衰値と対象軸の位置を先に作る。出力位置ごとに割り算をやり直さず、
    // 内側のループが 16 バイトの `SingleEntry` ではなく連続した配列を舐めるようにする。
    let width = params.radius as usize + 1;
    scratch.atten.clear();
    scratch.atten.reserve(lane.len() * width);
    scratch.pos.clear();
    scratch.pos.reserve(lane.len());
    for entry in lane {
        for d in 0..=params.radius {
            scratch.atten.push(atten(&entry.3, d));
        }
        scratch.pos.push(axis_pos(entry, params.axis));
    }

    if params.stride == 1 {
        gather_lane(lane, params, resolve, out, scratch);
    } else {
        scatter_lane(lane, params, out, scratch);
    }
}

/// 出力位置を 1 個書き出す。`template` からは対象軸以外の座標だけを使う。
#[inline]
fn emit<V>(
    out: &mut Vec<SingleEntry<V>>,
    template: &SingleEntry<V>,
    params: &FalloffParams,
    pos: i64,
    value: V,
) {
    let pos = params.wrap.map_or(pos, |w| pos.rem_euclid(w));
    out.push(with_axis_pos(template, params.axis, pos, value));
}

/// 飛び飛び格子（`stride > 1`）向けの散布。入力 1 個につき 2r+1 個書き出す。
fn scatter_lane<V: SafeValue>(
    lane: &[SingleEntry<V>],
    params: &FalloffParams,
    out: &mut Vec<SingleEntry<V>>,
    scratch: &Scratch<V>,
) {
    let width = params.radius as usize + 1;
    for (k, entry) in lane.iter().enumerate() {
        for df in params.df_range.clone() {
            let value = scratch.atten[k * width + df.unsigned_abs() as usize].clone();
            emit(
                out,
                entry,
                params,
                scratch.pos[k] + df * params.stride,
                value,
            );
        }
    }
}

/// 密な格子（`stride == 1`）向けの収集。出力位置ごとに半径内の入力を畳み込む。
///
/// 入力位置は昇順なので、出力位置 `q` を昇順に進めながら、窓 `[q-r, q+r]` に入る入力の
/// 区間 `[lo, hi)` を両端の追従だけで保てる（各ポインタはレーンを 1 周するだけ）。
fn gather_lane<V: SafeValue>(
    lane: &[SingleEntry<V>],
    params: &FalloffParams,
    resolve: &ResolveFn<'_, V>,
    out: &mut Vec<SingleEntry<V>>,
    scratch: &Scratch<V>,
) {
    let r = params.radius as i64;
    let width = params.radius as usize + 1;
    let pos = &scratch.pos[..];

    let (mut lo, mut hi) = (0usize, 0usize);
    let mut k = 0usize;
    while k < lane.len() {
        // 半径 r で被覆区間が繋がる入力のかたまりを 1 つ取り、その被覆区間を埋める。
        let start = pos[k] - r;
        let mut end = pos[k] + r;
        let mut last = k;
        while last + 1 < pos.len() && pos[last + 1] - r <= end + 1 {
            last += 1;
            end = pos[last] + r;
        }

        for q in start..=end {
            while lo < pos.len() && pos[lo] + r < q {
                lo += 1;
            }
            hi = hi.max(lo);
            while hi < pos.len() && pos[hi] - r <= q {
                hi += 1;
            }

            let mut acc: Option<V> = None;
            for (offset, &p) in pos[lo..hi].iter().enumerate() {
                let df = q - p;
                if !params.df_range.contains(&df) {
                    continue;
                }
                let v = &scratch.atten[(lo + offset) * width + df.unsigned_abs() as usize];
                acc = Some(match acc {
                    None => v.clone(),
                    Some(prev) => resolve(&prev, v),
                });
            }
            if let Some(v) = acc {
                emit(out, &lane[0], params, q, v);
            }
        }

        k = last + 1;
    }
}

/// 対象軸を最下位キーにして整列する。
///
/// キーは `u128` を組み立てず素のタプルにする。空間 3 軸のズームは高々
/// [`ZoomLevel::MAX`] = 30 なので、`f` にバイアスを足せば 3 要素の辞書式順序が
/// そのまま所望の順序になる（比較は先頭要素で早期終了する）。
fn sort_by_lane<V: SafeValue>(entries: &mut [SingleEntry<V>], axis: GridAxis) {
    /// F は負値を取るので、単調な符号なしへ寄せるためのバイアス。
    const F_BIAS: i64 = 1 << 30;

    #[inline]
    fn key<V>(entry: &SingleEntry<V>, axis: GridAxis) -> (u32, u32, u32) {
        let f = (entry.0 as i64 + F_BIAS) as u32;
        match axis {
            GridAxis::F => (entry.1, entry.2, f),
            GridAxis::X => (f, entry.2, entry.1),
            GridAxis::Y => (f, entry.1, entry.2),
        }
    }

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        if entries.len() >= PAR_SLICE_CUTOFF {
            entries.par_sort_unstable_by_key(|e| key(e, axis));
            return;
        }
    }
    entries.sort_unstable_by_key(|e| key(e, axis));
}

/// 単項演算の並びをグリッド経路で実行する。
///
/// 木を平坦化できた場合だけ `Some` を返す。`None` なら呼び出し側は従来の木経路で実行する。
pub(crate) fn try_run_grid<V: SafeValue>(
    tree: &WorkingTree<V>,
    ops: &[GridOp<V>],
    budget: u64,
) -> Option<Result<WorkingTree<V>, Error>> {
    // 0 個の演算に「成功」を返すと、呼び出し側の走査が 1 つも進まなくなる。
    if ops.is_empty() {
        return None;
    }
    // グリッドのズームは、演算のズームと入力の解像度の両方を表せる細かさが要る。
    let z = ops
        .iter()
        .map(|op| op.zoom().get())
        .chain(tree.core().max_zoomlevel())
        .max()?;
    let z = ZoomLevel::new(z).ok()?;

    let mut grid = UniformGrid::from_tree(tree, z, budget)?;
    for op in ops {
        match grid.apply(op) {
            Ok(Applied::Done) => {}
            // 途中まで進めたグリッドは捨てる。呼び出し側が元の木から木経路でやり直す。
            Ok(Applied::Unsupported) => return None,
            Err(e) => return Some(Err(e)),
        }
    }
    Some(Ok(grid.into_tree()))
}
