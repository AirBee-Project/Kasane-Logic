pub fn convert_f(z: u8, dim: (i64, i64)) -> Vec<(u8, i64)> {
    if (dim.0 >= 0 && dim.1 >= 0) || (dim.0 < 0 && dim.1 < 0) {
        //上下どちらかにある場合
        return convert_f_logic(z, dim);
    } else {
        let mut result = vec![];
        println!("a");

        result.extend(convert_f_logic(z, (dim.0, -1)));
        result.extend(convert_f_logic(z, (0, dim.1)));

        return result;
    };
}

fn convert_f_logic(z: u8, dim: (i64, i64)) -> Vec<(u8, i64)> {
    println!("STRART Z:{} DIM:{:?}", z, dim);

    let mut current_range = Some(dim);
    let mut now_z = z;
    let mut result = Vec::new();

    while let Some(mut target) = current_range {
        println!("--------");
        println!("CureentRange : {:?}", current_range);
        println!("Now_Z : {}", now_z);

        // 終了条件：範囲が縮退した or z=0
        if target.0 >= target.1 {
            result.push((now_z, target.0));
            break;
        }

        if now_z == 0 {
            break;
        }

        //もしも処理する部分が隣まできたら
        if target.1 - target.0 == 1 {
            //二つをまとめられるかを判定する
            if target.0 % 2 == 0 {
                //まとめられる
                result.push((now_z - 1, target.0 / 2))
            } else {
                //まとめられない
                result.push((now_z, target.0));

                result.push((now_z, target.1));
            }

            break;
        }

        // 左端が奇数なら個別処理
        if target.0 % 2 != 0 {
            result.push((now_z, target.0));
            target.0 += 1;
        }

        // 右端が偶数なら個別処理
        if target.1 % 2 == 0 {
            result.push((now_z, target.1));
            target.1 -= 1;
        }

        // 範囲が逆転したら終了
        if target.0 > target.1 {
            println!("逆転");
            break;
        }

        // 次のズームレベルに範囲を縮小
        let a = target.0 / 2;
        let b = if target.1 == -1 { -1 } else { target.1 / 2 };

        if a == b {
            result.push((now_z - 1, a));
            break;
        }

        current_range = Some((a.min(b), a.max(b)));
        now_z -= 1;
    }

    println!("RESULT:{:?}", result);
    result
}
