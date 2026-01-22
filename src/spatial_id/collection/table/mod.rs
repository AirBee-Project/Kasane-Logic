pub mod logic;
pub mod memory;

use crate::{
    spatial_id::collection::{FlexIdRank, ValueRank},
    storage::{KeyValueStore, OrderedKeyValueStore},
};

pub trait TableStorage {
    type Value: Clone + PartialEq + Ord;
    type Forward: KeyValueStore<FlexIdRank, ValueRank>;
    type Dictionary: OrderedKeyValueStore<ValueRank, Self::Value>;
    type Reverse: KeyValueStore<Self::Value, ValueRank>;

    fn forward(&self) -> &Self::Forward;
    fn forward_mut(&mut self) -> &mut Self::Forward;

    fn dictionary(&self) -> &Self::Dictionary;
    fn dictionary_mut(&mut self) -> &mut Self::Dictionary;

    fn reverse(&self) -> &Self::Reverse;
    fn reverse_mut(&mut self) -> &mut Self::Reverse;
}
