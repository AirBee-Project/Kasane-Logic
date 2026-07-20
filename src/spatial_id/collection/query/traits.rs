use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::{MaybeSend, MaybeSendSync, MaybeSync};
use crate::{Error, FlexId, FlexTreeCore};
use alloc::vec::Vec;

/// クエリ実行器・演算子が触れる「作業表現」の境界。演算子が実際に呼ぶメソッドだけを持つ
/// （`map_rebuild`/`map_rebuild_with` = per-cell 写像の recombiner、`from_items` = 展開結果からの
/// 再構築、`iter_ref`/`count` = 走査）。具象型 [`FlexTreeCore`] を `SpatialIdCollection` の公開
/// シグネチャへ露出させないための境界で、`SpatialIdCollection::Working` はこれを実装する。
///
/// 現状は [`FlexTreeCore`] のみが実装する。`merge_with`/`union`/`intersection`/`difference` は
/// 現時点でどの演算子からも呼ばれていないため含めない（今後 `BinaryOperator` の実装を追加する
/// たびに、実際に使うものだけを合わせて追加する）。
pub trait WorkingTree: Sized + MaybeSendSync {
    type Value: SafeValue;

    /// 値付きセルの数。
    fn count(&self) -> usize;

    /// 全セルを参照で走査する。
    fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &Self::Value)> + '_;

    /// `(FlexId, Value)` 列からツリーを構築する（重なりは union・左優先）。
    fn from_items(items: Vec<(FlexId, Self::Value)>) -> Self;

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

    fn from_items(items: Vec<(FlexId, V)>) -> Self {
        FlexTreeCore::from_items(items)
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
    /// 作業木 `target_a` を、`target_b` を右辺として二項演算した結果へ更新する。
    fn run(&self, target_a: &mut W, target_b: &W) -> Result<(), Error>;
}

/// クエリ実行の作業表現である [`WorkingTree`] に対する単項演算子の定義。
///
/// 演算子は「各セルの値の反映先を決める写像」であり、反映先が単射なら union（[`WorkingTree::map_rebuild`]）、
/// 非単射なら値解決付き（[`WorkingTree::map_rebuild_with`]）で組み直す。パラメーターは各演算子の
/// 構造体フィールドが保持する。
pub trait UnaryOperator<W: WorkingTree>: MaybeSendSync {
    /// 作業木 `target` をインプレースで単項演算した結果へ更新する。
    fn run(&self, target: &mut W) -> Result<(), Error>;
}
