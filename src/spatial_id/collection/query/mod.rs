/// クエリ実行を途中で打ち切るための協調的キャンセル
pub mod cancellation;

/// 演算子の種類
pub mod ops;

/// 複数の値が同じ空間で衝突した際の解決ポリシー
pub mod merge_policy;
pub use merge_policy::MergePolicy;

use crate::{
    Error, FlexId, Interval, RangeId, SafeValue, SpatialId,
    spatial_id::collection::flex_tree::core::ptr::MaybeSendSync,
};
use alloc::{boxed::Box, vec, vec::Vec};
use cancellation::CancellationToken;

/// `(FlexId, V)` を1件ずつ生成するイテレーター。
pub type ValueIter<'a, V> = Box<dyn Iterator<Item = (FlexId, V)> + 'a>;

/// Query全体を表現する型。
pub enum Query<V: SafeValue + 'static> {
    /// 演算の起点となる[Source]
    Source(Box<dyn Source<Value = V>>),

    /// 連続した単項演算子
    Unary(Vec<Box<dyn UnaryOperator<V>>>, Box<Query<V>>),

    // 二項演算
    Binary(Box<dyn BinaryOperator<V>>, Box<Query<V>>, Box<Query<V>>),

    /// エラー状態
    Error(Error),
}

impl<V: SafeValue + 'static> Query<V> {
    /// 検証してから実行し、結果を`(FlexId, V)`のペアとして1件ずつ生成するイテレーターを返す。
    pub fn run(&self) -> Result<ValueIter<'_, V>, Error> {
        self.run_cancellable(CancellationToken::never())
    }

    /// 検証してから実行し、結果を`(FlexId, V)`のペアとして1件ずつ生成するイテレーターを返す。[CancellationToken]を用いてキャンセルができる。
    pub fn run_cancellable(&self, token: CancellationToken) -> Result<ValueIter<'_, V>, Error> {
        self.validate()?;
        self.run_within_unchecked(RangeId::everything(), &token)
    }

    /// `target` と交差する部分だけを検証してから評価する。
    ///
    /// 演算子は [`UnaryOperator::inverse_bounds`]/[`BinaryOperator::inverse_bounds`] を使って
    /// 「`target` を得るのに実際どこまで入力が必要か」を逆算しながら木を降りるため、
    /// falloff/shiftのように近傍を参照する演算でも、対象領域の外にある無関係な入力を
    /// 読み込まずに済む。
    pub fn run_within(
        &self,
        target: impl Into<RangeId>,
        token: CancellationToken,
    ) -> Result<ValueIter<'_, V>, Error> {
        self.validate()?;
        self.run_within_unchecked(target.into(), &token)
    }

    /// [`run_within`](Self::run_within) の本体。事前に [`validate`](Self::validate) 済みであることを前提とする。
    fn run_within_unchecked(
        &self,
        target: RangeId,
        token: &CancellationToken,
    ) -> Result<ValueIter<'_, V>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        match self {
            Query::Source(source) => source.get(target, token.clone()),
            Query::Unary(ops, input) => {
                // 逆算はASTに書かれた順の逆（実行の逆順）で辿る。同時に、各演算子が
                // 自分の出力として本当に満たすべき領域（＝次の演算子が要求してきた領域）を
                // 覚えておく。falloff/extrudeはこれを使って、対象領域の外に出る候補を
                // 生成した端から捨てられる（scatterしてから捨てるのではなく、
                // 最初から要らないものを作らない）。
                let mut targets: Vec<RangeId> = Vec::with_capacity(ops.len());
                let mut req = target;
                let mut reachable = true;
                for op in ops.iter().rev() {
                    targets.push(req.clone());
                    match op.inverse_bounds(req.clone()) {
                        Some(t) => req = t,
                        None => {
                            reachable = false;
                            break;
                        }
                    }
                }
                if !reachable {
                    return Ok(Box::new(core::iter::empty()));
                }
                targets.reverse();

                let mut iter = input.run_within_unchecked(req, token)?;
                for (op, op_target) in ops.iter().zip(targets) {
                    if token.is_cancelled() {
                        return Err(Error::Cancelled);
                    }
                    iter = op.run(iter, op_target, token.clone())?;
                }
                Ok(iter)
            }
            Query::Binary(op, lhs, rhs) => {
                let (l, r) = op.inverse_bounds(target);
                let lhs_iter = match l {
                    Some(t) => lhs.run_within_unchecked(t, token)?,
                    None => Box::new(core::iter::empty()),
                };
                let rhs_iter = match r {
                    Some(t) => rhs.run_within_unchecked(t, token)?,
                    None => Box::new(core::iter::empty()),
                };
                op.run(lhs_iter, rhs_iter, token.clone())
            }
            Query::Error(e) => Err(e.clone()),
        }
    }

    /// `target`と交差する部分だけを評価して返す。
    pub fn lazy_get<T: SpatialId>(&self, target: T) -> Result<ValueIter<'_, V>, Error> {
        let range: RangeId = target.into();
        let iter = self.run_within(range.clone(), CancellationToken::never())?;
        Ok(Box::new(
            iter.filter(move |(id, _)| id.intersects_range(&range)),
        ))
    }

    /// 上流(`input`)が実際に持っているSegmentごとに、その値が届きうる出力領域を
    /// [`UnaryOperator::forward_bounds`]で求め、それぞれを[`run_within`](Self::run_within)で
    /// 評価してつなげる。
    ///
    /// タイルのような人為的なグリッドは敷かない。分割の単位はSource(または上流)が実際に
    /// 持っているSegmentの粒度そのものなので、疎な場所では大きな単位、密な場所では
    /// 細かい単位に自然と適応する。
    ///
    /// ただし近くのSegment同士は`forward_bounds`が重なり合うため、同じ出力位置が複数の
    /// 呼び出しから求まり得る。これは`FlexId`単位の重複除去で吸収している(値そのものを
    /// 覚えておくよりずっと小さく済むが、出力全体の大きさに比例して増える点は残る)。
    /// また重複した呼び出し自体(Sourceの同じ範囲を何度も読むこと)はここでは解決しない
    /// — 別途キャッシュ層を挟む話として切り分けている。
    ///
    /// `Query::Unary`以外(`Source`単体や`Binary`)では分割の余地が無いため、
    /// 素直に[`run_within`](Self::run_within)へフォールバックする。
    pub fn run_by_segments(
        &self,
        token: CancellationToken,
    ) -> Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + '_> {
        let Query::Unary(ops, input) = self else {
            return match self.run_within(RangeId::everything(), token) {
                Ok(iter) => Box::new(iter.map(Ok)),
                Err(e) => Box::new(core::iter::once(Err(e))),
            };
        };

        let base = match input.run_within_unchecked(RangeId::everything(), &token) {
            Ok(iter) => iter,
            Err(e) => return Box::new(core::iter::once(Err(e))),
        };

        // 各Segmentの影響範囲を先に集める。値そのものではなく領域だけを覚えるので、
        // 元のデータより十分小さい。
        let mut candidates: Vec<RangeId> = Vec::new();
        for (id, _) in base {
            let mut region: RangeId = id.into();
            let mut reachable = true;
            for op in ops.iter() {
                match op.forward_bounds(region.clone()) {
                    Some(r) => region = r,
                    None => {
                        reachable = false;
                        break;
                    }
                }
            }
            if reachable {
                candidates.push(region);
            }
        }

        // falloffのように半径を持つ演算では、近くのSegmentの影響範囲同士が大きく重なり合う。
        // 重なりを無視して候補ごとに`run_within`を呼ぶと、同じ近傍を候補の数だけ何度も
        // 読み直し・再計算することになり、割り当てる単位を細かくした分だけ遅くなってしまう。
        // 触れ合っている候補どうしを1つの領域へ統合してから評価することで、この重複を
        // データの疎密に応じて自然に減らす(密な場所ほど大きく統合され、疎な場所では
        // 元のまま小さい単位が残る — タイルサイズのような決め打ちの定数は使わない)。
        let candidates = merge_overlapping_candidates(candidates);

        let mut seen: alloc::collections::BTreeSet<FlexId> = alloc::collections::BTreeSet::new();
        Box::new(candidates.into_iter().flat_map(move |target| {
            match self.run_within(target, token.clone()) {
                Ok(iter) => {
                    let deduped: Vec<Result<(FlexId, V), Error>> =
                        iter.filter(|(id, _)| seen.insert(*id)).map(Ok).collect();
                    Box::new(deduped.into_iter())
                        as Box<dyn Iterator<Item = Result<(FlexId, V), Error>>>
                }
                Err(e) => Box::new(core::iter::once(Err(e))),
            }
        }))
    }

    /// `self` を単項演算子で包む。
    pub(crate) fn wrap_unary<O>(self, op: O) -> Self
    where
        O: UnaryOperator<V> + 'static,
    {
        match self {
            Query::Unary(mut ops, input) => {
                ops.push(Box::new(op));
                Query::Unary(ops, input)
            }
            other => Query::Unary(
                vec![Box::new(op) as Box<dyn UnaryOperator<V>>],
                Box::new(other),
            ),
        }
    }

    /// 実行までに全てのQueryのパラメーターが正常値の範囲内か検証する
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Query::Source(_) => Ok(()),
            Query::Unary(ops, input) => {
                input.validate()?;
                for op in ops {
                    op.validate()?;
                }
                Ok(())
            }
            Query::Binary(op, lhs, rhs) => {
                lhs.validate()?;
                rhs.validate()?;
                op.validate()
            }
            Query::Error(e) => Err(e.clone()),
        }
    }
}

