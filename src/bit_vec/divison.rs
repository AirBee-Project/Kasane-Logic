use crate::bit_vec::BitVec;

impl BitVec {
    /// target から division に含まれる部分を除いた残りの BitVec リストを返す
    pub fn division(mut target: BitVec, division: Vec<BitVec>) -> Vec<BitVec> {
        // 最初は target の範囲ひとつ
        let mut ranges: Vec<(BitVec, BitVec)> = vec![target.range()];

        // division それぞれを差し引く
        for div in division {
            let div_range = div.range();
            let mut new_ranges = vec![];

            for t_range in ranges {
                new_ranges.extend(BitVec::subtract_range(t_range, div_range.clone()));
            }

            ranges = new_ranges;
        }

        // 残った範囲の start を BitVec として返す
        ranges.into_iter().map(|(s, _)| s).collect()
    }

    /// BitVec を区間 [start, end) に変換
    pub fn range(&self) -> (BitVec, BitVec) {
        (self.clone(), self.next_prefix())
    }

    /// 区間の差 (t0, t1) - (d0, d1)
    fn subtract_range(
        (t0, t1): (BitVec, BitVec),
        (d0, d1): (BitVec, BitVec),
    ) -> Vec<(BitVec, BitVec)> {
        let mut out = vec![];

        // 左側の残り
        if t0 < d0 {
            out.push((t0.clone(), d0.clone()));
        }

        // 右側の残り
        if d1 < t1 {
            out.push((d1.clone(), t1.clone()));
        }

        out
    }
}
