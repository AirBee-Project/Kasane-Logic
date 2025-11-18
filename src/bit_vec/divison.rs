use crate::bit_vec::BitVec;

impl BitVec {
    /// 有効階層の長さ（=1bit目が1である階層の数）
    pub fn valid_layers(&self) -> usize {
        let mut cnt = 0;
        for b in &self.0 {
            if b & 0b10 != 0 {
                break; // [0, x] 層 → 無効
            }
            cnt += 1;
        }
        cnt
    }

    /// 両閉区間の開始・終了値を返す
    /// BitVec は階層的な範囲を表すため、自分自身から next_prefix までの範囲となる
    /// → range = [self, next_prefix)
    pub fn range(&self) -> (BitVec, BitVec) {
        self.under_prefix()
    }

    /// BitVec を1階層ぶん +1 した値を返す（辞書順で次の BitVec）
    pub fn next(&self) -> Option<BitVec> {
        let mut out = self.0.clone();
        for i in (0..out.len()).rev() {
            if out[i] < 3 {
                // 2bit で表す値は 0〜3
                out[i] += 1;
                return Some(BitVec(out));
            }
            out[i] = 0; // carry
        }
        None // これ以上進めない
    }

    /// BitVec を1階層 -1 した値
    pub fn prev(&self) -> Option<BitVec> {
        let mut out = self.0.clone();
        for i in (0..out.len()).rev() {
            if out[i] > 0 {
                out[i] -= 1;
                return Some(BitVec(out));
            }
            out[i] = 3; // borrow: 2bit最大
        }
        None
    }

    /// (t0, t1) - (d0, d1) の区間差をとる（すべて両閉区間）
    fn subtract_range(
        (t0, t1): (BitVec, BitVec),
        (d0, d1): (BitVec, BitVec),
    ) -> Vec<(BitVec, BitVec)> {
        let mut out = vec![];

        // 左側 [t0, d0 - 1]
        if let Some(left_end) = d0.prev() {
            if t0 <= left_end {
                out.push((t0.clone(), left_end));
            }
        }

        // 右側 [d1 + 1, t1]
        if let Some(right_start) = d1.next() {
            if right_start <= t1 {
                out.push((right_start, t1.clone()));
            }
        }

        out
    }

    /// target の範囲から division の複数区間を順に除外し、
    /// 最終的に残った BitVec の開始点のみ返す
    pub fn division(target: BitVec, division: Vec<BitVec>) -> Vec<BitVec> {
        // 初期は target の範囲
        let mut ranges: Vec<(BitVec, BitVec)> = vec![target.range()];

        for div in division {
            let d_range = div.range();
            let mut new_ranges = vec![];

            for t_range in ranges {
                new_ranges.extend(Self::subtract_range(t_range, d_range.clone()));
            }

            ranges = new_ranges;
        }

        // 残った区間の start だけ返す
        ranges.into_iter().map(|(s, _)| s).collect()
    }
}
