pub mod constructor;
pub mod convert;
pub mod impls;
pub mod random;
#[cfg(test)]
mod test;

use crate::SpatialId;
use crate::{
    Interval, SpatialIdError,
    error::Error,
    spatial_id::{helpers, time::span, zoom_level::ZoomLevel},
};

/// RangeIdは空間IDの範囲表現を表す型です。
///
/// 各インデックスを範囲で指定することができます。各次元の範囲を表す配列の順序には意味を持ちません。内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装しているが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// # use kasane_logic::SpatialId;
/// # use kasane_logic::ZoomLevel;
/// pub struct RangeId {
///     z: ZoomLevel,
///     f: [i32; 2],
///     x: [u32; 2],
///     y: [u32; 2],
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RangeId {
    z: ZoomLevel,
    f: [i32; 2],
    x: [u32; 2],
    y: [u32; 2],
    i: Interval,
    #[cfg(feature = "temporal_id")]
    t: [u64; 2],
}

impl RangeId {
    /// この `RangeId` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// ```
    pub fn z(&self) -> u8 {
        self.z.get()
    }

    /// この `RangeId` が保持しているズームレベル `[f1,f2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.f(), [-3i32,29i32]);
    /// ```
    pub fn f(&self) -> [i32; 2] {
        self.f
    }

    /// この `RangeId` が保持しているズームレベル `[x1,x2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.x(), [8u32,9u32]);
    /// ```
    pub fn x(&self) -> [u32; 2] {
        self.x
    }

    /// この `RangeId` が保持しているズームレベル `[y1,y2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.y(), [5u32,10u32]);
    /// ```
    pub fn y(&self) -> [u32; 2] {
        self.y
    }

    /// この `RangeId` の時間インデックス範囲 `[min, max]`（両端含む）を返します。
    ///
    /// `temporal_id` feature 無効時は常に `[0, 0]`（全時間の唯一のSegment）。
    pub fn t(&self) -> [u64; 2] {
        #[cfg(feature = "temporal_id")]
        {
            self.t
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            [0, 0]
        }
    }

    /// 時間を設定した自身を返します（ビルダー形式）。
    ///
    /// [`SingleId::with_time`](crate::SingleId::with_time)が単一Segmentしか受け取らないのに対し、
    /// こちらは空間と同じく**範囲**を受け取る。`t` は `[min, max]` の `[u64; 2]`、または
    /// 両端が等しい単一の `u64` のどちらでも渡せる（f/x/y 引数と同じ考え方）。
    ///
    /// FlexTreeが必要とする2の冪秒のSegmentへの分解は、木へ挿入する段階
    /// （[`IntoIterator`]による[`FlexId`](crate::FlexId)への展開）で自動的に行われる。
    ///
    ///
    /// ## 動作コスト
    ///
    /// 木構造（`SpatialIdSet` など）へ挿入する際、時間間隔が2の冪秒であれば必ず1つの時間Segment
    /// （[`FlexId`](crate::FlexId)）に収まります。しかし非2冪の秒数（例: 1800秒）を指定した場合、
    /// 内部で最大十数個のSegmentに分解されるため、**挿入コストと木の葉の数が比例して増大**します。
    /// パフォーマンスが重視される用途では、1800秒や3600秒の代わりに、**1024秒**や**4096秒**
    /// などの2の冪秒を利用することを強く推奨します（挿入処理が約10倍高速化されます）。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, RangeId};
    /// let id = RangeId::new(4, [-3, 6], [8, 9], [5, 10]).unwrap()
    ///     .with_time(Interval::HOUR, [5, 8]).unwrap();
    /// assert_eq!(id.to_string(), "4/-3:6/8:9/5:10_3600/5:8");
    ///
    /// // 単一の時刻も渡せる。
    /// let single = RangeId::new(4, [-3, 6], [8, 9], [5, 10]).unwrap()
    ///     .with_time(Interval::HOUR, 5).unwrap();
    /// assert_eq!(single.to_string(), "4/-3:6/8:9/5:10_3600/5");
    /// # }
    /// ```
    pub fn with_time<I, T>(self, interval: I, t: T) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
        T: helpers::IntoRange<u64>,
    {
        let interval: Interval = interval.try_into()?;
        let mut t = t.into_range();
        if t[0] > t[1] {
            t.swap(0, 1);
        }

        interval.validated_span(t[0], t[1])?;
        Ok(self.with_time_unchecked(interval, t))
    }

