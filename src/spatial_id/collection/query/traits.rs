use super::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::{MaybeSend, MaybeSendSync, MaybeSync};
use crate::{Error, FlexId, FlexTreeCore, RangeId};
use alloc::boxed::Box;
use alloc::vec::Vec;

/// クエリ実行器・演算子が触れる「作業表現」の境界。演算子が実際に呼ぶメソッドだけを持つ
/// （`map_rebuild`/`map_rebuild_with` = per-cell 写像の recombiner、`from_items` = 展開結果からの
/// 再構築、`iter_ref`/`count` = 走査）。具象型 [`FlexTreeCore`] を `SpatialIdCollection` の公開
/// シグネチャへ露出させないための境界で、`SpatialIdCollection::Working` はこれを実装する。
pub trait WorkingTree: Sized + MaybeSendSync {
    type Value: SafeValue;

    /// 値付きセルの数。
    fn count(&self) -> usize;

    /// 全セルを参照で走査する。
    fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &Self::Value)> + '_;

    /// このツリーに含まれる全セルを包む最小の [`RangeId`] を返す。空の場合は [`None`]。
    fn bounding_box(&self) -> Option<RangeId>;

    /// `(FlexId, Value)` 列からツリーを構築する（重なりは union・左優先）。
    fn from_items(items: Vec<(FlexId, Self::Value)>) -> Self;

    /// 自分自身をベースとし、`other` のセルで上書き重ね合わせを行った新しいツリーを返す。
    fn overlay(&self, other: &Self) -> Self;

    /// 各セルを `f` で写し、union（左優先）で組み直す。写像先が空間的に単射な演算子向け。
    fn map_rebuild<F, I>(&self, f: F) -> Result<Self, Error>
    where
        F: Fn(FlexId, &Self::Value) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, Self::Value)> + MaybeSend;

    /// 各セルを `f` で写し、写像先の重なりを `resolve` で合成して組み直す。写像先が空間的に
    /// 非単射な演算子向け。
    fn map_rebuild_with<F, I, R>(&self, f: F, resolve: R) -> Result<Self, Error>
    where
        F: Fn(FlexId, &Self::Value) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, Self::Value)> + MaybeSend,
        R: Fn(&Self::Value, &Self::Value) -> Self::Value + MaybeSync;

    /// 2つの作業木を、片側が空の領域も `default` で埋めてから `resolve` で重ね合わせる
    /// （両側とも空の領域は resolve を呼ばず空のまま）。二項演算子向け。
    fn merge_with_default<R>(&self, other: &Self, default: &Self::Value, resolve: R) -> Self
    where
        R: Fn(&Self::Value, &Self::Value) -> Self::Value + MaybeSync;
}

impl<V: SafeValue> WorkingTree for FlexTreeCore<V> {
    type Value = V;

    fn count(&self) -> usize {
        FlexTreeCore::count(self)
    }

    fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        FlexTreeCore::iter_ref(self)
    }

    fn bounding_box(&self) -> Option<RangeId> {
        FlexTreeCore::bounding_box(self)
    }

    fn from_items(items: Vec<(FlexId, V)>) -> Self {
        FlexTreeCore::from_items(items)
    }

    fn overlay(&self, other: &Self) -> Self {
        self.merge_with(other, |_base, top| top.clone())
    }

    fn map_rebuild<F, I>(&self, f: F) -> Result<Self, Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
    {
        FlexTreeCore::map_rebuild(self, f)
    }

    fn map_rebuild_with<F, I, R>(&self, f: F, resolve: R) -> Result<Self, Error>
    where
        F: Fn(FlexId, &V) -> Result<I, Error> + MaybeSendSync,
        I: IntoIterator<Item = (FlexId, V)> + MaybeSend,
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        FlexTreeCore::map_rebuild_with(self, f, resolve)
    }

    fn merge_with_default<R>(&self, other: &Self, default: &V, resolve: R) -> Self
    where
        R: Fn(&V, &V) -> V + MaybeSync,
    {
        FlexTreeCore::merge_with_default(self, other, default, resolve)
    }
}

/// クエリ実行の作業表現である [`WorkingTree`] に対する二項演算子の定義。
///
/// 実行器は連鎖の入口で 1 回だけコレクションを `S::Working` へ変換し、以降の演算子はすべて
/// これに対して動く。
pub trait BinaryOperator<W: WorkingTree>: MaybeSendSync {
    /// 実行前の事前検証（パラメータ検証など）。デフォルトは何もしない（`Ok(())`）。
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// 作業木 `target_a` を、`target_b` を右辺として二項演算した結果へ更新する。
    fn run(&self, target_a: &mut W, target_b: &W) -> Result<(), Error>;
}

