//! `split_shard` を T 軸で割ったらどうなるかの実験。
use kasane_logic::{FlexId, SingleId, SpatialIdSet};

fn main() {
    // 空間だけのデータ（全セル全時間）
    let mut set = SpatialIdSet::new_in_shard(FlexId::new(2, 0, 2, 0, 2, 0).unwrap());
    for id in [
        SingleId::new(4, 0, 1, 1).unwrap(),
        SingleId::new(4, 0, 2, 2).unwrap(),
        SingleId::new(4, 0, 3, 0).unwrap(),
    ] {
        set.insert(id);
    }
    println!("元: count={} ids={:?}", set.count(), names(&set));

    // T 軸で割った領域を直接 extract_region に渡す（split_shard が T を選んだ場合と同じ）
    let region = FlexId::new(2, 0, 2, 0, 2, 0).unwrap();
    #[cfg(feature = "temporal_id")]
    {
        use kasane_logic::Side;
        let lower = region.split_t(Side::Lower).unwrap();
        let upper = region.split_t(Side::Upper).unwrap();
        println!("lower領域 t={}/{}", lower.t_zoomlevel(), lower.t());
        println!("upper領域 t={}/{}", upper.t_zoomlevel(), upper.t());
    }
    let _ = region;

    // 参考: 空間軸で割った場合（現状の実装）
    let ((lr, lo), (ur, up)) = set.split_shard().unwrap();
    println!(
        "空間分割: lower(f/x/y={}/{}/{}) count={} ids={:?}",
        lr.f_zoomlevel(),
        lr.x_zoomlevel(),
        lr.y_zoomlevel(),
        lo.count(),
        names(&lo)
    );
    println!(
        "          upper(f/x/y={}/{}/{}) count={} ids={:?}",
        ur.f_zoomlevel(),
        ur.x_zoomlevel(),
        ur.y_zoomlevel(),
        up.count(),
        names(&up)
    );
}

fn names(s: &SpatialIdSet) -> Vec<String> {
    let mut v: Vec<String> = s.iter().map(|i| i.to_string()).collect();
    v.sort();
    v
}
