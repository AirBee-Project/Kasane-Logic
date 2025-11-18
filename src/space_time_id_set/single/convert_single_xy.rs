pub fn convert_xy(z: u8, dimension: [u64; 2]) -> Vec<(u8, u64)> {
    let mut target = dimension;
    let mut now_z = z;
    let mut result = Vec::new();

    loop {
        if target[0] == target[1] {
            // 終端 → これ以上分割しない
            result.push((now_z, target[0]));
            break;
        }

        // 左端が奇数なら個別に処理
        if !target[0].is_multiple_of(2) {
            result.push((now_z, target[0]));
            target[0] += 1;
        }

        // 右端が偶数なら個別に処理
        if target[1].is_multiple_of(2) {
            result.push((now_z, target[1]));
            target[1] -= 1;
        }

        // 範囲が逆転したら終了
        if target[0] > target[1] {
            break;
        }

        // 次のズームレベルへ（範囲を半分に縮小）
        if now_z == 0 {
            break; // z=0で終了
        }

        target = [target[0] / 2, target[1] / 2];
        now_z -= 1;
    }

    result
}
