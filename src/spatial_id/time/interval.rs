//! 時間間隔 `{i}`。**公開 API に出てくる唯一の時間の型**である。
//!
//! 「いつか」を表す `{t}` は空間 ID 側（[`SingleId`](crate::SingleId) /
//! [`RangeId`](crate::RangeId) / [`FlexId`](crate::FlexId)）がフィールドとして直接持ち、
//! `interval()` / `t()` / `seconds_range()` で読み書きする。

use crate::{SpatialIdError, error::Error};

/// 時間 ID の時間間隔 `{i}`（秒数）を表現する型。
///
/// [Ouranos 4D 時空間ID仕様](https://github.com/AirBee-Project)の Temporal ID は
/// 「任意の秒数」を時間間隔として許容するため、`1..=`[`MAX_SECONDS`](Self::MAX_SECONDS)
/// の範囲であれば任意の値を保持できる（`Day`/`Hour` のような固定候補への限定はしない）。
///
/// よく使う値は関連定数として用意している。
///
/// | 定数 | 秒数 |
/// |---|---|
/// | [`WHOLE`](Self::WHOLE) | `2^35`（約1089年） |
/// | [`DAY`](Self::DAY) | 86400 |
/// | [`HOUR`](Self::HOUR) | 3600 |
/// | [`MINUTE`](Self::MINUTE) | 60 |
/// | [`SECOND`](Self::SECOND) | 1 |
///
/// # `temporal_id` feature 無効時
///
/// 中身の無いスタブ（サイズ0）になり、[`WHOLE`](Self::WHOLE) 以外は構築できない。
/// 各 ID が持つ `interval` フィールドは1バイトも消費しない。
#[cfg(feature = "temporal_id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[repr(transparent)]
pub struct Interval(u64);

/// `temporal_id` feature 無効時の [`Interval`]。常に全時間を表すサイズ0のスタブ。
#[cfg(not(feature = "temporal_id"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Interval;

impl Interval {
    /// このライブラリが扱える最大の時間間隔＝全時間の秒数
    /// （`2^35` 秒 = 34,359,738,368 秒、約1089年）。[`new`](Self::new) はこれを超える値を拒む。
    ///
    /// 起点は仕様どおり 1970-01-01 00:00 UTC なので、**3058年ごろまで**を表せる。
    ///
    /// [`WHOLE`](Self::WHOLE)`.seconds()` と同じ値である。名前に `WHOLE` を使っていないのは、
    /// 日常的に使うのは [`WHOLE`](Self::WHOLE) のほうで、接頭辞を共有すると
    /// エディタの補完候補で埋もれてしまうからである。
    pub const MAX_SECONDS: u64 = 1u64 << Self::MAX_POW;

    /// 最も粗い時間区間を表す二進層の指数。時間軸の最大ズームレベルでもある
    /// （[`TZoomLevel`](crate::spatial_id::zoom_level::TZoomLevel) の `LIMIT`。
    /// 両者が食い違わないことは `zoom_level.rs` の `const` アサーションが保証する）。
    ///
    /// # なぜ 35 なのか
    ///
    /// 時間軸は Unix 元期を起点とする2進トライで、最深ズームの1セルが1秒である。
    /// したがって `MAX_POW` はそのままトライの深さになり、現実の時刻がどれだけ深い
    /// ところに沈むかを決める。
    ///
    /// | `MAX_POW` | 表せる範囲 | 深さ |
    /// |---|---|---|
    /// | 30（空間軸と同じ） | 2004年まで | 30 |
    /// | 31 | 2038年まで | 31 |
    /// | 32 | 2106年まで | 32 |
    /// | **35** | **3058年まで** | **35** |
    /// | 62 | 1461億年後まで | 62 |
    ///
    /// 空間軸の最大ズーム（30）に合わせると2004年で尽きるため揃えられない。実データの
    /// 射程（気候予測などは2100年以降も扱う）を考えると32も狭い。35なら1000年以上の
    /// 余裕があり、深さは62の場合の56%で済む。
    pub const MAX_POW: u8 = 35;

    /// 全時間（`2^35` 秒）。時間を指定していない ID はこの値を持つ。
    #[cfg(feature = "temporal_id")]
    pub const WHOLE: Interval = Interval(Self::MAX_SECONDS);
    /// 全時間。`temporal_id` feature 無効時に唯一有効な値。
    #[cfg(not(feature = "temporal_id"))]
    pub const WHOLE: Interval = Interval;

