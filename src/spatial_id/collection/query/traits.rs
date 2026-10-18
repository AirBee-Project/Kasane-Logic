use crate::spatial_id::collection::query::send_sync::MaybeSendSync;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{Error, FlexId, RangeId};
use alloc::boxed::Box;

pub type QueryIter<'a, V> = Box<dyn Iterator<Item = Result<(FlexId, V), Error>> + 'a>;

/// 二項演算子の定義。
pub trait BinaryOperator<V: SafeValue + 'static>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// `lhs` と `rhs` を受け取って二項演算した結果のイテレータを返す。
    fn run<'a>(
        &self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error>;

    /// 与えられた出力領域を計算するために必要な入力領域を逆算する。
    fn reverse_target(&self, output_target: RangeId) -> (Option<RangeId>, Option<RangeId>);
}

/// 単項演算子の定義。
pub trait UnaryOperator<V: SafeValue + 'static>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error>;

    /// 入力イテレータを受け取って単項演算した結果のイテレータを返す。
    fn run<'a>(&self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error>;

    /// 与えられた出力領域を計算するために必要な入力領域を逆算する。
    /// 遅延ビュー（Lazy View）が部分木を構築するために使用する。
    fn reverse_target(&self, output_target: RangeId) -> Option<RangeId>;
}
