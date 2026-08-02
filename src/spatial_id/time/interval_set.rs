//! 読み出し時に `{i}` として使ってよい時間間隔の候補集合。
//!
//! # なぜ必要か
//!
//! FlexTree は時間軸を2進トライで持つため、木から読み出した生のセルは
//! `2^(35 - zoom)` 秒の断片になる（`_8/182185424`、`_128/11386575` …）。人間が読める
//! `{i}` へ戻すには、結合した秒区間をどの単位で表すかを決めなければならない。
//!
//! 既定の `gcd(開始秒, 幅)` は**セル数が最小**（必ず1セル）になるが、`_7200/0` のように
//! 見慣れない単位が出てくる。候補集合を渡せば「その中で最もセル数が少なくなるもの」を
//! 選べるので、`_3600/0:1` のように意味の通る単位へ正規化できる。

use alloc::vec::Vec;

use crate::Interval;

/// 読み出しで `{i}` として使ってよい時間間隔の候補集合。
///
/// # 必ず含まれる2つの候補
///
/// どう構築しても [`Interval::WHOLE`] と（`temporal_id` feature 有効時は）
/// [`Interval::SECOND`] が自動的に加わる。この2つが無いと表現できない区間が生じるためである。
///
/// - **`SECOND`（1秒）**: 任意の秒区間を必ず表せる保証。これが無いと、たとえば
///   候補が `{MINUTE}` だけのとき `[30, 90)` を表せない（`30` が `60` で割り切れない）。
/// - **`WHOLE`（`2^35` 秒）**: 全時間の ID を1セルで表す保証。暦の単位は `2^35` を
///   割り切らない（`2^35 % 60 = 8`、`% 3600 = 2768`、`% 86400 = 13568`）ので、
///   `WHOLE` が無いと時間を指定していない ID が `_1/0:34359738367`（343億セル）に化ける。
///
/// ```
/// # #[cfg(feature = "temporal_id")]
/// # {
/// # use kasane_logic::{Interval, IntervalSet};
/// // 明示していなくても WHOLE と SECOND は入っている。
/// let units = IntervalSet::new([Interval::HOUR]);
/// assert!(units.contains(Interval::WHOLE));
/// assert!(units.contains(Interval::SECOND));
/// assert!(units.contains(Interval::HOUR));
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntervalSet {
    /// 秒数の降順。重複なし。先頭から見て最初に割り切れたものが「最も粗い」。
    units: Vec<Interval>,
}

impl IntervalSet {
    /// 候補から作る。[`Interval::WHOLE`] と [`Interval::SECOND`] は自動的に加わる。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, IntervalSet};
    /// // 30分・1時間で読みたい、という指定。
    /// let units = IntervalSet::new([Interval::new(1800).unwrap(), Interval::HOUR]);
    /// assert_eq!(units.coarsest_dividing(0, 7200), Interval::HOUR);
    /// assert_eq!(units.coarsest_dividing(0, 1800).seconds(), 1800);
    /// # }
    /// ```
    pub fn new(units: impl IntoIterator<Item = Interval>) -> Self {
        let mut units: Vec<Interval> = units.into_iter().collect();

        units.push(Interval::WHOLE);
        #[cfg(feature = "temporal_id")]
        units.push(Interval::SECOND);

        // 降順に並べておけば、探索は「先頭から最初に割り切れたもの」で済む。
        units.sort_unstable_by_key(|u| core::cmp::Reverse(u.seconds()));
        units.dedup();

        IntervalSet { units }
    }

    /// 暦の単位だけに正規化する候補集合。
    ///
    /// `{`[`WHOLE`](Interval::WHOLE)`,` [`DAY`](Interval::DAY)`,` [`HOUR`](Interval::HOUR)`,`
    /// [`MINUTE`](Interval::MINUTE)`,` [`SECOND`](Interval::SECOND)`}`。
    /// `temporal_id` feature 無効時は `WHOLE` のみになる（他の定数が存在しないため）。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, IntervalSet};
    /// let units = IntervalSet::calendar();
    /// // 2時間ぶんは「1時間 × 2セル」として表される（`gcd` なら `7200秒 × 1セル`）。
    /// assert_eq!(units.coarsest_dividing(0, 7200), Interval::HOUR);
    /// // 暦で割り切れない区間は秒まで落ちる。
    /// assert_eq!(units.coarsest_dividing(30, 90), Interval::SECOND);
    /// # }
    /// ```
    pub fn calendar() -> Self {
        #[cfg(feature = "temporal_id")]
        {
            Self::new([Interval::DAY, Interval::HOUR, Interval::MINUTE])
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            Self::new([])
        }
    }