    /// Unix 時刻（秒）が属する時間Segmentを設定した自身を返します。
    ///
    /// 仕様書 1.5.3 (3) の `t = floor(u / i)` をこちらで計算するので、呼び出し側で
    /// インデックスを求める必要がない。範囲ではなく単一Segmentになる。
    ///
    /// ## 動作コスト
    ///
    /// 木構造（`SpatialIdSet` など）へ挿入する際、時間間隔が2の冪秒であれば必ず1つの時間Segment
    /// （[`FlexId`](crate::FlexId)）に収まります。しかし非2冪の秒数（例: 1800秒）を指定した場合、
    /// 内部で最大十数個のSegmentに分解されるため、**挿入コストと木の葉の数が比例して増大**します。
    /// パフォーマンスが重視される用途では、1800秒や3600秒の代わりに、**1024秒**や**4096秒**
    /// などの2の冪秒を利用することを強く推奨します（挿入処理が約10倍高速化されます）。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, 0, 0, 0).unwrap().with_time_at(1800, 1_457_482_000).unwrap();
    /// assert_eq!(id.t(), [809712, 809712]);
    /// # }
    /// ```
    pub fn with_time_at<I>(self, interval: I, unix_seconds: u64) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
    {
        let interval: Interval = interval.try_into()?;
        self.with_time::<Interval, u64>(interval, interval.index_of(unix_seconds))
    }

    /// 絶対秒区間 `[start, end)` を設定した自身を返します（1970-01-01 00:00 UTC 起点）。
    ///
    /// 単位は「その区間をちょうど表せる最も粗い秒数」が自動で選ばれる
    /// （`start` と区間幅の最大公約数）。仕様が認める任意秒数の単位もそのまま出てくる。
    /// `RangeId` は時間も範囲で持てるので、**任意の区間を受け付ける**
    /// （単一Segmentに限る [`SingleId::with_time_span`](crate::SingleId::with_time_span) との違い）。
    ///
    /// ## 動作コスト
    ///
    /// 木構造（`SpatialIdSet` など）へ挿入する際、区間幅が2の冪秒であれば必ず1つの時間Segment
    /// （[`FlexId`](crate::FlexId)）に収まります。しかし非2冪の幅（例: 1800秒）が選ばれた場合、
    /// 内部で最大十数個のSegmentに分解されるため、**挿入コストと木の葉の数が比例して増大**します。
    /// パフォーマンスが重視される用途では、区間幅が1800秒や3600秒になる代わりに、**1024秒**や**4096秒**
    /// などの2の冪秒になるように指定することを強く推奨します（挿入処理が約10倍高速化されます）。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::RangeId;
    /// // [1457481600, 1457483400) はちょうど 1800 秒ぶん。
    /// let id = RangeId::new(4, 0, 0, 0).unwrap()
    ///     .with_time_span(1_457_481_600, 1_457_483_400).unwrap();
    /// assert_eq!(id.time_interval().seconds(), 1800);
    /// assert_eq!(id.t(), [809712, 809712]);
    /// # }
    /// ```
    pub fn with_time_span(self, start: u64, end: u64) -> Result<Self, Error> {
        let time_span =
            span::TimeSpan::new(start, end).ok_or(SpatialIdError::TOutOfRange { i: 1, t: end })?;
        let (interval, t_min, t_max) = time_span.to_interval_range()?;
        Ok(self.with_time_unchecked(interval, [t_min, t_max]))
    }

    /// 同じ絶対秒区間を、別の単位で表し直した自身を返します。
    ///
    /// 単位が変わっても指す区間は変わらない。その単位で割り切れない場合は
    /// [`SpatialIdError::TIntervalError`] を返す。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, RangeId};
    /// let id = RangeId::new(4, 0, 0, 0).unwrap().with_time(Interval::HOUR, [0, 1]).unwrap();
    /// assert_eq!(id.clone().relabel_time(1800).unwrap().t(), [0, 3]);
    /// // 2時間ぶんは1日単位では表せない。
    /// assert!(id.relabel_time(Interval::DAY).is_err());
    /// # }
    /// ```
    pub fn relabel_time<I>(self, interval: I) -> Result<Self, Error>
    where
        I: TryInto<Interval>,
        Error: From<I::Error>,
    {
        let interval: Interval = interval.try_into()?;
        let (start, end) = self.seconds_range();
        let unit = interval.seconds();
        if !start.is_multiple_of(unit) || !end.is_multiple_of(unit) {
            return Err(SpatialIdError::TIntervalError { i: unit }.into());
        }
        Ok(self.with_time_unchecked(interval, [start / unit, end / unit - 1]))
    }

