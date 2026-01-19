use crate::kv::KvStore;
use crate::spatial_id::flex_id::FlexId;
use crate::spatial_id::segment::Segment;
use roaring::RoaringTreemap;

mod map;
mod set;
pub type Rank = u64;

pub use map::{MapLogic, SpatialIdMap};
pub use set::{SetLogic, SpatialIdSet};

/// 空間インデックスに必要なストレージ機能をまとめたトレイト
pub trait MapTrait {
    type V; // ユーザーが格納する値の型

    type DimensionMap: KvStore<Segment, RoaringTreemap>;
    type MainMap: KvStore<Rank, (FlexId, Self::V)>;

    fn f(&self) -> &Self::DimensionMap;
    fn f_mut(&mut self) -> &mut Self::DimensionMap;

    fn x(&self) -> &Self::DimensionMap;
    fn x_mut(&mut self) -> &mut Self::DimensionMap;

    fn y(&self) -> &Self::DimensionMap;
    fn y_mut(&mut self) -> &mut Self::DimensionMap;

    fn main(&self) -> &Self::MainMap;
    fn main_mut(&mut self) -> &mut Self::MainMap;

    fn fetch_next_rank(&self) -> Rank;

    // 全削除用
    fn clear(&mut self);
}

/// 値に対して、高速なフィルターが欲しい場合のTrait
pub trait MapIndexTrait {
    type V: Ord;
    type ValueIndex: KvStore<Self::V, RoaringTreemap>;

    fn index(&self) -> &Self::ValueIndex;
    fn index_mut(&self) -> &mut Self::ValueIndex;
}
