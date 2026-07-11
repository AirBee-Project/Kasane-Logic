use crate::{ConflictPolicy, TemporalId};
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(Clone, Debug, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
/// Coreでは[`TemporalId`]は扱わない。このレイヤーでは始点と終点で考える。
/// このレイヤーでは数のキャッシュなども担当する。
pub(crate) struct TemporalCore<V> {
    // Vecの中身は（開始時刻,終了時刻,値）となっている。
    // 開始時刻と終了時刻から[Interval]のサイズが自動的に特定できます。
    ranges: Vec<(u64, u64, V)>,
    // `ranges` から導出できるキャッシュ値（同じ被覆なら常に同じ値になるよう、
    // 更新箇所は必ず再計算・差分計算で正しく保つこと）。
    // 等価性・順序・ハッシュの判定には決して使わない（`ranges` のみで判定する。
    // 下記の手動 impl を参照）。同じ被覆でも構築経路が違えばキャッシュ値が
    // 一時的に食い違うバグを踏んだ実績があるため、意図的に比較対象から外している。
    cached_len: usize,
}

/// 等価性・順序・ハッシュは `ranges` のみで判定する（`cached_len` はキャッシュに過ぎず、
/// 同じ被覆に対しては常に同じ値になるべきだが、それを比較の正としない）。
impl<V: PartialEq> PartialEq for TemporalCore<V> {
    fn eq(&self, other: &Self) -> bool {
        self.ranges == other.ranges
    }
}
impl<V: Eq> Eq for TemporalCore<V> {}
impl<V: PartialOrd> PartialOrd for TemporalCore<V> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.ranges.partial_cmp(&other.ranges)
    }
}
impl<V: Ord> Ord for TemporalCore<V> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.ranges.cmp(&other.ranges)
    }
}
impl<V: core::hash::Hash> core::hash::Hash for TemporalCore<V> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.ranges.hash(state);
    }
}

impl<V: Clone + PartialEq> TemporalCore<V> {
    /// 空の[`TemporalCore`]を作成する。
    pub(crate) fn new() -> Self {
        Self {
            ranges: Vec::new(),
            cached_len: 0,
        }
    }

    /// [`TemporalCore`]に新しい範囲を挿入する。
    pub(crate) fn insert(&mut self, range: core::ops::Range<u64>, v: V) {
        let s = range.start;
        let e = range.end;
        if s >= e {
            return;
        }

        // Fast path: 時系列データなどの「末尾への追記」をO(1)で処理する
        if let Some(last) = self.ranges.last_mut() {
            if last.1 <= s {
                if last.1 == s && last.2 == v {
                    // 直前の区間と連続し、値も同じ場合は結合。
                    //
                    // 注意: 結合後の区間は境界(s)が消えるぶん、より粗いセルへ
                    // 再分解され得る（例: [0,43200)+[43200,86400)→[0,86400) は
                    // Hourセル12個+12個ではなく、Dayセル1個に変わる）。そのため
                    // 単純に `count_range(s..e)` を加算するのではなく、結合前後の
                    // 区間全体を数え直して差分をキャッシュへ反映する。
                    let start = last.0;
                    let old_count = TemporalId::count_range(start..last.1);
                    last.1 = e;
                    let new_count = TemporalId::count_range(start..e);
                    self.cached_len = self.cached_len - old_count + new_count;
                    return;
                }
                if last.1 < s {
                    // 隙間がある場合は単に追加
                    self.ranges.push((s, e, v));
                    self.cached_len += TemporalId::count_range(s..e);
                    return;
                }
            }
        } else {
            // 空の場合は直接追加
            self.ranges.push((s, e, v));
            self.cached_len = TemporalId::count_range(s..e);
            return;
        }

        // Slow path: O(N) スイープによる上書き合成（重なりがある場合など）
        let other = Self {
            ranges: alloc::vec![(s, e, v.clone())],
            cached_len: TemporalId::count_range(s..e),
        };
        *self = self.overwrite(&other);
    }

    /// 空かどうか。
    pub(crate) fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// 正規化済みの時間範囲を借用で返す。
    pub(crate) fn ranges(&self) -> &[(u64, u64, V)] {
        &self.ranges
    }

    /// 指定秒が含まれる範囲とその値を取得します。
    pub(crate) fn contains_unixtime_range(&self, sec: u64) -> Option<(u64, u64, &V)> {
        let idx = self.ranges.partition_point(|(s, _, _)| *s <= sec);
        if idx == 0 {
            return None;
        }
        let (s, e, v) = &self.ranges[idx - 1];
        (sec < *e).then_some((*s, *e, v))
    }