/// 隙間なく重なる、または触れ合っている候補領域どうしを1つに統合する。
///
/// 統合の基準は候補どうしの実際の重なり(F/X/Y全軸で隙間がゼロであること)だけであり、
/// タイルサイズのような決め打ちの定数は一切使わない。そのため密な場所ほど大きな単位に、
/// 疎な場所では元のまま小さな単位に自然と落ち着く。
///
/// 統合はソートした順に隣接候補とだけ比較する一回の走査で行うため、ソート順で
/// 離れた場所にある候補同士の重なりは見逃すことがある。ただしその場合も単に統合されず
/// 個別に評価されるだけであり(統合前の状態と同じ)、誤って過大な領域を作ることはない。
///
/// Xが周期境界をまたいでいる候補、Fが下限なし(`-1`)の候補、全時間でない候補は、
/// 正しく統合するための軸情報を素直に扱えないため対象から外し、個別の候補として残す。
fn merge_overlapping_candidates(candidates: Vec<RangeId>) -> Vec<RangeId> {
    let (mut mergeable, rest): (Vec<RangeId>, Vec<RangeId>) =
        candidates.into_iter().partition(|r| {
            r.x()[0] <= r.x()[1] && r.f()[0] != -1 && r.time_interval() == Interval::WHOLE
        });

    if mergeable.len() < 2 {
        mergeable.extend(rest);
        return mergeable;
    }

    let cmp_z = mergeable.iter().map(|r| r.z()).max().unwrap_or(0);
    mergeable.sort_by_key(|r| {
        let (f_min, _) = r.f_fine_range(cmp_z);
        let (x_min, _) = r.x_fine_range(cmp_z);
        let (y_min, _) = r.y_fine_range(cmp_z);
        (f_min, x_min, y_min)
    });

    let mut merged: Vec<RangeId> = Vec::with_capacity(mergeable.len());
    let mut iter = mergeable.into_iter();
    if let Some(first) = iter.next() {
        let mut acc = first;
        for next in iter {
            match union_if_touching(&acc, &next, cmp_z) {
                Some(u) => acc = u,
                None => {
                    merged.push(acc);
                    acc = next;
                }
            }
        }
        merged.push(acc);
    }

    merged.extend(rest);
    merged
}