    /// 1日（86400秒）。
    #[cfg(feature = "temporal_id")]
    pub const DAY: Interval = Interval(86_400);
    /// 1時間（3600秒）。
    #[cfg(feature = "temporal_id")]
    pub const HOUR: Interval = Interval(3_600);
    /// 1分（60秒）。
    #[cfg(feature = "temporal_id")]
    pub const MINUTE: Interval = Interval(60);
    /// 1秒。
    #[cfg(feature = "temporal_id")]
    pub const SECOND: Interval = Interval(1);

    /// 秒数から [`Interval`] を作成する。
    ///
    /// # バリデーション
    /// - `seconds` が `0`、または [`MAX_SECONDS`](Self::MAX_SECONDS) を超える場合は
    ///   [`SpatialIdError::TIntervalError`] を返す。
    /// - `temporal_id` feature 無効時は [`MAX_SECONDS`](Self::MAX_SECONDS) 以外を拒否する。
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// assert_eq!(Interval::new(3600).unwrap(), Interval::HOUR);
    /// // 仕様どおり、候補以外の秒数も許容する。
    /// assert_eq!(Interval::new(1800).unwrap().seconds(), 1800);
    /// assert!(Interval::new(0).is_err());
    /// # }
    /// ```
    pub const fn new(seconds: u64) -> Result<Interval, Error> {
        #[cfg(feature = "temporal_id")]
        {
            if seconds == 0 || seconds > Self::MAX_SECONDS {
                return Err(Error::SpatialId(SpatialIdError::TIntervalError {
                    i: seconds,
                }));
            }
            Ok(Interval(seconds))
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            if seconds == Self::MAX_SECONDS {
                Ok(Interval)
            } else {
                Err(Error::SpatialId(SpatialIdError::TIntervalError {
                    i: seconds,
                }))
            }
        }
    }

    /// この [`Interval`] の秒数。
    pub const fn seconds(self) -> u64 {
        #[cfg(feature = "temporal_id")]
        {
            self.0
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            Self::MAX_SECONDS
        }
    }

    /// この単位で、Unix 時刻（秒）が属するセルのインデックス `{t}` を返す。
    ///
    /// 仕様書 1.5.3 (3) の `t = floor(u / i)`。
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// assert_eq!(Interval::HOUR.index_of(7_300), 2);
    /// assert_eq!(Interval::new(1800).unwrap().index_of(1_457_482_000), 809_712);
    /// # }
    /// ```
    pub const fn index_of(self, unix_seconds: u64) -> u64 {
        unix_seconds / self.seconds()
    }

    /// 検証済みの秒数から直接構築する。クレート内部専用。
    ///
    /// 呼び出し側は `1 <= seconds <= `[`MAX_SECONDS`](Self::MAX_SECONDS) を保証すること。
    /// 絶対秒区間からの復元（[`coarsest_unit`](super::cells::coarsest_unit) の結果）の
    /// ように、構成上その範囲に収まることが証明できる経路でのみ使う。
    pub(crate) const fn from_seconds_unchecked(seconds: u64) -> Interval {
        #[cfg(feature = "temporal_id")]
        {
            Interval(seconds)
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            debug_assert!(seconds == Self::MAX_SECONDS);
            Interval
        }
    }
}

impl core::fmt::Display for Interval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.seconds())
    }
}

impl From<Interval> for i64 {
    fn from(interval: Interval) -> i64 {
        interval.seconds() as i64
    }
}

impl TryFrom<u64> for Interval {
    type Error = Error;

    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        Interval::new(seconds)
    }
}

/// `u64` へ収まらない値は**必ず失敗させる**。`as u64` で丸めると、`u128` の
/// `2^64 + 3600` や `i128` の負値が `Ok(Interval::HOUR)` として通ってしまう。
macro_rules! impl_try_from_integer {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for Interval {
                type Error = Error;

                fn try_from(seconds: $t) -> Result<Self, Self::Error> {
                    let seconds = u64::try_from(seconds)
                        .map_err(|_| SpatialIdError::TIntervalError { i: 0 })?;
                    Interval::new(seconds)
                }
            }
        )*
    };
}

