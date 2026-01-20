use roaring::RoaringTreemap;
pub mod logic;
pub mod memory;

use crate::{
    BTreeMapTrait,
    spatial_id::collection::{FlexIdRank, ValueRank},
};

pub trait TableStorage {
    /// 格納するデータの型
    type Value: Clone + PartialEq + Ord;

    type Forward: BTreeMapTrait<FlexIdRank, ValueRank>;
    type Dictionary: BTreeMapTrait<ValueRank, Self::Value>;
    type Reverse: BTreeMapTrait<Self::Value, ValueRank>;

    fn forward(&self) -> &Self::Forward;
    fn forward_mut(&mut self) -> &mut Self::Forward;

    fn dictionary(&self) -> &Self::Dictionary;
    fn dictionary_mut(&mut self) -> &mut Self::Dictionary;

    fn reverse(&self) -> &Self::Reverse;
    fn reverse_mut(&mut self) -> &mut Self::Reverse;
}