/// `a`と`b`がF/X/Y全軸で隙間なく重なっている(重複、または境界がぴったり接している)なら、
/// それらを覆う最小の領域を返す。1軸でも隙間があれば`None`。
fn union_if_touching(a: &RangeId, b: &RangeId, cmp_z: u8) -> Option<RangeId> {
    let (af0, af1) = a.f_fine_range(cmp_z);
    let (bf0, bf1) = b.f_fine_range(cmp_z);
    let (ax0, ax1) = a.x_fine_range(cmp_z);
    let (bx0, bx1) = b.x_fine_range(cmp_z);
    let (ay0, ay1) = a.y_fine_range(cmp_z);
    let (by0, by1) = b.y_fine_range(cmp_z);

    let touches = |a0: i64, a1: i64, b0: i64, b1: i64| a0 <= b1 + 1 && b0 <= a1 + 1;
    if !touches(af0 as i64, af1 as i64, bf0 as i64, bf1 as i64)
        || !touches(ax0 as i64, ax1 as i64, bx0 as i64, bx1 as i64)
        || !touches(ay0 as i64, ay1 as i64, by0 as i64, by1 as i64)
    {
        return None;
    }

    RangeId::new(
        cmp_z,
        [af0.min(bf0), af1.max(bf1)],
        [ax0.min(bx0), ax1.max(bx1)],
        [ay0.min(by0), ay1.max(by1)],
    )
    .ok()
}