    /// 境界イベント走査。各素区間 `[p, q)` について `self`/`other` の値を `combine` で合成する。
    pub(crate) fn sweep<U, F>(&self, other: &TemporalCore<U>, mut combine: F) -> Self
    where
        F: FnMut(Option<&V>, Option<&U>) -> Option<V>,
    {
        let a = &self.ranges;
        let b = &other.ranges;
        let mut out: Vec<(u64, u64, V)> = Vec::with_capacity(a.len() + b.len());

        let mut ia = 0;
        let mut ib = 0;

        // 最初のイベント時刻（探索開始地点）を決定
        let mut cur = match (a.first(), b.first()) {
            (Some(x), Some(y)) => x.0.min(y.0),
            (Some(x), None) => x.0,
            (None, Some(y)) => y.0,
            (None, None) => return Self::new(),
        };

        while ia < a.len() || ib < b.len() {
            // cur時点での a の状態（値と次の変化が起きる時刻）を判定
            let (a_val, next_a) = if ia < a.len() {
                if cur < a[ia].0 {
                    (None, a[ia].0) // 空隙にいる
                } else {
                    (Some(&a[ia].2), a[ia].1) // 区間内にいる
                }
            } else {
                (None, u64::MAX) // 枯渇
            };

            // cur時点での b の状態を判定
            let (b_val, next_b) = if ib < b.len() {
                if cur < b[ib].0 {
                    (None, b[ib].0)
                } else {
                    (Some(&b[ib].2), b[ib].1)
                }
            } else {
                (None, u64::MAX)
            };

            // 次のイベント（区間の切り替わり）は a と b の次時刻の小さい方
            let next_event = next_a.min(next_b);

            // この微小な区間 [cur, next_event) に対して値を合成
            if let Some(v) = combine(a_val, b_val) {
                if let Some(last) = out.last_mut() {
                    if last.1 == cur && last.2 == v {
                        // 直前の区間と連続し、値も同じならマージする
                        last.1 = next_event;
                    } else if cur < next_event {
                        out.push((cur, next_event, v));
                    }
                } else if cur < next_event {
                    out.push((cur, next_event, v));
                }
            }

            // 時刻を次のイベントへ進める
            cur = next_event;

            // 区間の終端を過ぎたらインデックスを進める
            if ia < a.len() && cur == a[ia].1 {
                ia += 1;
            }
            if ib < b.len() && cur == b[ib].1 {
                ib += 1;
            }
        }

        let cached_len = out
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s..*e))
            .sum();

        Self {
            ranges: out,
            cached_len,
        }
    }
    /// 差集合 `self - other`（時間で other を除く。値は self 由来）。
    pub(crate) fn difference(&self, other: &Self) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a_val), None) => Some(a_val.clone()),
            _ => None,
        })
    }

    /// 上書き合成。時間が重なる部分は `other` の値が勝ち、重ならない部分は各自の値を保つ。
    pub(crate) fn overwrite(&self, other: &Self) -> Self {
        self.sweep(other, |a, b| b.or(a).cloned())
    }

    /// 時間窓 `window`（値なしの区間列）に含まれる時間だけを残す（値は self 由来）。
    pub(crate) fn intersect_time(&self, window: &TemporalCore<()>) -> Self {
        self.sweep(window, |a, b| match (a, b) {
            (Some(a_val), Some(_)) => Some(a_val.clone()),
            _ => None,
        })
    }

    /// `window`に含まれる時間を取り除く。
    pub(crate) fn subtract_time(&self, window: &TemporalCore<()>) -> Self {
        self.sweep(window, |a, b| match (a, b) {
            (Some(a_val), None) => Some(a_val.clone()),
            _ => None,
        })
    }

    /// 保持する[`TemporalId`]の個数を返す。
    pub(crate) fn len(&self) -> usize {
        self.cached_len
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (TemporalId, &V)> + '_ {
        self.ranges
            .iter()
            .flat_map(|(s, e, v)| TemporalId::from_range(*s..*e).unwrap().map(move |c| (c, v)))
    }

    #[cfg_attr(not(any(test, feature = "persist")), allow(dead_code))]
    pub(crate) fn ranges_ref(&self) -> Vec<(u64, u64, &V)> {
        self.ranges.iter().map(|(s, e, v)| (*s, *e, v)).collect()
    }

    /// 呼び出し側は列が正規化済みであることを保証すること。
    #[cfg(feature = "persist")]
    pub(crate) fn from_raw_ranges(ranges: Vec<(u64, u64, V)>) -> Self {
        let cached_len = ranges
            .iter()
            .map(|(s, e, _)| TemporalId::count_range(*s..*e))
            .sum();
        Self { ranges, cached_len }
    }
}

impl<V: Clone + Ord> TemporalCore<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub(crate) fn union(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        })
    }

    /// 積（both のみ・`policy` で値解決）。
    #[allow(dead_code)]
    pub(crate) fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        self.sweep(other, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            _ => None,
        })
    }
}

impl<'a, V> IntoIterator for &'a TemporalCore<V>
where
    V: Clone + PartialEq,
{
    type Item = (TemporalId, &'a V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.ranges
                .iter()
                .flat_map(|(s, e, v)| TemporalId::from_range(*s..*e).unwrap().map(move |c| (c, v))),
        )
    }
}

impl<V: Clone + 'static> IntoIterator for TemporalCore<V> {
    type Item = (TemporalId, V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.ranges.into_iter().flat_map(|(s, e, v)| {
            TemporalId::from_range(s..e)
                .unwrap()
                .map(move |c| (c, v.clone()))
        }))
    }
}
