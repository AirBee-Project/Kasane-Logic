#[cfg(test)]
mod tests {
    use crate::{F_MIN, MAX_ZOOM_LEVEL, RangeId, SingleId, TableOnMemory};

    #[test]
    fn first_insert_single_id() {
        futures::executor::block_on(async {
            let mut set = TableOnMemory::new();
            let single_id = SingleId::new(3, 3, 3, 3).unwrap();
            set.insert(&single_id, &"Neko".to_string()).await;
            let range_ids: Vec<_> = set.range_ids_values().collect();
            assert_eq!(1, range_ids.len());
            assert_eq!(
                (RangeId::from(single_id), "Neko".to_string()),
                range_ids.first().unwrap().clone()
            )
        });
    }

    #[test]
    fn first_insert_range_id() {
        futures::executor::block_on(async {
            let mut logic = TableOnMemory::new();

            // RangeIdの作成
            let range_id = RangeId::new(4, [-4, 5], [2, 10], [3, 3]).unwrap();
            let value = "Inu".to_string();

            logic.insert(&range_id, &value).await;

            let range_ids_with_val: Vec<_> = logic.range_ids_values().collect();

            let mut retrieved_single_ids: Vec<SingleId> = range_ids_with_val
                .iter()
                .flat_map(|(id, _val)| id.single_ids()) // _val は無視して IDだけ展開
                .collect();

            let mut answer: Vec<SingleId> = range_id.single_ids().collect();

            answer.sort();
            retrieved_single_ids.sort();

            assert_eq!(answer, retrieved_single_ids);

            assert_eq!(range_ids_with_val[0].1, "Inu");
        });
    }

    ///0/0/0/0を1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_largest() {
        futures::executor::block_on(async {
            //Setの新規作成
            let mut set = TableOnMemory::new();
            let single_id = SingleId::new(0, 0, 0, 0).unwrap();
            set.insert(&single_id, &"Neko".to_string()).await;
            let range_ids: Vec<_> = set.range_ids_values().collect();

            //長さは1になるはず
            assert_eq!(1, range_ids.len());

            //含まれるIDは0/0/0/0と一致するはず
            assert_eq!(
                (RangeId::from(single_id), "Neko".to_string()),
                range_ids.first().unwrap().clone()
            )
        });
    }

    ///最も小さなSingleIdを1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_smallest() {
        futures::executor::block_on(async {
            //Setの新規作成
            let mut set = TableOnMemory::new();

            //SingleIdの作成と挿入
            let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, 10, 10, 10).unwrap();
            set.insert(&single_id, &"Rakuda".to_string()).await;

            let range_ids: Vec<_> = set.range_ids_values().collect();

            assert_eq!(1, range_ids.len());

            assert_eq!(
                (RangeId::from(single_id), "Rakuda".to_string()),
                range_ids.first().unwrap().clone()
            )
        });
    }

    #[test]
    fn first_insert_single_id_smallest_edge_start() {
        futures::executor::block_on(async {
            //Setの新規作成
            let mut set = TableOnMemory::new();

            //SingleIdの作成と挿入
            let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, F_MIN[MAX_ZOOM_LEVEL], 0, 0).unwrap();
            set.insert(&single_id, &"neko".to_string()).await;

            let range_ids: Vec<_> = set.range_ids_values().collect();

            assert_eq!(1, range_ids.len());

            //含まれるIDは生成したSingleIdと一致するはず
            assert_eq!(
                (RangeId::from(single_id), "neko".to_string()),
                range_ids.first().unwrap().clone()
            )
        });
    }
}
