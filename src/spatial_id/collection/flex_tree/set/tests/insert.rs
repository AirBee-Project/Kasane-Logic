#[cfg(test)]
mod tests {
    use crate::{
        F_MAX, F_MIN, FlexId, IntoFlexIds, IntoSingleIds, IterSingleIds, MAX_ZOOM_LEVEL, RangeId,
        SingleId, SpatialIdSet, XY_MAX,
    };
    ///単純なSingleIdを1つだけ挿入するケース
    #[test]
    fn first_insert_single_id() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(3, 3, 3, 3).unwrap();
        set.insert(single_id.clone());

        //SetからSingleIdを取り出す
        let single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //含まれるIDは3/3/3/3と一致するはず
        assert_eq!(single_id, single_ids.first().unwrap().clone())
    }

    #[test]
    fn first_insert_range_id() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //RangeIdの作成と挿入
        let range_id = RangeId::new(4, [-4, 5], [2, 10], [3, 3]).unwrap();
        set.insert(range_id.clone());

        //SetからSingleIdを取り出す
        let mut single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //正解
        let mut answer: Vec<SingleId> = range_id.into_single_ids().collect();

        answer.sort();
        single_ids.sort();

        //並び替えれば全く同じになる
        assert_eq!(answer, single_ids);
    }

    ///0/0/0/0を1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_largest() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(0, 0, 0, 0).unwrap();
        set.insert(single_id.clone());

        //SetからSingleIdを取り出す
        let single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //含まれるIDは0/0/0/0と一致するはず
        assert_eq!(single_id, single_ids.first().unwrap().clone())
    }

    ///0/-1:0/0/0を1つだけ挿入するケース
    #[test]
    fn first_insert_range_id_largest() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //RangeIdの作成と挿入
        let range_id = RangeId::new(0, [-1, 0], [0, 0], [0, 0]).unwrap();
        set.insert(range_id.clone());

        //SetからRangeIdを取り出す
        let mut single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(2, single_ids.len());

        //正解
        let mut answer: Vec<SingleId> = range_id.into_single_ids().collect();

        answer.sort();
        single_ids.sort();

        //並び替えれば全く同じになる
        assert_eq!(answer, single_ids);
    }

    ///最も小さなSingleIdを1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_smallest() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, 10, 10, 10).unwrap();
        set.insert(single_id.clone());

        //SetからSingleIdを取り出す
        let single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //答えが一致するはず
        assert_eq!(single_id, single_ids.first().unwrap().clone())
    }

    ///最も小さなSingleIdを端に1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_smallest_edge_start() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, F_MIN[MAX_ZOOM_LEVEL], 0, 0).unwrap();
        set.insert(single_id.clone());

        //SetからRangeIdを取り出す
        let single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(single_id, single_ids.first().unwrap().clone())
    }

    ///最も小さなSingleIdを端に1つだけ挿入するケース
    #[test]
    fn first_insert_single_id_smallest_edge_end() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id = SingleId::new(
            MAX_ZOOM_LEVEL as u8,
            F_MAX[MAX_ZOOM_LEVEL],
            XY_MAX[MAX_ZOOM_LEVEL],
            XY_MAX[MAX_ZOOM_LEVEL],
        )
        .unwrap();
        set.insert(single_id.clone());

        //SetからRangeIdを取り出す
        let single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(single_id, single_ids.first().unwrap().clone())
    }

    ///2つのIDを挿入するテスト
    ///AがBに含まれる場合にBのみが残るかをテストする
    ///1:1の重複していた場合の競合解消
    #[test]
    fn multiple_insert_single_id_overlap() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(3, 1, 1, 0).unwrap();

        set.insert(single_id_a.clone());
        set.insert(single_id_b.clone());

        //SetからSingleIdを取り出す
        let single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(single_id_b, single_ids.first().unwrap().clone())
    }

    ///2つのIDを挿入するテスト
    ///AとBが兄弟の場合にRangeIdとして帰ってくるのかを検証する
    #[test]
    fn multiple_insert_single_id_join() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(4, 3, 2, 0).unwrap();

        set.insert(single_id_a.clone());
        set.insert(single_id_b.clone());

        //SetからRangeIdを取り出す
        let flex_ids: Vec<FlexId> = set.into_flex_ids().collect();

        //長さは1になるはず
        assert_eq!(1, flex_ids.len());

        let answer: Vec<FlexId> = RangeId::new(4, [3, 3], [2, 2], [0, 1])
            .unwrap()
            .into_flex_ids()
            .collect();

        //含まれるIDは生成したFlexIdと一致するはず
        assert_eq!(flex_ids.first(), answer.first())
    }

    ///2つのIDを挿入するテスト
    ///AとBが隣り合っているが、兄弟ではない場合に分かれて帰ってくるか
    #[test]
    fn multiple_insert_single_id_no_join() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //SingleIdの作成と挿入
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(4, 3, 2, 2).unwrap();

        set.insert(single_id_a.clone());
        set.insert(single_id_b.clone());

        //SetからRangeIdを取り出す
        let mut single_ids: Vec<SingleId> = set.into_single_ids().collect();

        //長さは2になるはず
        assert_eq!(2, single_ids.len());

        //答え
        let mut answer = vec![single_id_a, single_id_b];

        single_ids.sort();
        answer.sort();

        //含まれるIDは生成したSingleIdと一致するはず
        assert_eq!(single_ids, answer)
    }

    ///RangeIdを挿入したときに、大きなIDになって帰ってくるか
    #[test]
    fn first_insert_range_id_join() {
        //Setの新規作成
        let mut set = SpatialIdSet::default();

        //RangeIdの作成と挿入
        let range_id = RangeId::new(4, [0, F_MAX[4]], [0, XY_MAX[4]], [0, XY_MAX[4]]).unwrap();

        set.insert(range_id.clone());

        //SetからSingleidを取り出す
        let single_ids: Vec<SingleId> = set.iter_single_ids().collect();

        //長さは1になるはず
        assert_eq!(1, single_ids.len());

        //地表面より上の全てのID=0/0/0/0と一致するはず
        assert_eq!(
            *single_ids.first().unwrap(),
            SingleId::new(0, 0, 0, 0).unwrap()
        )
    }
}
