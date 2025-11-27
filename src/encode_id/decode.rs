use crate::{
    bit_vec::BitVec,
    encode_id::EncodeID,
    space_time_id::{F_MAX, SpaceTimeID},
    time_interval::TimeInterval,
};

impl EncodeID {
    /// EncodeIDをSpaceTimeIDにデコードする
    ///
    /// 各次元（f, x, y, t）のBitVecをデコードし、対応するズームレベルと値を取得する。
    /// 空間次元（f, x, y）の最大ズームレベルを使用して座標範囲を計算する。
    ///
    /// # 戻り値
    ///
    /// デコードされた`SpaceTimeID`構造体
    pub fn decode(&self) -> SpaceTimeID {
        let (f_z, f_v) = to_segment_f(&self.f);
        let (x_z, x_v) = to_segment_xy(&self.x);
        let (y_z, y_v) = to_segment_xy(&self.y);
        let (t_z, t_v) = to_segment_xy(&self.t);

        let max_z = f_z.max(x_z).max(y_z);

        let f = if max_z == f_z {
            [f_v, f_v]
        } else {
            let k = 2_i64.pow((max_z - f_z) as u32);
            [f_v * k, (f_v + 1) * k - 1]
        };

        let x = if max_z == x_z {
            [x_v, x_v]
        } else {
            let k = 2_u64.pow((max_z - x_z) as u32);
            [x_v * k, (x_v + 1) * k - 1]
        };

        let y = if max_z == y_z {
            [y_v, y_v]
        } else {
            let k = 2_u64.pow((max_z - y_z) as u32);
            [y_v * k, (y_v + 1) * k - 1]
        };

        let t = {
            let k = 2_u64.pow((63 - t_z) as u32);
            [t_v * k, (t_v + 1) * k - 1]
        };

        SpaceTimeID {
            z: max_z,
            f,
            x,
            y,
            t,
        }
    }

    /// 時間範囲をTimeIntervalとして取得
    ///
    /// BitVecからデコードされた時間範囲をTimeIntervalとして返す。
    /// これにより、効率的な区間演算（包含、重なり、差分など）が可能になる。
    pub fn time_interval(&self) -> TimeInterval {
        let (t_z, t_v) = to_segment_xy(&self.t);
        let k = 2_u64.pow((63 - t_z) as u32);
        let start = t_v * k;
        let end = (t_v + 1) * k - 1;
        TimeInterval::new(start, end)
    }

    /// 二つのEncodeIDの時間範囲が重なるかどうかを判定
    ///
    /// TimeIntervalを使用して効率的に判定を行う
    pub fn time_overlaps(&self, other: &Self) -> bool {
        self.time_interval().overlaps(&other.time_interval())
    }

    /// 二つのEncodeIDの時間範囲の共通部分を取得
    ///
    /// 重ならない場合はNoneを返す
    pub fn time_intersection(&self, other: &Self) -> Option<TimeInterval> {
        self.time_interval().intersection(&other.time_interval())
    }
}

pub(crate) fn to_segment_xy(bitmask: &BitVec) -> (u8, u64) {
    let bytes = &bitmask.0;
    let total_bits = bytes.len() * 8;
    let total_layers = (total_bits + 1) / 2;

    let mut uxy: u64 = 0;
    let mut max_z: i32 = -1; // 見つかった最大のz

    // now_z=0 から順に処理
    for now_z in 0..total_layers {
        let index = (now_z * 2) / 8;
        let in_index = now_z % 4;

        let byte = bytes[index];
        let valid = (byte >> (7 - in_index * 2)) & 1;
        let branch = (byte >> (6 - in_index * 2)) & 1;

        if valid == 1 {
            max_z = now_z as i32;
            // now_z の位置に branch を配置
            uxy |= (branch as u64) << now_z;
        }
    }

    // uxy を反転（ビットの並びを逆にする）
    let final_z = max_z as u8;
    let mut reversed_uxy = 0u64;
    for i in 0..=final_z {
        let bit = (uxy >> i) & 1;
        reversed_uxy |= bit << (final_z - i);
    }

    (final_z, reversed_uxy)
}

pub(crate) fn to_segment_f(bitmask: &BitVec) -> (u8, i64) {
    let (z, f) = to_segment_xy(bitmask);

    //仮に負の範囲の場合
    if *bitmask
        .0
        .first()
        .expect("Internal error: BitVec is empty in invert_bitmask_f")
        >= 0b11000000
    {
        return (z, -(f as i64) + F_MAX[z as usize]);
    } else {
        return (z, f as i64);
    }
}
