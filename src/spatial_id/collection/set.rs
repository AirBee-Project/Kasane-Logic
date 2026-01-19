use std::ops::{BitAnd, BitOr, Deref, DerefMut, Not, Sub};

use crate::{
    RangeId,
    spatial_id::{
        SpatialIdEncode,
        collection::{
            MapTrait,
            map::{MapLogic, OnMemoryMap}, // トレイト定義との衝突回避のため適宜リネームや調整
        },
        flex_id::FlexId,
    },
};

// =============================================================================
//  2. Public API: SpatialIdMap
//     ユーザーが利用する標準のマップ型 (Logic + Storage)
// =============================================================================

/// 標準のインメモリ空間IDマップ。
///
/// 空間IDをキーとして値を保持します。挿入・削除時に自動的に結合・分割が行われます。
#[derive(Clone)]
pub struct SpatialIdMap<V>(MapLogic<OnMemoryMap<V>>);

impl<V> SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    /// 新しい空のマップを作成します。
    pub fn new() -> Self {
        Self(MapLogic::new(OnMemoryMap::new()))
    }
}

impl<V> Default for SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

// MapLogicへの自動委譲
impl<V> Deref for SpatialIdMap<V> {
    type Target = MapLogic<OnMemoryMap<V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for SpatialIdMap<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// イテレータ対応 (参照)
impl<'a, V> IntoIterator for &'a SpatialIdMap<V>
where
    V: Clone + PartialEq + 'static,
{
    type Item = (RangeId, &'a V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.iter())
    }
}

// =============================================================================
//  3. Logic Layer: SetLogic
//     MapLogicをラップして集合演算を提供する中間層
// =============================================================================

#[derive(Clone)]
pub struct SetLogic<S>(pub(crate) MapLogic<S>);

impl<S> Default for SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    pub fn new() -> Self {
        Self(MapLogic::new(S::default()))
    }

    /// 既存のMap（V=()）からSetを作成
    pub fn from_map(map: MapLogic<S>) -> Self {
        Self(map)
    }

    pub fn size(&self) -> usize {
        self.0.size()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.0.keys()
    }

    /// 演算用の内部IDイテレータ
    fn iter_encode(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.0.keys_encode()
    }

    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T) {
        // 値は常に ()
        self.0.insert(target, &());
    }

    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        self.0.remove(target);
    }

    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> SetLogic<S> {
        // Mapのsubset結果をラップして返す
        SetLogic(self.0.subset(target))
    }

    /// 和集合 (A | B)
    pub fn union(&self, other: &SetLogic<S>) -> SetLogic<S> {
        let mut result = self.clone();
        for encode in other.iter_encode() {
            result.insert(&encode);
        }
        result
    }

    /// 積集合 (A & B)
    /// 実装方針: 小さい集合の要素ごとに、大きい集合から subset (切り出し) を行いマージする。
    /// これにより MapLogic の結合・分割ロジックを再利用でき、保守性が高まる。
    pub fn intersection(&self, other: &SetLogic<S>) -> SetLogic<S> {
        let (small, large) = if self.size() < other.size() {
            (self, other)
        } else {
            (other, self)
        };

        let mut result = SetLogic::new();

        // 小さい方の集合に含まれる空間IDそれぞれについて
        for encode_id in small.iter_encode() {
            // 大きい方の集合から、その空間IDに含まれる部分だけを切り出す
            let part = large.subset(&encode_id);
            // 切り出した部分を結果に追加する
            for part_encode in part.iter_encode() {
                result.insert(&part_encode);
            }
        }
        result
    }

    /// 差集合 (A - B)
    pub fn difference(&self, other: &SetLogic<S>) -> SetLogic<S> {
        let mut result = self.clone();
        for encode in other.iter_encode() {
            result.remove(&encode);
        }
        result
    }

    /// 補集合 (Universe - A)
    pub fn not(self) -> Self {
        let mut universe = SetLogic::new();
        // 全空間を表すID (Level 0) を挿入
        // ※ RangeId::new_unchecked の引数は実際の定義に合わせてください
        let root_range = unsafe { RangeId::new_unchecked(0, [0, 1], [0, 0], [0, 0]) };
        universe.insert(&root_range);
        universe.difference(&self)
    }
}

// =============================================================================
//  4. Public API: SpatialIdSet
//     ユーザーが利用する標準のセット型 (Logic + Storage)
// =============================================================================

/// 標準のインメモリ空間IDセット。
///
/// 空間IDの集合を管理し、和集合(|), 積集合(&), 差集合(-) などの集合演算を提供します。
#[derive(Clone, Default)]
pub struct SpatialIdSet(SetLogic<OnMemoryMap<()>>);

impl SpatialIdSet {
    /// 新しい空のセットを作成します。
    pub fn new() -> Self {
        Self(SetLogic::new())
    }

    // --- ラッパーメソッド ---

    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T) {
        self.0.insert(target);
    }

    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        self.0.remove(target);
    }

    /// 和集合 (A ∪ B)
    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0))
    }

    /// 積集合 (A ∩ B)
    pub fn intersection(&self, other: &Self) -> Self {
        Self(self.0.intersection(&other.0))
    }

    /// 差集合 (A \ B)
    pub fn difference(&self, other: &Self) -> Self {
        Self(self.0.difference(&other.0))
    }

    /// 部分集合の取得
    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> Self {
        Self(self.0.subset(target))
    }
}

// SetLogicへの自動委譲 (読み取り専用メソッド用)
// ※ insert/remove等はAPIを制御するため直接DerefMutしない設計としています
impl Deref for SpatialIdSet {
    type Target = SetLogic<OnMemoryMap<()>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// --- 演算子オーバーロード ---

impl BitOr for &SpatialIdSet {
    type Output = SpatialIdSet;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitAnd for &SpatialIdSet {
    type Output = SpatialIdSet;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl Sub for &SpatialIdSet {
    type Output = SpatialIdSet;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl Not for SpatialIdSet {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

// --- イテレータ対応 ---

// 参照イテレータ (for id in &set)
impl<'a> IntoIterator for &'a SpatialIdSet {
    type Item = RangeId;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.iter())
    }
}

// 消費イテレータ (for id in set)
impl IntoIterator for SpatialIdSet {
    type Item = RangeId;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        // 内部構造のイテレータをVecに収集して返す
        let ids: Vec<RangeId> = self.0.iter().collect();
        Box::new(ids.into_iter())
    }
}

// コレクション変換: collect()
impl<T> FromIterator<T> for SpatialIdSet
where
    T: SpatialIdEncode,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = SpatialIdSet::new();
        for item in iter {
            set.insert(&item);
        }
        set
    }
}

// コレクション拡張: extend()
impl<T> Extend<T> for SpatialIdSet
where
    T: SpatialIdEncode,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(&item);
        }
    }
}
