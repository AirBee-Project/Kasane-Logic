use super::execution::group_commutative::types::CommutativityInfo;
use crate::FlexId;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
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
    fn inverse_bounds(&self, output_bounds: RangeId) -> (Option<RangeId>, Option<RangeId>);

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

    /// 演算子を適用した際のデータサイズの推定拡大倍率
    fn expansion_ratio(&self) -> f64 {
        1.0
    }

    /// 与えられた出力領域を計算するために必要な入力領域を逆算する。
    /// 遅延ビュー（Lazy View）が部分木を構築するために使用する。
    fn inverse_bounds(&self, output_bounds: RangeId) -> Option<RangeId>;

    /// `Display` 出力用の演算子表現
    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "?")
    }

    /// 入力の (FlexId, V) 1 つが生成する出力 (FlexId, V) を `out` へ追記する。
    ///
    /// 中間の [`WorkingTree`] を構築せずに合成実行するために使う。
    /// デフォルト実装は 1 要素の WorkingTree を作り [`run`](Self::run) へ委譲する。
    fn forward_map(&self, id: FlexId, value: V, out: &mut Vec<(FlexId, V)>) -> Result<(), Error> {
        let mut wt: WorkingTree<V> = core::iter::once((id, value)).collect();
        self.run(&mut wt)?;
        out.extend(wt);
        Ok(())
    }

    /// この演算子が `forward_map` を用いた直接合成計算をサポートしているか。
    /// false を返す場合、`run_composed_chain` はこの演算子の前で木を再構築する。
    fn can_forward_map(&self) -> bool {
        true
    }

    /// 同じ [`FlexId`] に複数の出力が集まる演算子は、ここでマージ関数を返す。
    /// 返される関数は、ソート済みの `data` を受け取ってインプレースでマージを行う。
    ///
    /// 衝突しない演算子（shift, filter_values 等）は [`None`]（デフォルト）を返す。
    #[allow(clippy::type_complexity)]
    fn collision_merge(&self) -> Option<fn(&mut Vec<(FlexId, V)>)> {
        None
    }

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn grid_zoom(&self) -> Option<crate::ZoomLevel> {
        None
    }

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn apply_to_grid(
        &self,
        _grid: &mut crate::spatial_id::collection::query::grid::UniformGrid<V>,
        _token: &crate::CancellationToken,
    ) -> Result<crate::spatial_id::collection::query::grid::Applied, Error> {
        Ok(crate::spatial_id::collection::query::grid::Applied::Unsupported)
    }
}
