#[cfg(test)]
mod tests {
    use crate::{RangeId, SetOnMemory, SingleId};
    ///単純なSingleIdを1つだけ挿入するケース
    #[test]
    fn first_insert_single_id() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(3, 3, 3, 3).unwrap();
        set.insert(&single_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは3/3/3/3と一致するはず
        assert_eq!(RangeId::from(single_id), range_ids.first().unwrap().clone())
    }

    #[test]
    fn first_insert_range_id() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //RangeIdの作成と挿入
        let range_id = RangeId::new(4, [-4, 5], [2, 10], [3, 3]).unwrap();
        set.insert(&range_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //取り出したRangeIdを全てSingleIdに変換する
        let mut single_ids: Vec<SingleId> =
            range_ids.iter().flat_map(|id| id.single_ids()).collect();

        //正解
        let mut answer: Vec<SingleId> = range_id.single_ids().collect();

        //並び替えれば全く同じになる
        assert_eq!(answer.sort(), single_ids.sort());
    }

    ///0/0/0/0を1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_largest() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(0, 0, 0, 0).unwrap();
        set.insert(&single_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは0/0/0/0と一致するはず
        assert_eq!(RangeId::from(single_id), range_ids.first().unwrap().clone())
    }

    ///0/-1:0/0/0を1つだけ挿入するケース
    #[test]
    fn first_insert_range_id_largest() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //RangeIdの作成と挿入
        let range_id = RangeId::new(0, [-1, 0], [0, 0], [0, 0]).unwrap();
        set.insert(&range_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //取り出したRangeIdを全てSingleIdに変換する
        let mut single_ids: Vec<SingleId> =
            range_ids.iter().flat_map(|id| id.single_ids()).collect();

        //正解
        let mut answer: Vec<SingleId> = range_id.single_ids().collect();

        //並び替えれば全く同じになる
        assert_eq!(answer.sort(), single_ids.sort());
    }
}
