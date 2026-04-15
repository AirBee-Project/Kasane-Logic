use crate::TemporalId;

impl TemporalId {
    /// 2つのTemporalIdの重なる時間範囲（Intersection）を計算して返します。
    /// 重なりがない場合は None を返します。
    pub fn intersection(&self, other: &TemporalId) -> Option<TemporalId> {
        let s1 = self.start_unixstamp() as u128;
        let e1 = self.end_unixtime_exclusive();

        let s2 = other.start_unixstamp() as u128;
        let e2 = other.end_unixtime_exclusive();

        let s_intersect = s1.max(s2);
        let e_intersect = e1.min(e2);

        if s_intersect >= e_intersect {
            return None;
        }
        let mut intersected = TemporalId::new(1, [s_intersect as u64, (e_intersect - 1) as u64])
            .expect("Intersection logic should never overflow or have i=0");

        intersected.optimize_i();

        Some(intersected)
    }

    /// 相手の[TemporalId]との差集合（self - other）を計算し、イテレータとして返します。
    /// 結果は 0個（完全に消滅）、1個（一部削られる or そのまま）、2個（分断される）のいずれかになります。
    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let s_a = self.start_unixstamp();
        let e_a = self.end_unixstamp_inclusive();

        let s_b = other.start_unixstamp();
        let e_b = other.end_unixstamp_inclusive();

        let mut results = Vec::new();

        if e_a < s_b || s_a > e_b {
            results.push(self.clone());
            return results.into_iter();
        }

        if s_b <= s_a && e_b >= e_a {
            return results.into_iter();
        }

        if s_a < s_b {
            let mut left = TemporalId::new(1, [s_a, s_b - 1])
                .expect("Left difference logic is mathematically safe");
            left.optimize_i();
            results.push(left);
        }

        if e_a > e_b {
            let mut right = TemporalId::new(1, [e_b + 1, e_a])
                .expect("Right difference logic is mathematically safe");
            right.optimize_i();
            results.push(right);
        }

        results.into_iter()
    }
}
