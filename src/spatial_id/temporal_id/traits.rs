use core::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::error::Error;

#[cfg(doc)]
use crate::{FlexId, RangeId, SingleId};

/// 時間区間を表す型が共通して持つTrait。
///
/// [`FlexId`]/[`SingleId`]（点）は生の2進セル表現（`TemporalSegment`）を、[`RangeId`]（範囲）は
/// 人間に読みやすい範囲表現（`TemporalRange`）を持つ。両者の形は異なるが、
/// [`SpatialId::temporal`](crate::SpatialId::temporal) を通じて共通の操作（全時間判定・絶対秒区間
/// への変換）を行えるようにするためのTrait。
pub trait TemporalId:
    Debug + Display + Clone + Eq + Hash + Ord + PartialOrd + FromStr<Err = Error>
{
    /// 全時間を表す値。
    const WHOLE: Self;

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    fn is_whole(&self) -> bool;

    /// この値が表す絶対秒区間 `[start, end)` を返す。
    ///
    /// 形の異なる2つの具象型（生セル・人間向け範囲）を横断して比較・集約するための共通基盤。
    fn seconds_range(&self) -> (u64, u64);
}
