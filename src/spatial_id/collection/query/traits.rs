use super::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::{MaybeSend, MaybeSendSync, MaybeSync};
use crate::{Error, FlexId, FlexTreeCore, RangeId};
use alloc::vec::Vec;

/// クエリ実行器・演算子が触れるの境界。将来的にメモリ実装とDisk実装を抽象化するために使用される。
/// 走査（[`IntoIterator`]）と構築（[`FromIterator`]）は標準トレイトで表す。
/// 演算子は `items.into_iter().collect()` でツリーを組み直す。
pub trait WorkingTree:
    Sized
    + MaybeSendSync
    + IntoIterator<Item = (FlexId, Self::Value)>
    + FromIterator<(FlexId, Self::Value)>
{
    type Value: SafeValue;

    fn count(&self) -> usize;

    fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &Self::Value)> + '_;

    /// このツリーに含まれる全セルを包む最小の [`RangeId`] を返す。空の場合は [`None`]。
    ///
    /// 基本的にO(1)で返すこと。
    fn bounding_box(&self) -> Option<RangeId>;

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

/// 二項演算子の定義。
pub trait BinaryOperator<W: WorkingTree>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// 作業木 `target_a` を、`target_b` を右辺として二項演算した結果へ更新する。
    fn run(&self, target_a: &mut W, target_b: &W) -> Result<(), Error>;

    /// 与えられた出力領域を計算するために必要な入力領域を逆算する。
    /// 返り値は (target_a の必要領域, target_b の必要領域)。
    fn inverse_bounds(&self, output_bounds: RangeId) -> (Vec<RangeId>, Vec<RangeId>);

    /// `Display` 出力用の演算子表現
    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "?")
    }
}

/// 単項演算子の定義。
pub trait UnaryOperator<W: WorkingTree>: MaybeSendSync + core::any::Any {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error>;

    /// 実行する
    fn run(&self, target: &mut W) -> Result<(), Error>;

    /// この演算子の可換性情報
    fn commutativity_info(&self) -> CommutativityInfo;

    /// ダウンキャスト用
    fn as_any(&self) -> &dyn core::any::Any;

    /// 演算子を適用した際のデータサイズの推定拡大倍率。
    fn expansion_ratio(&self) -> f32 {
        1.0
    }

    /// 与えられた出力領域を計算するために必要な入力領域を逆算する。
    /// 遅延ビュー（Lazy View）が部分木を構築するために使用する。
    fn inverse_bounds(&self, output_bounds: RangeId) -> Vec<RangeId>;

    /// `Display` 出力用の演算子表現
    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "?")
    }
}
