use crate::bit_vec::BitVec;

impl BitVec {
    /// target の範囲から division の複数区間を順に除外し、
    /// 残った範囲の開始点のみ返す
    pub fn subtract_ranges(&self, division: &[BitVec]) -> Vec<BitVec> {
        let mut result = vec![self.clone()]; // self を target として使用

        for div in division {
            let mut next = vec![];

            for now in result.into_iter() {
                // div が now の範囲に含まれる場合 → 分割
                if div >= &now && &div <= &&now.upper_bound() {
                    let div_clone = div.clone();
                    next.extend(BitVec::subtract_range(&now, &div_clone));
                } else {
                    next.push(now);
                }
            }

            result = next;
        }

        result
    }

    /// 単体範囲 BitVec から単体 sub を引いて残りの開始点を返す
    pub fn subtract_range(&self, sub: &BitVec) -> Vec<BitVec> {
        // sub が self と等しい、または sub が self の祖先（self を含む）の場合、
        // 引き算の結果は空になるため、空のベクタを返す
        if sub <= self {
            return vec![];
        }

        let mut result: Vec<BitVec> = vec![];
        let mut sub_clone = sub.clone();

        // sub が self と一致するまで leaf を操作
        while self != &sub_clone {
            sub_clone.flip_lowest_layer();
            result.push(sub_clone.clone());
            sub_clone.remove_lowest_layer();
        }

        result
    }
}
