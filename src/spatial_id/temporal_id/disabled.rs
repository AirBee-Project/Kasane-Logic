//! `temporal_id` feature 無効時の [`TemporalId`] 本体定義。
//!
//! 演算・trait 実装・コレクション型は以下の各スタブに分散している。
//! 有効時のファイル構造と 1:1 で対応するため、追加・変更があれば有効時の
//! 対応ファイルを参照すること。
//!
//! | 役割               | 有効時                         | 無効時（スタブ）                          |
//! |--------------------|-------------------------------|------------------------------------------|
//! | `TemporalId` 本体  | `temporal_id/mod.rs`          | `temporal_id/disabled.rs`（本ファイル）   |
//! | `Interval` 型      | `temporal_id/interval/mod.rs` | `temporal_id/interval/disabled.rs`       |
//! | 演算               | `temporal_id/ops.rs`          | `temporal_id/ops/disabled.rs`            |
//! | trait 実装         | `temporal_id/impls.rs`        | `temporal_id/impls/disabled.rs`          |
//! | `TemporalSet`      | `temporal_id/collection/set/` | `temporal_id/collection/set/disabled.rs` |
//! | `TemporalMap`      | `temporal_id/collection/map/` | `temporal_id/collection/map/disabled.rs` |

use crate::{Interval, error::Error};

/// 時間IDの区間表現を表す型（`temporal_id` feature 無効時のスタブ）。
///
/// `temporal_id` feature が無効な場合、[`TemporalId`] は常に全時間を表す
/// スタブ実装となる。すべてのメソッドは全時間を表す状態を返す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TemporalId;

impl TemporalId {
    /// 全時間を表す定数。
    pub const WHOLE: TemporalId = TemporalId;

    /// 新しい [`TemporalId`] を作成する。
    ///
    /// `temporal_id` feature 無効時は [`Interval::WHOLE_SECONDS`] のみ受け付ける。
    pub fn new<I>(_interval: I, _t: u64) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
    {
        let _ = _interval.try_into()?;
        Ok(Self::WHOLE)
    }

    /// このインスタンスが全時間を表すかを判定する。
    ///
    /// `temporal_id` feature 無効時は常に `true` を返す。
    pub fn is_whole(&self) -> bool {
        true
    }

    /// 時間区間の開始時刻をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature 無効時は常に `0` を返す。
    pub fn start_unixtime(&self) -> u64 {
        0
    }

    /// 時間区間の終了時刻（排他的）をUNIXタイムスタンプで取得する。
    ///
    /// `temporal_id` feature 無効時は常に時間ドメイン終端 [`Interval::WHOLE_SECONDS`] を返す。
    pub fn end_unixtime_exclusive(&self) -> u64 {
        Interval::WHOLE_SECONDS
    }

    /// 時間間隔を取得する。
    ///
    /// `temporal_id` feature 無効時は常に [`Interval::Whole`] を返す。
    pub fn i(&self) -> Interval {
        Interval::Whole
    }

    /// 時間インデックス `t` を取得する。
    ///
    /// `temporal_id` feature 無効時は常に `0` を返す。
    pub fn t(&self) -> u64 {
        0
    }

    /// 開始と終了の UNIX タイムスタンプから [`TemporalId`] のイテレータを生成する。
    ///
    /// `temporal_id` feature 無効時は有効な範囲なら `WHOLE` を 1 つ返す。
    pub fn from_range(
        range: core::ops::Range<u64>,
    ) -> Result<impl Iterator<Item = TemporalId>, Error> {
        let mut yielded = false;
        let empty = range.start >= range.end;
        Ok(core::iter::from_fn(move || {
            if empty || yielded {
                return None;
            }
            yielded = true;
            Some(TemporalId::WHOLE)
        }))
    }

    /// 開始と終了のUNIXタイムスタンプから、時間範囲を表す [`TemporalId`] の個数を返す。
    pub(crate) fn count_range(range: core::ops::Range<u64>) -> usize {
        Self::from_range(range).unwrap().count()
    }
}