impl_try_from_integer!(u8, u16, u32, u128, usize, i8, i16, i32, i64, i128, isize);

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range_seconds() {
        assert!(Interval::new(0).is_err());
        assert!(Interval::new(Interval::MAX_SECONDS + 1).is_err());
        assert!(Interval::new(Interval::MAX_SECONDS).is_ok());
    }

    /// `as u64` による丸めを許すと、範囲外の値が別の有効な `Interval` に化けてしまう。
    #[test]
    fn try_from_wide_integers_never_truncates() {
        // 2^64 + 3600。`as u64` だと 3600（＝1時間）になっていた。
        assert!(Interval::try_from((1u128 << 64) + 3600).is_err());
        // 負の巨大値。`as u64` だと同じく 3600 として通っていた。
        assert!(Interval::try_from(-((1i128 << 64) - 3600)).is_err());

        assert!(Interval::try_from(-1i32).is_err());
        assert!(Interval::try_from(-1i64).is_err());
        assert!(Interval::try_from(u128::MAX).is_err());

        // 範囲内は従来どおり通る。`i64` は実装漏れしていた。
        assert_eq!(Interval::try_from(3600i64).unwrap(), Interval::HOUR);
        assert_eq!(Interval::try_from(3600u32).unwrap(), Interval::HOUR);
        assert_eq!(Interval::try_from(3600i128).unwrap(), Interval::HOUR);
    }

    #[test]
    fn index_of_follows_the_spec_formula() {
        // t = floor(u / i)
        for (i, u, expected) in [
            (3600u64, 7_300u64, 2u64),
            (1800, 1_457_482_000, 809_712),
            (1, 1_770_000_000, 1_770_000_000),
            (86_400, 86_399, 0),
        ] {
            assert_eq!(Interval::new(i).unwrap().index_of(u), expected);
        }
    }
}

/// 3型（[`SingleId`](crate::SingleId) / [`RangeId`](crate::RangeId) / [`FlexId`](crate::FlexId)）
/// で、時間 API の名前・引数・戻り値の形が揃っていることを固定する。
///
/// 型ごとに「表せる時間の形」は違う（1セル / 範囲 / 2進セル1個）が、**入口の名前と失敗の
/// 仕方は同じ**であるべき。片方だけに生えた補助コンストラクタや、`Option` と `Result` の
/// 混在を防ぐための回帰テストである。
#[cfg(all(test, feature = "temporal_id"))]
mod api_symmetry {
    use crate::{FlexId, Interval, RangeId, SingleId, SpatialId};

    /// `with_time` は3型とも同じ名前・同じ `Result` で、`{i}/{t}` を受ける。
    #[test]
    fn with_time_is_available_on_every_type() {
        let single = SingleId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(1024, 7)
            .unwrap();
        let range = RangeId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(1024, 7)
            .unwrap();
        // FlexId だけは母語（ズーム）で指定する。ズーム25 = 2^(35-25) = 1024 秒幅。
        let flex = FlexId::new(4, 0, 4, 1, 4, 1)
            .unwrap()
            .with_time(25, 7)
            .unwrap();

        // 同じ `{i}/{t}` を指定したので、占める秒区間も一致する。
        let expected = (1024 * 7, 1024 * 8);
        assert_eq!(single.seconds_range(), expected);
        assert_eq!(range.seconds_range(), expected);
        assert_eq!(flex.seconds_range(), expected);
    }

    /// `interval` 引数は `Interval` でも整数リテラルでも `u64` でも通る。
    #[test]
    fn interval_argument_accepts_every_spelling() {
        let by_const = SingleId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(Interval::HOUR, 3)
            .unwrap();
        let by_literal = SingleId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(3600, 3)
            .unwrap();
        let by_u64 = SingleId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(3600u64, 3)
            .unwrap();

        assert_eq!(by_const, by_literal);
        assert_eq!(by_const, by_u64);
    }

