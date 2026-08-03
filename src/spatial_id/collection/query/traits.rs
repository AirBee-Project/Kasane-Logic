use super::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::grid::GridOp;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, RangeId};
use alloc::vec::Vec;

/// 二項演算子の定義。
pub trait BinaryOperator<V: SafeValue>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// 作業木 `target_a` を、`target_b` を右辺として二項演算した結果へ更新する。
    fn run(&self, target_a: &mut WorkingTree<V>, target_b: &WorkingTree<V>) -> Result<(), Error>;

    /// 与えられた出力領域を計算するために必要な入力領域を逆算する。
    /// 返り値は (target_a の必要領域, target_b の必要領域)。
    fn inverse_bounds(&self, output_bounds: RangeId) -> (Vec<RangeId>, Vec<RangeId>);

    /// `Display` 出力用の演算子表現
    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "?")
    }
}

/// 単項演算子の定義。
pub trait UnaryOperator<V: SafeValue>: MaybeSendSync + core::any::Any {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error>;

    /// 実行する
    fn run(&self, target: &mut WorkingTree<V>) -> Result<(), Error>;

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

    // --- 一様ズーム平坦表現（グリッド）用API ---

    /// この演算子を一様ズームの平坦表現の上で直接実行するための記述を返す。
    ///
    /// クエリエンジンの内部最適化の口であり、クレート外からは実装できない
    /// （`GridOp` を組み立てる手段が非公開）。既定の `None` のままでよい。
    ///
    /// `None` なら木経路（[`run`](Self::run)）で実行される。分離可能な演算（軸方向の
    /// 平行移動だけで表せるもの）だけが `Some` を返せる。衝突解決を伴う演算は、可換な
    /// [`MergePolicy`](crate::merge_policy::MergePolicy) のときだけ `Some` を返すこと
    /// （グリッド側は畳み込み順を保証しない）。
    #[doc(hidden)]
    fn grid_op(&self) -> Option<GridOp<V>> {
        None
    }
}
