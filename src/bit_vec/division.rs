use crate::bit_vec::BitVec;

impl BitVec {
    /// 有効階層の長さ（=1bit目が1である階層の数）を返す
    ///
    /// BitVecの中で有効なビットが設定されている階層の数を数える。
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
    ///
    /// 仕様より、BitVec の prefix の最大値は「自分自身」であるため、
    /// range = [self, self] となる。
    ///
    /// # 戻り値
    /// (開始値, 終了値) のタプル（両方とも self のクローン）
    pub fn range(&self) -> (BitVec, BitVec) {
        (self.clone(), self.clone())
    }

    /// BitVecを1階層ぶん +1 した値を返す（辞書順で次のBitVec）
    ///
    /// # 戻り値
    /// 次のBitVecが存在する場合は Some、オーバーフローする場合は None
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

    /// BitVecを1階層 -1 した値を返す
    ///
    /// # 戻り値
    /// 前のBitVecが存在する場合は Some、アンダーフローする場合は None
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

    /// 区間の差分を計算する: (t0, t1) - (d0, d1) （すべて両閉区間）
    ///
    /// target区間から division区間を除いた残りの区間を返す。
    ///
    /// # 引数
    /// * `(t0, t1)` - target区間（両閉区間）
    /// * `(d0, d1)` - division区間（両閉区間）
    fn subtract_range(
        (t0, t1): (BitVec, BitVec),
        (d0, d1): (BitVec, BitVec),
    ) -> Vec<(BitVec, BitVec)> {
        let mut out = vec![];

        // 左側 [t0, d0 - 1]
        if let Some(left_end) = d0.prev()
            && t0 <= left_end
        {
            out.push((t0.clone(), left_end));
        }

        // 右側 [d1 + 1, t1]
        if let Some(right_start) = d1.next()
            && right_start <= t1
        {
            out.push((right_start, t1.clone()));
        }

        out
    }

    /// target の範囲から division の複数区間を順に除外し、
    /// 最終的に残った BitVec の開始点のみ返す
    ///
    /// # 引数
    /// * `target` - 分割対象のBitVec
    /// * `division` - 除外する範囲のBitVecのリスト
    ///
    /// # 戻り値
    /// 残った区間の開始点のリスト
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
