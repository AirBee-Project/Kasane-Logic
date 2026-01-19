use std::{
    collections::BTreeMap,
    ops::{BitAnd, BitOr, Not, Sub},
};

use roaring::RoaringTreemap;

use crate::{
    RangeId,
    kv::KvStore,
    spatial_id::{
        SpatialIdEncode,
        collection::{
            MapTrait, Rank,
            map::{MapLogic, OnMemoryMap},
        },
        flex_id::FlexId,
        segment::Segment,
    },
};

// Setは実質的に Map<()> のラッパー
#[derive(Clone, Default)]
pub struct OnMemorySet(pub(crate) OnMemoryMap<()>);

impl OnMemorySet {
    pub fn new() -> Self {
        Self(OnMemoryMap::new())
    }
}

// MapTraitを委譲する
impl MapTrait for OnMemorySet {
    type V = ();
    type DimensionMap = BTreeMap<Segment, RoaringTreemap>;
    type MainMap = BTreeMap<Rank, (FlexId, ())>;

    fn f(&self) -> &Self::DimensionMap {
        self.0.f()
    }
    fn f_mut(&mut self) -> &mut Self::DimensionMap {
        self.0.f_mut()
    }
    fn x(&self) -> &Self::DimensionMap {
        self.0.x()
    }
    fn x_mut(&mut self) -> &mut Self::DimensionMap {
        self.0.x_mut()
    }
    fn y(&self) -> &Self::DimensionMap {
        self.0.y()
    }
    fn y_mut(&mut self) -> &mut Self::DimensionMap {
        self.0.y_mut()
    }
    fn main(&self) -> &Self::MainMap {
        self.0.main()
    }
    fn main_mut(&mut self) -> &mut Self::MainMap {
        self.0.main_mut()
    }
    fn fetch_next_rank(&self) -> Rank {
        self.0.fetch_next_rank()
    }
    fn clear(&mut self) {
        self.0.clear()
    }
}

#[derive(Clone)]
pub struct SetLogic<S>(pub(crate) MapLogic<S>);

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
        // Map::subset は部分集合のMapを返すので、それをSetにする
        SetLogic(self.0.subset(target))
    }

    /// 和集合 (A | B)
    pub fn union(&self, other: &SetLogic<S>) -> SetLogic<S> {
        // 新しいSetを作成 (S::default() で空のストアを作成)
        let mut result = self.clone();
        for encode in other.iter_encode() {
            result.insert(&encode);
        }
        result
    }

    /// 積集合 (A & B)
    pub fn intersection(&self, other: &SetLogic<S>) -> SetLogic<S> {
        let mut result = SetLogic::new();

        let (small, large) = if self.size() < other.size() {
            (self, other)
        } else {
            (other, self)
        };

        for encode_id in small.iter_encode() {
            let related_ranks = large.0.related(&encode_id);

            let large_store = large.0.inner();

            for rank in related_ranks {
                if let Some((large_id, _)) = large_store.main().get(&rank) {
                    if let Some(inter) = encode_id.intersection(large_id) {
                        result.insert(&inter);
                    }
                }
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
}

impl<S> BitOr for &SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    type Output = SetLogic<S>;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl<S> BitAnd for &SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    type Output = SetLogic<S>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<S> Sub for &SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    type Output = SetLogic<S>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl<S> Not for SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    type Output = Self;
    fn not(self) -> Self::Output {
        let mut universe = SetLogic::new();
        let root_range = unsafe { RangeId::new_unchecked(0, [0, 1], [0, 0], [0, 0]) };
        universe.insert(&root_range);
        universe.difference(&self)
    }
}

impl<S> Default for SetLogic<S>
where
    S: MapTrait<V = ()> + Default + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
