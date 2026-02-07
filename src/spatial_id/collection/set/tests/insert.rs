#[cfg(test)]
mod tests {
    use crate::{
        F_MAX, F_MIN, MAX_ZOOM_LEVEL, RangeId, SingleId, XY_MAX,
        spatial_id::collection::set::SetOnMemory,
    };
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

        answer.sort();
        single_ids.sort();

        //並び替えれば全く同じになる
        assert_eq!(answer, single_ids);
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

        answer.sort();
        single_ids.sort();

        //並び替えれば全く同じになる
        assert_eq!(answer, single_ids);
    }

    ///最も小さなSingleIdを1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_smallest() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, 10, 10, 10).unwrap();
        set.insert(&single_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDはランダムに生成したSingleIdと一致するはず
        assert_eq!(RangeId::from(single_id), range_ids.first().unwrap().clone())
    }

    ///最も小さなSingleIdを端に1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_smallest_edge_start() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, F_MIN[MAX_ZOOM_LEVEL], 0, 0).unwrap();
        set.insert(&single_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(RangeId::from(single_id), range_ids.first().unwrap().clone())
    }

    ///最も小さなSingleIdを端に1つだけ挿入するケース
    ///コーナーケースのバグを拾う
    #[test]
    fn first_insert_single_id_smallest_edge_end() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(
            MAX_ZOOM_LEVEL as u8,
            F_MAX[MAX_ZOOM_LEVEL],
            XY_MAX[MAX_ZOOM_LEVEL],
            XY_MAX[MAX_ZOOM_LEVEL],
        )
        .unwrap();
        set.insert(&single_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(RangeId::from(single_id), range_ids.first().unwrap().clone())
    }

    ///2つのIDを挿入するテスト
    ///AがBに含まれる場合にBのみが残るかをテストする
    ///1:1の重複していた場合の競合解消
    #[test]
    fn multiple_insert_single_id_overlap() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(3, 1, 1, 0).unwrap();

        set.insert(&single_id_a);
        set.insert(&single_id_b);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(
            RangeId::from(single_id_b),
            range_ids.first().unwrap().clone()
        )
    }

    ///2つのIDを挿入するテスト
    ///AとBが兄弟の場合にRangeIdとして帰ってくるのかを検証する
    #[test]
    fn multiple_insert_single_id_join() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(4, 3, 2, 0).unwrap();

        set.insert(&single_id_a);
        set.insert(&single_id_b);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(
            *range_ids.first().unwrap(),
            RangeId::new(4, [3, 3], [2, 2], [0, 1]).unwrap()
        )
    }

    ///2つのIDを挿入するテスト
    ///AとBが隣り合っているが、兄弟ではない場合に分かれて帰ってくるか
    #[test]
    fn multiple_insert_single_id_no_join() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(4, 3, 2, 2).unwrap();

        set.insert(&single_id_a);
        set.insert(&single_id_b);

        //SetからRangeIdを取り出す
        let mut range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(2, range_ids.len());

        //答え
        let mut answer = vec![RangeId::from(single_id_a), RangeId::from(single_id_b)];

        range_ids.sort();
        answer.sort();

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(range_ids, answer)
    }

    ///RangeIdを挿入したときに、大きなIDになって帰ってくるか
    #[test]
    fn first_insert_range_id_join() {
        //Setの新規作成
        let mut set = SetOnMemory::new();

        //SingleIdの作成と挿入
        let range_id = RangeId::new(4, [0, F_MAX[4]], [0, XY_MAX[4]], [0, XY_MAX[4]]).unwrap();

        set.insert(&range_id);

        //SetからRangeIdを取り出す
        let range_ids: Vec<RangeId> = set.range_ids().collect();

        //長さは1になるはず
        assert_eq!(1, range_ids.len());

        //地表面より上の全てのID=0/0/0/0と一致するはず
        assert_eq!(
            *range_ids.first().unwrap(),
            RangeId::new(0, [0, 0], [0, 0], [0, 0]).unwrap()
        )
    }
}
