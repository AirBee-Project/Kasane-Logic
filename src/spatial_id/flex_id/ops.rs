use crate::{FlexId, Side, SpatialId};

impl FlexId {
    /// 相手の[FlexId]との差集合（self - other）を計算し、イテレータとして返します。
    /// 空間と時間の両方を考慮し、相手にくり抜かれた「残りの領域」を過不足なく細かい FlexId に分割して返します。
    pub fn difference(&self, other: &FlexId) -> impl Iterator<Item = FlexId> {
        let mut results = Vec::new();

        // 1. 交差（Intersection）を取得する
        let intersect = match self.intersection(other) {
            Some(i) => i,
            None => {
                // まったく重なっていなければ、自分がそのまま丸ごと残る
                results.push(self.clone());
                return results.into_iter();
            }
        };

        // 2. 自分と交差領域が完全に一致する場合、差分は空（完全にくり抜かれた）
        if self == &intersect {
            return results.into_iter();
        }

        // 3. 空間的に分割しながら差分（外側の削りカス）を抽出していく
        let mut current = self.clone();

        // F軸の分割：交差領域の解像度に達するまで自分を割り続ける
        while current.f_zoomlevel < intersect.f_zoomlevel {
            let lower = current.f_split(Side::Lower).unwrap();
            let upper = current.f_split(Side::Upper).unwrap();

            // 交差領域が含まれる方（intersectと交差する方）を次に進め、
            // 含まれない方（外側）は差分の一部として結果リストに退避する
            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        // X軸の分割
        while current.x_zoomlevel < intersect.x_zoomlevel {
            let lower = current.x_split(Side::Lower).unwrap();
            let upper = current.x_split(Side::Upper).unwrap();

            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        // Y軸の分割
        while current.y_zoomlevel < intersect.y_zoomlevel {
            let lower = current.y_split(Side::Lower).unwrap();
            let upper = current.y_split(Side::Upper).unwrap();

            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        // 4. この時点で、current の「空間（F, X, Y）」は intersect と完全に一致している状態。
        // つまり空間の差分はすべて results に退避された。
        // 最後に、同じ空間位置における「時間（TemporalId）」の差分を計算して適用する。
        for t_diff in current.temporal().difference(other.temporal()) {
            results.push(FlexId {
                f_zoomlevel: current.f_zoomlevel,
                f_index: current.f_index,
                x_zoomlevel: current.x_zoomlevel,
                x_index: current.x_index,
                y_zoomlevel: current.y_zoomlevel,
                y_index: current.y_index,
                temporal_id: t_diff,
            });
        }

        // イテレータとして返す
        results.into_iter()
    }

    /// 2つのFlexIdの重なっている領域（Intersection）を計算して返します。
    /// 重なりがない場合は None を返します。
    pub fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let (f_z, f_i) = Self::intersect_axis_i32(
            self.f_zoomlevel,
            self.f_index,
            other.f_zoomlevel,
            other.f_index,
        )?;

        let (x_z, x_i) = Self::intersect_axis_u32(
            self.x_zoomlevel,
            self.x_index,
            other.x_zoomlevel,
            other.x_index,
        )?;

        let (y_z, y_i) = Self::intersect_axis_u32(
            self.y_zoomlevel,
            self.y_index,
            other.y_zoomlevel,
            other.y_index,
        )?;

        let temporal_id = self.temporal().intersection(other.temporal())?;

        Some(FlexId {
            f_zoomlevel: f_z,
            f_index: f_i,
            x_zoomlevel: x_z,
            x_index: x_i,
            y_zoomlevel: y_z,
            y_index: y_i,
            temporal_id,
        })
    }

    fn intersect_axis_i32(z1: u8, i1: i32, z2: u8, i2: i32) -> Option<(u8, i32)> {
        let (deep_z, deep_i, shallow_z, shallow_i) = if z1 > z2 {
            (z1, i1, z2, i2)
        } else {
            (z2, i2, z1, i1)
        };

        let shift = deep_z - shallow_z;

        if (deep_i >> shift) == shallow_i {
            Some((deep_z, deep_i))
        } else {
            None
        }
    }

    fn intersect_axis_u32(z1: u8, i1: u32, z2: u8, i2: u32) -> Option<(u8, u32)> {
        let (deep_z, deep_i, shallow_z, shallow_i) = if z1 > z2 {
            (z1, i1, z2, i2)
        } else {
            (z2, i2, z1, i1)
        };

        let shift = deep_z - shallow_z;
        if (deep_i >> shift) == shallow_i {
            Some((deep_z, deep_i))
        } else {
            None
        }
    }
}