    /// 時間の指定を外し、全時間へ戻した自身を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, RangeId, SpatialId};
    /// let id = RangeId::new(4, 0, 0, 0).unwrap().with_time(Interval::HOUR, [0, 1]).unwrap();
    /// assert!(id.without_time().is_whole_time());
    /// # }
    /// ```
    pub fn without_time(self) -> Self {
        self.with_time_unchecked(Interval::WHOLE, [0, 0])
    }

    /// 検証済みの `{i}` / `{t}` を書き込む。
    fn with_time_unchecked(
        #[cfg_attr(not(feature = "temporal_id"), allow(unused_mut))] mut self,
        interval: Interval,
        t: [u64; 2],
    ) -> Self {
        self.i = interval;
        // `temporal_id` 無効時に検証を通るのは全時間だけで、そのとき `t` は `[0, 0]` に
        // しかならないので情報は落ちない。
        #[cfg(feature = "temporal_id")]
        {
            self.t = t;
        }
        #[cfg(not(feature = "temporal_id"))]
        debug_assert_eq!(t, [0, 0], "temporal_id 無効時に t != [0,0] が検証を通った");
        self
    }

    /// この `RangeId` の時間を、FlexTree が使う2分岐Segmentの列へ分解する。クレート内部専用。
    pub(crate) fn time_segments(&self) -> span::TimeSegments {
        self.time_span().into_segments()
    }