/// 二項演算子の定義。
pub trait BinaryOperator<V: SafeValue>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// `lhs` と `rhs` を1つに合成した結果を流すイテレーターを作る。
    fn run<'a>(
        &'a self,
        lhs: ValueIter<'a, V>,
        rhs: ValueIter<'a, V>,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error>;

    /// 出力領域 `output` を得るのに必要な、左右それぞれの入力領域を逆算する。
    /// 既定は両辺とも `output` と同じ領域を必要とする(Intersection/Difference/Mergeはこれで正しい)。
    fn inverse_bounds(&self, output: RangeId) -> (Option<RangeId>, Option<RangeId>) {
        (Some(output.clone()), Some(output))
    }
}

/// 単項演算子の定義。
pub trait UnaryOperator<V: SafeValue>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// `input` にこの演算を適用した結果を流すイテレーターを作る。
    ///
    /// `target`は呼び出し側が本当に必要としている出力領域(`inverse_bounds`に渡したのと
    /// 同じもの)。falloff/extrudeのように1入力が複数候補へ展開されうる演算は、これを使って
    /// `target`と交差しない候補をscatterした端から捨てられる。座標を動かさない演算は
    /// 無視してよい。
    ///
    /// `token` は協調的キャンセル用で、時間のかかる集約処理の合間に確認する。
    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error>;

    /// 出力領域 `output` を得るのに必要な入力領域を逆算する。
    ///
    /// 既定は恒等写像で、形を変えない演算(filterなど)はこのままでよい。`shift`/`falloff`の
    /// ように座標を動かしたり近傍を参照したりする演算だけがオーバーライドする必要がある。
    /// `None` は「この入力は`output`に一切寄与しない」ことを意味し、呼び出し側はその入力の
    /// 評価を丸ごと省略できる。
    fn inverse_bounds(&self, output: RangeId) -> Option<RangeId> {
        Some(output)
    }

    /// [`inverse_bounds`](Self::inverse_bounds)の逆方向。入力領域 `input` の値が
    /// 影響を与えうる出力領域を求める。
    ///
    /// 既定は恒等写像。`shift`/`falloff`のように座標を動かす演算だけがオーバーライドする
    /// 必要がある。`extrude`のように「どの入力も同じ固定範囲に写る」演算では、`input`に
    /// 関係なく常に同じ範囲を返すことになる(その演算にはこの逆算を使う意味が薄い)。
    /// `None`は「この入力はどこにも影響しない」ことを意味する。
    fn forward_bounds(&self, input: RangeId) -> Option<RangeId> {
        Some(input)
    }
}

/// クエリを実行するためのTrait。
pub trait Source: MaybeSendSync {
    type Value: SafeValue;

    /// `target` と交差する部分の[FlexId]と値の組を取り出す
    fn get<'a>(
        &'a self,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, Self::Value>, Error>;

    fn query(self) -> Query<Self::Value>
    where
        Self: Sized + 'static,
    {
        Query::Source(Box::new(self))
    }
}

/// `Source` を実装する型を、二項演算子の引数などで直接 [`Query`] として渡せるようにする。
impl<V: SafeValue + 'static, S: Source<Value = V> + 'static> From<S> for Query<V> {
    fn from(source: S) -> Self {
        source.query()
    }
}