    /// 候補に含まれるか。
    pub fn contains(&self, unit: Interval) -> bool {
        self.units.contains(&unit)
    }

    /// 候補を秒数の降順で列挙する。
    pub fn iter(&self) -> impl Iterator<Item = Interval> + '_ {
        self.units.iter().copied()
    }

    /// 絶対秒区間 `[start, end)` を表せる候補のうち、**最も粗いもの**を返す。
    ///
    /// 単位 `u` でこの区間を表せる条件は `start % u == 0 && end % u == 0` である
    /// （`t = [start/u, end/u - 1]` になる）。粗いほどセル数
    /// （`t_max - t_min + 1`）が少なくなるので、これが候補の中でのセル数最小を与える。
    ///
    /// [`Interval::SECOND`] が必ず候補にあるため、**この関数は必ず値を返す**
    /// （`temporal_id` feature 無効時は全時間しか存在せず、`WHOLE` が返る）。
    pub fn coarsest_dividing(&self, start: u64, end: u64) -> Interval {
        for unit in self.units.iter().copied() {
            let seconds = unit.seconds();
            if start.is_multiple_of(seconds) && end.is_multiple_of(seconds) {
                return unit;
            }
        }

        // 降順の末尾は必ず SECOND（feature 無効時は WHOLE で、区間も全時間しかない）。
        unreachable!("IntervalSet は必ず全区間を表せる候補を含む")
    }
}

impl Default for IntervalSet {
    /// [`calendar`](Self::calendar) と同じ。
    fn default() -> Self {
        Self::calendar()
    }
}

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::*;

    #[test]
    fn whole_and_second_are_always_present() {
        for units in [
            IntervalSet::new([]),
            IntervalSet::new([Interval::HOUR]),
            IntervalSet::calendar(),
        ] {
            assert!(units.contains(Interval::WHOLE));
            assert!(units.contains(Interval::SECOND));
        }
    }

    #[test]
    fn candidates_are_sorted_descending_without_duplicates() {
        let units = IntervalSet::new([Interval::HOUR, Interval::MINUTE, Interval::HOUR]);
        let seconds: Vec<u64> = units.iter().map(|u| u.seconds()).collect();
        assert_eq!(seconds, [Interval::MAX_SECONDS, 3600, 60, 1]);
    }

    /// 全時間は `WHOLE` で1セルに収まる。
    ///
    /// 暦の単位はどれも `2^35` を割り切らないので、`WHOLE` を必須にしていないと
    /// 時間なしの ID が `_1/0:34359738367`（343億セル）になってしまう。
    #[test]
    fn whole_time_stays_a_single_cell() {
        let units = IntervalSet::calendar();
        assert_eq!(
            units.coarsest_dividing(0, Interval::MAX_SECONDS),
            Interval::WHOLE
        );

        // 前提の確認: 暦の単位では 2^35 を割り切れない。
        for u in [86_400u64, 3_600, 60] {
            assert_ne!(
                Interval::MAX_SECONDS % u,
                0,
                "2^35 が {u} で割り切れてしまう"
            );
        }
    }

    #[test]
    fn picks_the_coarsest_candidate() {
        let units = IntervalSet::calendar();
        // ちょうど1日
        assert_eq!(units.coarsest_dividing(86_400, 172_800), Interval::DAY);
        // 2時間ぶん（`gcd` なら 7200 秒だが、候補にないので 1 時間 × 2 セル）
        assert_eq!(units.coarsest_dividing(0, 7_200), Interval::HOUR);
        // 30分ぶん
        assert_eq!(units.coarsest_dividing(0, 1_800), Interval::MINUTE);
        // 暦で表せない端数は秒まで落ちる
        assert_eq!(units.coarsest_dividing(30, 90), Interval::SECOND);
    }

    /// 候補を足せば、その単位が選ばれる。
    #[test]
    fn custom_candidates_are_honoured() {
        let half_hour = Interval::new(1_800).unwrap();
        let units = IntervalSet::new([half_hour]);
        assert_eq!(units.coarsest_dividing(1_800, 3_600), half_hour);
        // 候補に無い 3600 は選ばれず、1800 × 2 セルになる。
        assert_eq!(units.coarsest_dividing(0, 3_600), half_hour);
    }
}

/// 3コレクションから候補集合つきで読み出せることの疎通確認。
///
/// 単位の選択ロジックそのものは上の `tests` が押さえているので、ここでは
/// 「`SpatialIdSet` / `SpatialIdMap` / `SpatialIdTable` から同じ形で使えること」と
/// 「暦の正規化が既定（`gcd`）と実際に違う結果になること」を固定する。
#[cfg(all(test, feature = "temporal_id"))]
mod collection_api {
    use crate::{Interval, IntervalSet, SingleId, SpatialIdMap, SpatialIdSet, SpatialIdTable};
    use alloc::vec::Vec;

