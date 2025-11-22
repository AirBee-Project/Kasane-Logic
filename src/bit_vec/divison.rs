use crate::bit_vec::BitVec;

impl BitVec {
    /// target の範囲から division の複数区間を順に除外し、
    /// 最終的に残った BitVec の開始点のみ返す
    pub fn division(target: BitVec, division: Vec<BitVec>) -> Vec<BitVec> {
        let mut result = vec![target];

        for div in division {
            let mut next = vec![];

            for now in result.into_iter() {
                // div が now の範囲に含まれる場合 → 分割
                if div >= now && div <= now.upper_bound() {
                    let mut div_clone = div.clone();
                    next.extend(Self::sub(&now, &mut div_clone));
                } else {
                    // そのまま残す
                    next.push(now);
                }
            }

            result = next;
        }

        result
    }

    /// 単体範囲 target から単体 sub を引いて残りの開始点を返す
    pub fn sub(target: &BitVec, sub: &mut BitVec) -> Vec<BitVec> {
        let mut result: Vec<BitVec> = vec![];

        // sub が target と一致するまで bottom layer を変化させる
        while target != sub {
            sub.reverse_bottom_layer();
            result.push(sub.clone());
            sub.remove_bottom_layer();
        }

        result
    }
}
