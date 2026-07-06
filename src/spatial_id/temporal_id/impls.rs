use alloc::string::ToString;
use alloc::vec::Vec;

use core::{fmt::Display, str::FromStr};

use crate::{SpatialIdError, TemporalId, error::Error};

impl Display for TemporalId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/", self.i().seconds())?;
        write!(f, "{}", self.t)?;
        Ok(())
    }
}

impl Default for TemporalId {
    fn default() -> Self {
        Self::WHOLE
    }
}

/// 文字列表現から [`TemporalId`] を復元する。
///
/// `"i/t"` 形式の文字列をパースして [`TemporalId`] を構築する。
/// `i` は約数鎖（[`Interval`](crate::Interval)）に含まれる秒数である必要があり、
/// `t` は任意の `u64` 値である。
///
/// # パラメーター
///
/// 入力文字列は `"i/t"` の形式である必要がある。
/// - `i` — 時間間隔（10進数表記）
/// - `t` — 時間インデックス（10進数表記）
///
/// # エラー
///
/// 以下の場合に [`Error`] を返す：
/// - 区切り文字 `/` が見つからない
/// - `i` または `t` が有効な `u64` に変換できない
/// - [`TemporalId::new`] による検証に失敗した場合
///
/// # 例
///
/// 有効な文字列のパース:
/// ```
/// # #[cfg(feature = "temporal_id")]
/// # {
/// # use kasane_logic::TemporalId;
/// # use core::str::FromStr;
/// let id = TemporalId::from_seconds(3600, 5).unwrap();
/// let parsed: TemporalId = "3600/5".parse().unwrap();
/// assert_eq!(id, parsed);
/// # }
/// ```
///
/// Display と FromStr の往復:
/// ```
/// # #[cfg(feature = "temporal_id")]
/// # {
/// # use kasane_logic::TemporalId;
/// # use core::str::FromStr;
/// let original = TemporalId::from_seconds(60, 120).unwrap();
/// let string_repr = original.to_string();
/// let parsed = TemporalId::from_str(&string_repr).unwrap();
/// assert_eq!(original, parsed);
/// # }
/// ```
impl FromStr for TemporalId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            }
            .into());
        }

        let i = parts[0]
            .parse::<u64>()
            .map_err(|_| SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            })?;

        let t = parts[1]
            .parse::<u64>()
            .map_err(|_| SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            })?;

        TemporalId::from_seconds(i, t)
    }
}