    /// 同じ空間セルの隣り合う2時間ぶん。値は同じなので結合される。
    fn two_hours() -> [SingleId; 2] {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        [
            base.clone().with_time(Interval::HOUR, 0).unwrap(),
            base.with_time(Interval::HOUR, 1).unwrap(),
        ]
    }

    #[test]
    fn set_reads_back_with_calendar_units() {
        let mut set = SpatialIdSet::new();
        for id in two_hours() {
            set.insert(id);
        }

        // 既定は gcd なので「7200 秒 × 1 セル」。
        let natural: Vec<_> = set.range_ids().map(|r| r.to_string()).collect();
        assert_eq!(natural, ["12/0/3638/1614_7200/0"]);

        // 暦に正規化すると「3600 秒 × 2 セル」。
        let calendar: Vec<_> = set.range_ids_calendar().map(|r| r.to_string()).collect();
        assert_eq!(calendar, ["12/0/3638/1614_3600/0:1"]);

        // `range_ids_in` に明示しても同じ。
        let explicit: Vec<_> = set
            .range_ids_in(&IntervalSet::calendar())
            .map(|r| r.to_string())
            .collect();
        assert_eq!(explicit, calendar);

        // 秒区間はどの表現でも変わらない。
        assert_eq!(
            set.range_ids().next().unwrap().seconds_range(),
            set.range_ids_calendar().next().unwrap().seconds_range()
        );
    }

    #[test]
    fn map_and_table_read_back_with_calendar_units() {
        let mut map: SpatialIdMap<u32> = SpatialIdMap::new();
        let mut table: SpatialIdTable<u32> = SpatialIdTable::new();
        for id in two_hours() {
            map.insert(id.clone(), 7);
            table.insert(id, 7);
        }

        for got in [
            map.range_ids_calendar()
                .map(|(r, v)| (r.to_string(), *v))
                .collect::<Vec<_>>(),
            table
                .range_ids_calendar()
                .map(|(r, v)| (r.to_string(), *v))
                .collect::<Vec<_>>(),
        ] {
            assert_eq!(got, [("12/0/3638/1614_3600/0:1".into(), 7u32)]);
        }
    }

    /// `flat_single_ids` 側でも単位を選べる。暦にすると2セルなので2件へ展開される。
    #[test]
    fn flat_single_ids_honours_the_unit() {
        let mut set = SpatialIdSet::new();
        for id in two_hours() {
            set.insert(id);
        }

        // 既定（gcd, 7200秒 × 1セル）は1件。
        assert_eq!(set.flat_single_ids().count(), 1);

        // 暦（3600秒 × 2セル）は2件へ展開される。
        let calendar: Vec<_> = set
            .flat_single_ids_calendar()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(calendar, ["12/0/3638/1614_3600/0", "12/0/3638/1614_3600/1"]);
    }

    /// 時間を持たない ID は、候補集合を渡しても全時間のまま。
    ///
    /// 暦の単位はどれも `2^35` を割り切らないので、`WHOLE` が候補に無いと
    /// `_1/0:34359738367`（343億セル）に化ける。
    #[test]
    fn whole_time_ids_are_untouched() {
        let mut set = SpatialIdSet::new();
        set.insert(SingleId::new(12, 0, 3638, 1614).unwrap());

        for got in [
            set.range_ids().next().unwrap(),
            set.range_ids_calendar().next().unwrap(),
        ] {
            assert_eq!(got.interval(), Interval::WHOLE);
            assert_eq!(got.t(), [0, 0]);
            assert_eq!(got.to_string(), "12/0/3638/1614");
        }
    }

    /// 仕様書の例（1800秒）は、既定なら往復し、暦に正規化すると分（60秒）へ落ちる。
    #[test]
    fn spec_example_under_each_policy() {
        let original = SingleId::new(12, 0, 3638, 1614)
            .unwrap()
            .with_time(1800, 809712)
            .unwrap();
        let mut set = SpatialIdSet::new();
        set.insert(original.clone());

        assert_eq!(
            set.range_ids().next().unwrap().to_string(),
            "12/0/3638/1614_1800/809712"
        );
        // 1800 は暦の候補に無いので、割り切れる最も粗い候補＝分になる。
        let calendar = set.range_ids_calendar().next().unwrap();
        assert_eq!(calendar.interval(), Interval::MINUTE);
        assert_eq!(calendar.t(), [24_291_360, 24_291_389]);
        // 秒区間は変わらない。
        assert_eq!(calendar.seconds_range(), original.seconds_range());
    }
}
