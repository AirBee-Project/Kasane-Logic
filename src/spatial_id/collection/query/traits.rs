use crate::FlexTreeCore;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use alloc::boxed::Box;

/// クエリ実行の作業表現である [`FlexTreeCore`] に対する二項演算子の定義。
///
/// 実行器は連鎖の入口で 1 回だけコレクションを `FlexTreeCore<V>` へ変換し、以降の演算子はすべて
pub trait BinaryOperator<V: SafeValue>: Send + Sync {
    /// 作業木 `target_a` を、`target_b` を右辺として二項演算した結果へ更新する。
    fn run(
        &self,
        target_a: &mut FlexTreeCore<V>,
        target_b: &FlexTreeCore<V>,
    ) -> Result<(), Box<dyn core::error::Error + 'static>>;
}

/// クエリ実行の作業表現である [`FlexTreeCore`] に対する単項演算子の定義。
///
/// 演算子は「各セルの値の反映先を決める写像」であり、反映先が単射なら union（[`FlexTreeCore::map_rebuild`]
/// または構造シフト [`FlexTreeCore::shift_x`]）、非単射なら merge_with（[`FlexTreeCore::map_rebuild_with`]）で
/// 組み直す。パラメーターは各演算子の構造体フィールドが保持する。
pub trait UnaryOperator<V: SafeValue>: Send + Sync {
    /// 作業木 `target` をインプレースで単項演算した結果へ更新する。
    fn run(
        &self,
        target: &mut FlexTreeCore<V>,
    ) -> Result<(), Box<dyn core::error::Error + 'static>>;
}
