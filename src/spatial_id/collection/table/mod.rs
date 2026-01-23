pub mod logic;
pub mod memory;

use crate::{
    spatial_id::collection::{FlexIdRank, ValueRank},
    storage::{KeyValueStore, OrderedKeyValueStore},
};

pub trait TableStorage {
    type Value: Clone + PartialEq + Ord;
    type Forward: KeyValueStore<FlexIdRank, ValueRank>;
    type Dictionary: KeyValueStore<ValueRank, Self::Value>;
    type Reverse: OrderedKeyValueStore<Self::Value, ValueRank>;

    fn forward(&self) -> &Self::Forward;
    fn forward_mut(&mut self) -> &mut Self::Forward;

    fn dictionary(&self) -> &Self::Dictionary;
    fn dictionary_mut(&mut self) -> &mut Self::Dictionary;

    fn reverse(&self) -> &Self::Reverse;
    fn reverse_mut(&mut self) -> &mut Self::Reverse;

    fn fetch_value_rank(&mut self) -> u64;
    fn return_value_rank(&mut self, rank: u64);

    ///ストレージ間でデータを移動するときに次に割り当てるべきRankを引き継ぐ用
    fn move_value_rank(&self) -> u64;

    ///ストレージ間でデータを移動するときにゴミのRankを引き継ぐ用
    fn move_value_rank_free_list(&self) -> Vec<u64>;
}
