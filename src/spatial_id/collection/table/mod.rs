pub mod logic;
pub mod memory;

use crate::storage::Batch;
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

    fn insert_value(&mut self, value: &Self::Value, flex_id_ranks: Vec<FlexIdRank>) -> ValueRank {
        let value_rank = if let Some(rank) = self.reverse().get(&value) {
            rank
        } else {
            let new_rank = self.fetch_value_rank();
            let mut dict_batch = Batch::new();
            dict_batch.put(new_rank, value.clone());
            self.dictionary_mut().apply_batch(dict_batch);
            let mut rev_batch = Batch::new();
            rev_batch.put(value.clone(), new_rank);
            self.reverse_mut().apply_batch(rev_batch);
            new_rank
        };

        let mut fwd_batch = Batch::new();
        for id_rank in flex_id_ranks {
            fwd_batch.put(id_rank, value_rank);
        }
        self.forward_mut().apply_batch(fwd_batch);

        value_rank
    }
}
