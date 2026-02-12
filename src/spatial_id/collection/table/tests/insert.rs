#[cfg(test)]
mod tests {
    use crate::{RangeId, SingleId, TableOnMemory};

    #[test]
    fn first_insert_single_id() {
        //Setの新規作成
        let mut set = TableOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(3, 3, 3, 3).unwrap();
        set.insert(&single_id, &"neko".to_string());

        //SetからRangeIdを取り出す
        let range_ids: Vec<_> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは3/3/3/3と一致するはず
        assert_eq!(
            (RangeId::from(single_id), &"neko".to_string()),
            range_ids.first().unwrap().clone()
        )
    }

    #[test]
    fn first_insert_range_id() {
        //Setの新規作成
        let mut set = TableOnMemory::new();

        //RangeIdの作成と挿入
        let range_id = RangeId::new(4, [-4, 5], [2, 10], [3, 3]).unwrap();
        set.insert(&range_id, &334);

        //SetからRangeIdを取り出す
        let range_ids: Vec<_> = set.range_ids().collect();

        //取り出したRangeIdを全てSingleIdに変換する
        let mut single_ids: Vec<_> = range_ids
            .iter()
            .flat_map(|(id, value)| id.single_ids())
            .collect();

        //正解
        let mut answer: Vec<SingleId> = range_id.single_ids().collect();

        answer.sort();
        single_ids.sort();

        //並び替えれば全く同じになる
        assert_eq!(answer, single_ids);
    }

    ///0/0/0/0を1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_largest() {
        //Setの新規作成
        let mut set = TableOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(0, 0, 0, 0).unwrap();
        set.insert(&single_id, &"inu".to_string());

        //SetからRangeIdを取り出す
        let range_ids: Vec<_> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは0/0/0/0と一致するはず
        assert_eq!(
            (RangeId::from(single_id), &"inu".to_string()),
            range_ids.first().unwrap().clone()
        )
    }
}
