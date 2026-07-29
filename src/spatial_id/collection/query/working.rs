//! クエリの作業表現 [`WorkingTree`]。

use crate::FlexId;
use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use crate::spatial_id::collection::flex_tree::core::LeavesIntoIter;
use crate::spatial_id::collection::flex_tree::core::SafeValue;

/// クエリ実行中の中間状態。演算子はこの上で動く。
///
/// クレート内部限定のラッパーですが、中身は [`SpatialIdMap`](crate::SpatialIdMap) と
/// 同じ [`FlexTreeCore`] です。`shift`/`extrude`/`zoom_out`/`falloff`/`merge` などの演算子は、
/// この作業木の上での集合演算（構造共有・COW・並列マージ）として実装されています。
///
/// **内部構造は非公開**です。外から見えるのは
///
/// - セル列との相互変換（[`FromIterator`] / [`IntoIterator`]）
/// - 件数（[`count`](Self::count) / [`is_empty`](Self::is_empty)）
/// - 具象コレクションへの変換（`Into<`[`SpatialIdTable`](crate::SpatialIdTable)`>` /
///   `Into<`[`SpatialIdSet`](crate::SpatialIdSet)`>`）
///
/// だけです。内部表現は最適化の都合で予告なく変わりうるため、意図的に閉じています。
///
/// # 入力源を書くとき
/// [`Source`](crate::Source) を実装する側が必要なのは `FromIterator` だけです。
/// 読み出したセルを集めて `collect()` すれば作業木になります。
///
/// ```ignore
/// fn read_subset(&self, bounds: &[RangeId]) -> Result<WorkingTree<V>, Error> {
///     let mut cells: Vec<(FlexId, V)> = Vec::new();
///     // ... bounds に重なるセルを読む ...
///     Ok(cells.into_iter().collect())
/// }
/// ```
pub struct WorkingTree<V: SafeValue> {
    core: FlexTreeCore<V>,
}

impl<V: SafeValue> WorkingTree<V> {
    /// 空の作業木。
    pub fn new() -> Self {
        Self {
            core: FlexTreeCore::new(),
        }
    }

    /// 保持しているセル（[`FlexId`]）の数。
    pub fn count(&self) -> usize {
        self.core.count()
    }

    /// セルを1つも持たないか。
    pub fn is_empty(&self) -> bool {
        self.core.is_empty()
    }

    // --- 以下はクレート内部専用。作業木の中身を触れるのは演算子だけ。 ---

    pub(crate) fn core(&self) -> &FlexTreeCore<V> {
        &self.core
    }

    pub(crate) fn core_mut(&mut self) -> &mut FlexTreeCore<V> {
        &mut self.core
    }

    pub(crate) fn into_core(self) -> FlexTreeCore<V> {
        self.core
    }

    pub(crate) fn from_core(core: FlexTreeCore<V>) -> Self {
        Self { core }
    }
}

impl<V: SafeValue> Default for WorkingTree<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: SafeValue> Clone for WorkingTree<V> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
        }
    }
}

impl<V: SafeValue> PartialEq for WorkingTree<V> {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
    }
}

impl<V: SafeValue> Eq for WorkingTree<V> {}

impl<V: SafeValue> core::fmt::Debug for WorkingTree<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WorkingTree")
            .field("count", &self.count())
            .finish_non_exhaustive()
    }
}

impl<V: SafeValue> FromIterator<(FlexId, V)> for WorkingTree<V> {
    /// `(FlexId, 値)` 列から作業木を組む（重なりは union・左優先）。
    fn from_iter<I: IntoIterator<Item = (FlexId, V)>>(iter: I) -> Self {
        Self {
            core: iter.into_iter().collect(),
        }
    }
}

impl<V: SafeValue> IntoIterator for WorkingTree<V> {
    type Item = (FlexId, V);
    type IntoIter = LeavesIntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        self.core.into_iter()
    }
}