    pub fn set_f(&mut self, value: [i32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let mut value = value;
        let f_min = ZoomLevel::new(z).unwrap().f_min();
        let f_max = ZoomLevel::new(z).unwrap().f_max();

        for &f_value in &value {
            if f_value < f_min || f_value > f_max {
                return Err(SpatialIdError::FOutOfRange { f: f_value, z }.into());
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.f = value;
        Ok(())
    }

    pub fn set_x(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let xy_max = ZoomLevel::new(z).unwrap().xy_max();

        for &x_value in &value {
            if x_value > xy_max {
                return Err(SpatialIdError::XOutOfRange { x: x_value, z }.into());
            }
        }

        self.x = value;
        Ok(())
    }

    pub fn set_y(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let mut value = value;
        let xy_max = ZoomLevel::new(z).unwrap().xy_max();

        for &y_value in &value {
            if y_value > xy_max {
                return Err(SpatialIdError::YOutOfRange { y: y_value, z }.into());
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.y = value;
        Ok(())
    }

    /// x 軸範囲を、より細かいズームレベル `target_z` における生のインデックス範囲へ変換します。
    ///
    /// `self.x()` が折り返し（`x[0] > x[1]`）の場合、この関数はそれを解決せず、
    /// 両端をそれぞれ独立に拡大した `(min, max)`（`min > max` のまま）を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, 0, [1, 2], 0).unwrap();
    /// assert_eq!(id.x_fine_range(6), (4, 11));
    /// ```
    pub fn x_fine_range(&self, target_z: impl Into<u8>) -> (u32, u32) {
        let (min, max) = Self::fine_range_raw(
            self.x[0] as i64,
            self.x[1] as i64,
            self.z.get(),
            target_z.into(),
        );
        (min as u32, max as u32)
    }

    /// y 軸範囲を、より細かいズームレベル `target_z` における生のインデックス範囲へ変換します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, 0, 0, [1, 2]).unwrap();
    /// assert_eq!(id.y_fine_range(6), (4, 11));
    /// ```
    pub fn y_fine_range(&self, target_z: impl Into<u8>) -> (u32, u32) {
        let (min, max) = Self::fine_range_raw(
            self.y[0] as i64,
            self.y[1] as i64,
            self.z.get(),
            target_z.into(),
        );
        (min as u32, max as u32)
    }

    /// f 軸範囲を、より細かいズームレベル `target_z` における生のインデックス範囲へ変換します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, [1, 2], 0, 0).unwrap();
    /// assert_eq!(id.f_fine_range(6), (4, 11));
    /// ```
    pub fn f_fine_range(&self, target_z: impl Into<u8>) -> (i32, i32) {
        let (min, max) = Self::fine_range_raw(
            self.f[0] as i64,
            self.f[1] as i64,
            self.z.get(),
            target_z.into(),
        );
        (min as i32, max as i32)
    }

    /// x/y/f_fine_range が共有する拡大計算の本体。
    fn fine_range_raw(lo: i64, hi: i64, z: u8, fine_z: u8) -> (i64, i64) {
        debug_assert!(fine_z >= z);
        let scale = 1i64 << (fine_z - z);
        (lo * scale, (hi + 1) * scale - 1)
    }

    /// x 軸の区間の両端に、ズーム `target_z` において `shift_min`/`shift_max` だけずらした `RangeId` を返します。`target_z` が `self.z()` 未満の場合はエラーになります。
    /// x軸は周期境界を持つため、結果を1本の `RangeId` で正確に表現できなくても安全側にフォールバックし、常に範囲を返します。
    ///
    /// 平行移動（両端に同じ量を加算）:
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(5, 0, [10, 12], 0).unwrap();
    /// let result = id.x_edges_shift(5, -2, -2).unwrap().unwrap();
    /// assert_eq!(result.x(), [8, 10]);
    /// ```
    ///
    /// 拡張（両端に異なる量を加算）:
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(5, 0, [10, 10], 0).unwrap();
    /// let result = id.x_edges_shift(5, -1, 1).unwrap().unwrap();
    /// assert_eq!(result.x(), [9, 11]);
    /// ```
    pub fn x_edges_shift(
        &self,
        target_z: impl Into<u8>,
        shift_min: i64,
        shift_max: i64,
    ) -> Result<Option<RangeId>, Error> {
        let target_z = target_z.into();
        let current_z = self.z.get();
        if target_z < current_z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z,
                target_z,
            }
            .into());
        }
        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let (x_min, x_max) = self.x_fine_range(target_z);
        let x_min = x_min as i64;
        let mut x_max = x_max as i64;
        if self.x[0] > self.x[1] {
            // self.x 自体が折り返している場合、x_fine_range は末尾側をそのまま返すため
            // min > max のままになる。もう1周したぶんを補って正しい大きさを保つ。
            x_max += 1i64 << target_z;
        }

        let mut result = self.clone();
        result.set_x_wrapped_coarsened(x_min + shift_min, x_max + shift_max, target_z);
        Ok(Some(result))
    }

    /// [`x_edges_shift`](Self::x_edges_shift)の押し閉じ部分。
    /// `raw_min`/`raw_max`（ズーム `max_z` での生の範囲）を折り返した上で `self.z()` へ
    /// 丸め込む。折り返しを1本の `RangeId` で正確に表現できなければ、取りこぼす（過小評価する）
    /// くらいなら安全側に倒して全域にする。呼び出し元が `max_z >= self.z()` を保証すること。
    pub(crate) fn set_x_wrapped_coarsened(&mut self, raw_min: i64, raw_max: i64, max_z: u8) {
        let current_z = self.z.get();
        debug_assert!(current_z <= max_z);

        let scale_t = max_z - current_z;
        let max_len = 1i64 << max_z;

        // 折り返し前の時点で既に全周以上を覆っている場合は全域になる。
        if raw_max - raw_min >= max_len {
            self.x = [0, self.z.xy_max()];
            return;
        }

        let wrapped_min = raw_min.rem_euclid(max_len);
        let wrapped_max = raw_max.rem_euclid(max_len);

        let Some([(a_min, a_max), (b_min, b_max)]) =
            Self::split_wrapped_range(wrapped_min, wrapped_max, max_len - 1)
        else {
            // 折り返しなし: 独立に丸め込んでも常に正確。
            self.x = [
                (wrapped_min >> scale_t) as u32,
                (wrapped_max >> scale_t) as u32,
            ];
            return;
        };

        // 折り返しあり: [a_min, a_max]（=[wrapped_min, max_len-1]）と
        // [b_min, b_max]（=[0, wrapped_max]）それぞれを丸め込んだ上で、その和集合を取る。
        let a_min = (a_min >> scale_t) as u32;
        let a_max = (a_max >> scale_t) as u32;
        let b_min = (b_min >> scale_t) as u32;
        let b_max = (b_max >> scale_t) as u32;

        if a_min > b_max {
            // 丸め込み後も隙間が残るので、1本の折り返し RangeId として正確に表現できる。
            self.x = [a_min, b_max];
        } else {
            // 丸め込みで隙間が消えた（＝実質全域）ので、安全側に倒して全域にする。
            self.x = [b_min, a_max];
        }
    }

    /// `min > max`（[`set_x`](Self::set_x)の折り返し規約）を、2つの非折り返し区間
    /// `[min, bound]`と`[0, max]`に分解する。折り返していなければ `None`。
    pub(crate) fn split_wrapped_range(min: i64, max: i64, bound: i64) -> Option<[(i64, i64); 2]> {
        (min > max).then_some([(min, bound), (0, max)])
    }

    /// y 軸の区間の両端に、ズーム `target_z` において `shift_min`/`shift_max` だけずらした`RangeId` を返します。`target_z` が `self.z()` 未満の場合はエラーになります。
    /// y軸は境界を持つため、押し閉じで範囲が押し潰れてなくなった場合は `Ok(None)` を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(5, 0, 0, [10, 10]).unwrap();
    /// let result = id.y_edges_shift(5, -1, 1).unwrap().unwrap();
    /// assert_eq!(result.y(), [9, 11]);
    /// ```
    ///
    /// 範囲が押し潰れてなくなる場合（`shift_min` 側と `shift_max` 側が逆転してしまう場合）は `Ok(None)`:
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(1, 0, 0, 0).unwrap();
    /// assert_eq!(id.y_edges_shift(1, 5, -5).unwrap(), None);
    /// ```
    pub fn y_edges_shift(
        &self,
        target_z: impl Into<u8>,
        shift_min: i64,
        shift_max: i64,
    ) -> Result<Option<RangeId>, Error> {
        let target_z = target_z.into();
        let current_z = self.z.get();
        if target_z < current_z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z,
                target_z,
            }
            .into());
        }
        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let (y_min, y_max) = self.y_fine_range(target_z);
        let mut result = self.clone();
        Ok(result
            .set_y_coarsened_clamped(y_min as i64 + shift_min, y_max as i64 + shift_max, target_z)
            .then_some(result))
    }

    /// [`y_edges_shift`](Self::y_edges_shift)の押し閉じ部分。
    /// `raw_min`/`raw_max`を`[0, max_len - 1]`にクランプしてから丸め込む。
    /// 呼び出し元が `max_z >= self.z()` を保証すること。
    pub(crate) fn set_y_coarsened_clamped(
        &mut self,
        raw_min: i64,
        raw_max: i64,
        max_z: u8,
    ) -> bool {
        let current_z = self.z.get();
        debug_assert!(current_z <= max_z);
        let scale_t = max_z - current_z;
        let max_len = 1i64 << max_z;

        match Self::coarsened_clamped_raw(raw_min, raw_max, 0, max_len - 1, scale_t) {
            Some((min, max)) => {
                self.y = [min as u32, max as u32];
                true
            }
            None => false,
        }
    }

    /// f 軸の区間の両端に、ズーム `target_z` において `shift_min`/`shift_max` だけずらした`RangeId` を返します。`target_z` が `self.z()` 未満の場合はエラーになります。
    /// f軸も境界を持つため、押し閉じで範囲が押し潰れてなくなった場合は `Ok(None)` を返します。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(5, [10, 10], 0, 0).unwrap();
    /// let result = id.f_edges_shift(5, -1, 1).unwrap().unwrap();
    /// assert_eq!(result.f(), [9, 11]);
    /// ```
    ///
    /// 範囲が押し潰れてなくなる場合（`shift_min` 側と `shift_max` 側が逆転してしまう場合）は `Ok(None)`:
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(1, 0, 0, 0).unwrap();
    /// assert_eq!(id.f_edges_shift(1, 10, -10).unwrap(), None);
    /// ```
    pub fn f_edges_shift(
        &self,
        target_z: impl Into<u8>,
        shift_min: i64,
        shift_max: i64,
    ) -> Result<Option<RangeId>, Error> {
        let target_z = target_z.into();
        let current_z = self.z.get();
        if target_z < current_z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z,
                target_z,
            }
            .into());
        }
        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let (f_min, f_max) = self.f_fine_range(target_z);
        let mut result = self.clone();
        Ok(result
            .set_f_coarsened_clamped(f_min as i64 + shift_min, f_max as i64 + shift_max, target_z)
            .then_some(result))
    }

    /// [`f_edges_shift`](Self::f_edges_shift)の押し閉じ部分。
    /// `raw_min`/`raw_max`を`[f_min, f_max]`にクランプしてから丸め込む。
    /// 呼び出し元が `max_z >= self.z()` を保証すること。
    pub(crate) fn set_f_coarsened_clamped(
        &mut self,
        raw_min: i64,
        raw_max: i64,
        max_z: u8,
    ) -> bool {
        let current_z = self.z.get();
        debug_assert!(current_z <= max_z);
        let scale_t = max_z - current_z;
        let max_z_obj = ZoomLevel::new(max_z).unwrap();
        let min_f = max_z_obj.f_min() as i64;
        let max_f = max_z_obj.f_max() as i64;

        match Self::coarsened_clamped_raw(raw_min, raw_max, min_f, max_f, scale_t) {
            Some((min, max)) => {
                self.f = [min as i32, max as i32];
                true
            }
            None => false,
        }
    }

    /// [`set_y_coarsened_clamped`](Self::set_y_coarsened_clamped)/
    /// [`set_f_coarsened_clamped`](Self::set_f_coarsened_clamped) が共有する
    /// クランプ＋丸め込みの本体。範囲が `[min_bound, max_bound]` の外に完全に出た場合は `None`。
    fn coarsened_clamped_raw(
        raw_min: i64,
        raw_max: i64,
        min_bound: i64,
        max_bound: i64,
        scale_t: u8,
    ) -> Option<(i64, i64)> {
        let clamped_min = raw_min.clamp(min_bound, max_bound);
        let clamped_max = raw_max.clamp(min_bound, max_bound);

        if clamped_min > clamped_max {
            None
        } else {
            Some((clamped_min >> scale_t, clamped_max >> scale_t))
        }
    }

    /// 指定したズームレベル `target_z` に細分化した、この `RangeId` を含むすべての子 `RangeId` を生成します。
    ///
    /// # パラメータ
    /// * `target_z` — 生成したい子 `RangeId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより浅い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段深いズームへの細分化
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children_at_zoom(6).unwrap();
    /// assert_eq!(result,  RangeId::new(6, [-6, 59], [16, 19], [10, 21] ).unwrap());
    ///
    /// ```
    ///
    /// 現在より浅いズームを指定した場合
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::{Error, RangeId, SpatialIdError};
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children_at_zoom(4);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 5, target_z: 4 }))));
    /// ```
    pub fn spatial_children_at_zoom(&self, target_z: u8) -> Result<RangeId, Error> {
        let z = self.z.get();
        if target_z < z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: z,
                target_z,
            }
            .into());
        }

        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let difference = target_z - z;
        let scale_f = 1_i32 << difference as u32;
        let scale_xy = 1_u32 << difference as u32;

        let f = helpers::scale_range_i32(self.f[0], self.f[1], scale_f);
        let x = helpers::scale_range_u32(self.x[0], self.x[1], scale_xy);
        let y = helpers::scale_range_u32(self.y[0], self.y[1], scale_xy);

        Ok(RangeId {
            z: ZoomLevel::new(target_z).unwrap(),
            f,
            x,
            y,

            i: self.i,
            #[cfg(feature = "temporal_id")]
            t: self.t,
        })
    }

    /// 指定したズームレベル `target_z` に縮約した、この `RangeId` の親 `RangeId` を返します。
    ///
    /// # パラメータ
    /// * `target_z` — 取得したい親 `RangeId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより深い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段浅いズームへの縮約
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [1,29], [8,9], [5,10]).unwrap();
    /// let parent = id.spatial_parent_at_zoom(4).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [0,14]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// Fが負の場合の挙動:
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let parent = id.spatial_parent_at_zoom(4).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [-5,-3]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// 現在より深いズームを指定した場合:
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::{Error, RangeId, SpatialIdError};
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_parent_at_zoom(6);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 5, target_z: 6 }))));
    /// ```
    pub fn spatial_parent_at_zoom(&self, target_z: u8) -> Result<RangeId, Error> {
        let z = self.z.get();
        if target_z > z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: z,
                target_z,
            }
            .into());
        }

        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let shift = (z - target_z) as u32;

        let f = [
            if self.f[0] == -1 {
                -1
            } else {
                self.f[0] >> shift
            },
            if self.f[1] == -1 {
                -1
            } else {
                self.f[1] >> shift
            },
        ];

        let x = [self.x[0] >> shift, self.x[1] >> shift];
        let y = [self.y[0] >> shift, self.y[1] >> shift];

        Ok(RangeId {
            z: ZoomLevel::new(target_z).unwrap(),
            f,
            x,
            y,

            i: self.i,
            #[cfg(feature = "temporal_id")]
            t: self.t,
        })
    }
}