/// クエリ実行の作業表現である [`WorkingTree`] に対する単項演算子の定義。
///
/// 演算子は「各セルの値の反映先を決める写像」であり、反映先が単射なら union（[`WorkingTree::map_rebuild`]）、
/// 非単射なら値解決付き（[`WorkingTree::map_rebuild_with`]）で組み直す。パラメーターは各演算子の
/// 構造体フィールドが保持する。
pub trait UnaryOperator<W: WorkingTree>: MaybeSendSync + core::any::Any {
    /// 実行前の事前検証（パラメータ検証など）。
    fn validate(&self) -> Result<(), Error>;

    /// 作業木 `target` をインプレースで単項演算した結果へ更新する。
    fn run(&self, target: &mut W) -> Result<(), Error>;

    /// この演算子の可換性情報を返す。
    fn commutativity_info(&self) -> CommutativityInfo;

    /// ダウンキャスト用
    fn as_any(&self) -> &dyn core::any::Any;

    /// `other` が自分と同じ具象型かつパラメータ的に合成可能なら、両者を1つに統合した演算子を
    /// 返す。デフォルトは「常にマージ不可」。個々の演算子は `other.as_any().downcast_ref::<Self>()`
    /// で同型判定した後、完全に型付きのRustでパラメータを合成して実装する
    /// （`Query::CommutativeGroup` 内、つまり互いに可換であることが既に保証された演算子同士でのみ
    /// 呼ばれる）。
    fn try_merge(&self, other: &dyn UnaryOperator<W>) -> Option<Box<dyn UnaryOperator<W>>> {
        let _ = other;
        None
    }

    /// 演算子を適用した際のデータサイズ（セル数）の推定拡大倍率（概算）。
    /// 1.0はサイズ不変、<1.0は縮小、>1.0は拡大を表す。
    /// 可換グループ内では、この倍率が小さい順に実行されるよう自動的に並べ替えられ、
    /// 複数の中間データ拡大操作が連続した場合の処理コスト総和を最小化する。
    fn expansion_ratio(&self) -> f32 {
        1.0
    }

    /// 対象データのバウンディングボックス（空間的な広がり）を考慮して、
    /// オペレータ適用時の実質的なデータ拡張倍率（概算）を計算する。
    /// 空間が既に広い（データが密集している）場合はマージにより実際の拡張倍率が小さくなることを反映する。
    fn effective_expansion_ratio(&self, _bbox: Option<&crate::RangeId>) -> f32 {
        self.expansion_ratio()
    }
}

/// 演算子family の「正準蓄積器」。互いに可換な複数の具象型演算子（自分自身を含む）を1つに
/// 畳み込むための型が実装する（例: Shift familyの `ShiftFXY`）。`try_merge` を家族の各メンバーが
/// 個別に総当たりで実装する代わりに、「どの具象型がfamilyに属し、どう合成するか」をこの1箇所
/// （蓄積器側）へ集約できる。新しい家族（例: 将来のRotate系）を増やす際は、その正準型に対して
/// このtraitを実装し、家族の各メンバーの `try_merge` を [`try_merge_via_accumulator`] へ委譲する
/// だけでよい。
pub trait MergeAccumulator<W: WorkingTree>: UnaryOperator<W> + Sized {
    /// `op` がこのfamilyに属するなら、それを表す蓄積器を生成する。
    fn seed(op: &dyn UnaryOperator<W>) -> Option<Self>;

    /// `op` を `self` へ合成する。`op` がこのfamilyに属さない、またはパラメータ的に合成不能
    /// なら `false` を返し `self` は変更しない（部分的な適用も行わない）。
    fn absorb(&mut self, op: &dyn UnaryOperator<W>) -> bool;
}

/// `A` をfamilyの正準蓄積器として、`lhs`・`rhs` を1つの演算子へ畳み込めるか試す。
/// family内の各メンバーの `try_merge` はこれへ委譲するだけでよい。
pub fn try_merge_via_accumulator<W, A>(
    lhs: &dyn UnaryOperator<W>,
    rhs: &dyn UnaryOperator<W>,
) -> Option<Box<dyn UnaryOperator<W>>>
where
    W: WorkingTree + 'static,
    A: MergeAccumulator<W> + 'static,
{
    let mut acc = A::seed(lhs)?;
    if acc.absorb(rhs) {
        Some(Box::new(acc))
    } else {
        None
    }
}
