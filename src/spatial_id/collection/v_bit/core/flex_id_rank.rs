use crate::{FlexId, MAX_ZOOM_LEVEL};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FlexIdRank {
    pub f: i32,
    pub x: u32,
    pub y: u32,
}

impl FlexIdRank {
    /// FlexIdRankを上位64bit(キー)と下位32bit(Bitmap用)に分割する
    #[inline]
    pub fn split(&self) -> (u64, u32) {
        let high_64 = ((self.f as u32) as u64) << 32 | (self.x as u64);
        let low_32 = self.y;
        (high_64, low_32)
    }

    #[inline]
    pub fn from_parts(high_64: u64, low_32: u32) -> Self {
        Self {
            f: (high_64 >> 32) as u32 as i32,
            x: (high_64 & 0xFFFFFFFF) as u32,
            y: low_32,
        }
    }
}

impl FlexId {
    /// FlexIdが表す空間領域のうち、最も左下・左上（原点に最も近い最小インデックス）にあたる
    /// 最小単位ボクセル（MAX_ZOOM_LEVELにおける開始位置）の座標を計算し、
    /// それをもとにZ階数曲線（Morton Code）を用いた FlexIdRank を発行。
    ///
    /// 重なりのないFlexIdの集合（Set）において、このRankは完全に一意となり、
    /// 空間的な近接性を保ったBTree等での高速なソート・探索を可能にする。
    pub fn flex_id_rank(&self) -> FlexIdRank {
        let (f_z, f_idx) = self.f();
        let (x_z, x_idx) = self.x();
        let (y_z, y_idx) = self.y();

        // 基準となる最大ズームレベルまでの差分（シフト量）を計算
        let scale_f = MAX_ZOOM_LEVEL as u8 - f_z;
        let scale_x = MAX_ZOOM_LEVEL as u8 - x_z;
        let scale_y = MAX_ZOOM_LEVEL as u8 - y_z;

        // MAX_ZOOM_LEVEL における「領域の開始インデックス（原点側）」を算出
        let f_start = f_idx << scale_f;
        let x_start = x_idx << scale_x;
        let y_start = y_idx << scale_y;

        // XとYの開始インデックスを元に 64bit の Morton Code を生成
        let morton = morton_encode(x_start, y_start);

        FlexIdRank {
            f: f_start,
            // 64bitのMorton Codeを上位・下位の32bitずつに分割して格納
            x: (morton >> 32) as u32,
            y: (morton & 0xFFFFFFFF) as u32,
        }
    }
}

/// 32bitの整数の各ビットの間に1ビットの隙間を空ける
#[inline]
fn part1by1(mut n: u64) -> u64 {
    n &= 0x00000000ffffffff;
    n = (n | (n << 16)) & 0x0000FFFF0000FFFF;
    n = (n | (n << 8)) & 0x00FF00FF00FF00FF;
    n = (n | (n << 4)) & 0x0F0F0F0F0F0F0F0F;
    n = (n | (n << 2)) & 0x3333333333333333;
    n = (n | (n << 1)) & 0x5555555555555555;
    n
}

#[inline]
fn morton_encode(x: u32, y: u32) -> u64 {
    part1by1(x as u64) | (part1by1(y as u64) << 1)
}