    /// `with_time_at` は3型とも同じ名前で、Unix 時刻からセルを決める。
    #[test]
    fn with_time_at_is_available_on_every_type() {
        const UNIX: u64 = 1_770_000_000;
        let single = SingleId::new(4, 0, 1, 1)
            .unwrap()
            .with_time_at(1024, UNIX)
            .unwrap();
        let range = RangeId::new(4, 0, 1, 1)
            .unwrap()
            .with_time_at(1024, UNIX)
            .unwrap();
        let flex = FlexId::new(4, 0, 4, 1, 4, 1)
            .unwrap()
            .with_time_at(25, UNIX)
            .unwrap();

        assert_eq!(single.seconds_range(), range.seconds_range());
        assert_eq!(single.seconds_range(), flex.seconds_range());
        assert!(single.seconds_range().0 <= UNIX && UNIX < single.seconds_range().1);
    }

    /// `with_time_span` は3型とも同じ名前で、表せない区間は `Err`。
    #[test]
    fn with_time_span_is_available_on_every_type() {
        // [1024, 2048) は3型とも表せる（2の冪・整列）。
        for got in [
            SingleId::new(4, 0, 1, 1)
                .unwrap()
                .with_time_span(1024, 2048)
                .map(|v| v.seconds_range()),
            RangeId::new(4, 0, 1, 1)
                .unwrap()
                .with_time_span(1024, 2048)
                .map(|v| v.seconds_range()),
            FlexId::new(4, 0, 4, 1, 4, 1)
                .unwrap()
                .with_time_span(1024, 2048)
                .map(|v| v.seconds_range()),
        ] {
            assert_eq!(got.unwrap(), (1024, 2048));
        }

        // [1800, 7200) は 1800 秒 × 3 セルなので、範囲でしか表せない。
        assert!(
            SingleId::new(4, 0, 1, 1)
                .unwrap()
                .with_time_span(1800, 7200)
                .is_err()
        );
        assert!(
            FlexId::new(4, 0, 4, 1, 4, 1)
                .unwrap()
                .with_time_span(1800, 7200)
                .is_err()
        );
        let range = RangeId::new(4, 0, 1, 1)
            .unwrap()
            .with_time_span(1800, 7200)
            .unwrap();
        assert_eq!(range.seconds_range(), (1800, 7200));
        assert_eq!((range.interval().seconds(), range.t()), (1800, [1, 3]));
    }

    /// `relabel_time` は3型とも `Result`（`Option` との混在を許さない）。
    #[test]
    fn relabel_time_returns_result_on_every_type() {
        let single = SingleId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(3600, 5)
            .unwrap();
        assert_eq!(single.clone().relabel_time(Interval::HOUR).unwrap(), single);
        assert!(single.relabel_time(1800).is_err());

        let range = RangeId::new(4, 0, 1, 1)
            .unwrap()
            .with_time(3600, [0, 1])
            .unwrap();
        assert_eq!(range.clone().relabel_time(1800).unwrap().t(), [0, 3]);
        assert!(range.relabel_time(Interval::DAY).is_err());

        // `FlexId` には `relabel_time` を持たせない。セルが秒区間を一意に決めるので、
        // 「同じ区間を別の単位で」は定義上ありえず、成功しても必ず元と同じ値になる
        // （常に no-op か Err にしかならないAPIは、対称性のためだけに置く価値がない）。
    }

    /// `without_time` は3型とも同じ名前で、全時間へ戻す。
    #[test]
    fn without_time_is_available_on_every_type() {
        assert!(
            SingleId::new(4, 0, 1, 1)
                .unwrap()
                .with_time(1024, 7)
                .unwrap()
                .without_time()
                .is_whole_time()
        );
        assert!(
            RangeId::new(4, 0, 1, 1)
                .unwrap()
                .with_time(1024, [7, 9])
                .unwrap()
                .without_time()
                .is_whole_time()
        );
        assert!(
            FlexId::new(4, 0, 4, 1, 4, 1)
                .unwrap()
                .with_time(25, 7)
                .unwrap()
                .without_time()
                .is_whole_time()
        );
    }

    /// `new` は3型とも空間だけを受け、時間は必ず全時間から始まる。
    #[test]
    fn new_always_starts_from_whole_time() {
        assert!(SingleId::new(4, 0, 1, 1).unwrap().is_whole_time());
        assert!(RangeId::new(4, 0, 1, 1).unwrap().is_whole_time());
        assert!(FlexId::new(4, 0, 4, 1, 4, 1).unwrap().is_whole_time());
    }
}
