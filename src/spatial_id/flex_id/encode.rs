use crate::{FlexId, spatial_id::zoom_level::ZoomLevel};

impl FlexId {
    /// 自身を固定長 14 バイトのバイト列に変換する。
    ///
    /// # フォーマット
    ///
    /// ```text
    /// byte 0      : [zf: 5bit][zx: 5bit 上位3bit]
    /// byte 1      : [zx: 5bit 下位2bit][zy: 5bit][padding: 1bit]
    /// byte 2..=5  : f_shifted (u32, big-endian)
    ///               f_shifted = f_index - f_min(zf)  （0 以上の符号なし整数として格納）
    /// byte 6..=9  : x_index (u32, big-endian)
    /// byte 10..=13: y_index (u32, big-endian)
    /// ```
    ///
    /// ズームレベル 3 つが各 5 ビット（合計 15 ビット）で 2 バイトに収まり、
    /// 残りの 3 フィールドが各 4 バイト（big-endian）で続く。合計 14 バイト固定長。
    ///
    /// # 動作例
    ///
    /// ```
    /// # use kasane_logic::FlexId;
    /// let id = FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// let bytes = id.spatial_encode();
    /// assert_eq!(bytes.len(), 14);
    /// let decoded = FlexId::spatial_decode(&bytes).unwrap();
    /// assert_eq!(decoded, id);
    /// ```
    pub fn spatial_encode(&self) -> [u8; 14] {
        let zf = self.f_zoomlevel();
        let zx = self.x_zoomlevel();
        let zy = self.y_zoomlevel();

        // f_index をオフセット付きの符号なし整数へ変換
        let f_min = ZoomLevel::new(zf).unwrap().f_min();
        let f_shifted = (self.f_index() - f_min) as u32;

        // ズームレベル 3 つを 15 ビットに詰める
        // byte 0: [zf(5bit)][zx(5bit) 上位3bit]
        // byte 1: [zx(5bit) 下位2bit][zy(5bit)][padding(1bit)=0]
        let header = ((zf as u16) << 10) | ((zx as u16) << 5) | (zy as u16);
        let byte0 = (header >> 7) as u8;
        let byte1 = ((header & 0x7F) << 1) as u8; // 下位 7bit を左に 1bit シフト（padding bit = 0）

        let mut out = [0u8; 14];
        out[0] = byte0;
        out[1] = byte1;
        out[2..6].copy_from_slice(&f_shifted.to_be_bytes());
        out[6..10].copy_from_slice(&self.x_index().to_be_bytes());
        out[10..14].copy_from_slice(&self.y_index().to_be_bytes());
        out
    }

    /// 固定長 14 バイトのバイト列から [`FlexId`] を復元する。
    ///
    /// [`spatial_encode`](Self::spatial_encode) が生成したバイト列を元に [`FlexId`] を再構築する。
    ///
    /// # バリデーション
    ///
    /// - ズームレベルが [`ZoomLevel::MAX`] を超える場合は [`crate::SpatialIdError::ZOutOfRange`] を返す。
    /// - 復元した `f_index` / `x_index` / `y_index` が各ズームレベルの範囲外の場合は
    ///   各種範囲外エラーを返す。
    ///
    /// # 動作例
    ///
    /// ```
    /// # use kasane_logic::FlexId;
    /// let id = FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// let decoded = FlexId::spatial_decode(&id.spatial_encode()).unwrap();
    /// assert_eq!(decoded, id);
    /// ```
    pub fn spatial_decode(bytes: &[u8; 14]) -> Result<Self, crate::error::Error> {
        // ヘッダー (15 ビット) を復元
        let header = ((bytes[0] as u16) << 7) | ((bytes[1] as u16) >> 1);
        let zf = ((header >> 10) & 0x1F) as u8;
        let zx = ((header >> 5) & 0x1F) as u8;
        let zy = (header & 0x1F) as u8;

        // ズームレベルのバリデーション
        if zf > ZoomLevel::MAX.get() {
            return Err(crate::SpatialIdError::ZOutOfRange { z: zf }.into());
        }
        if zx > ZoomLevel::MAX.get() {
            return Err(crate::SpatialIdError::ZOutOfRange { z: zx }.into());
        }
        if zy > ZoomLevel::MAX.get() {
            return Err(crate::SpatialIdError::ZOutOfRange { z: zy }.into());
        }

        let f_shifted = u32::from_be_bytes(bytes[2..6].try_into().unwrap());
        let x_index = u32::from_be_bytes(bytes[6..10].try_into().unwrap());
        let y_index = u32::from_be_bytes(bytes[10..14].try_into().unwrap());

        // f_index をオフセットから実際の値へ戻す
        let f_min = ZoomLevel::new(zf).unwrap().f_min();
        let f_index = f_shifted as i64 + f_min as i64;

        FlexId::new(zf, f_index as i32, zx, x_index, zy, y_index)
    }
}
