use std::fs::File;

use kasane_logic::{
    function::line::line,
    point::{Coordinate, Point},
    space_time_id_set::SpaceTimeIdSet,
};
use std::io::Write;
fn main() {
    let mut set = SpaceTimeIdSet::new();

    // 新宿三丁目交差点付近（実在座標）
    let base_lat = 35.690921;
    let base_lon = 139.704405;

    // 中心（必ず交差する点）
    let center_alt = 20.0;

    // 方向を表すオフセット（約 20m 相当）
    let d = 0.00025;

    // 直線の端点群を作る
    let endpoints = vec![
        // ────────── 水平
        // 東西
        ((0.0, -d, 0.0), (0.0, d, 0.0)),
        // 南北
        ((-d, 0.0, 0.0), (d, 0.0, 0.0)),
        // ────────── 斜め（水平）
        // NE → SW
        ((-d, -d, 0.0), (d, d, 0.0)),
        // NW → SE
        ((-d, d, 0.0), (d, -d, 0.0)),
        // ────────── 高度を変えた 3D 交差
        // 斜めに上昇
        ((-d, 0.0, -10.0), (d, 0.0, 30.0)),
        // 斜めに下降
        ((0.0, -d, 30.0), (0.0, d, -10.0)),
        // 水平から上昇
        ((-d, d * 0.5, 0.0), (d, -d * 0.5, 40.0)),
        // 水平から下降
        ((-d, -d * 0.5, 40.0), (d, d * 0.5, 5.0)),
        // ────────── さらに複雑
        // ねじれた 3D 斜めライン
        ((-d, 0.0, 50.0), (d, 0.0, -20.0)),
        ((0.0, -d, 25.0), (0.0, d, -10.0)),
    ];

    // 直線を大量に生成
    for (start, end) in endpoints {
        let a = Point::Coordinate(Coordinate {
            latitude: base_lat + start.0,
            longitude: base_lon + start.1,
            altitude: center_alt + start.2,
        });

        let b = Point::Coordinate(Coordinate {
            latitude: base_lat + end.0,
            longitude: base_lon + end.1,
            altitude: center_alt + end.2,
        });

        let line_points = line(25, a, b); // 精度・分割数

        for ele in line_points {
            set.insert(ele);
        }
    }

    let mut file = File::create("output.txt").expect("cannot create file");

    for ele in set.get_all() {
        writeln!(file, "{},", ele).expect("cannot write to file");
    }

    println!("output.txt に書き出しました");
}
