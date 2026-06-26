use crate::SingleId;
use crate::spatial_id::zoom_level::ZoomLevel;

impl SingleId {
    /// 自身を前方一致検索しやすい固定長バイト列に変換する。
    ///
    /// この関数は、空間パスを上位 91 ビットに、ズームレベル `z` を下位 5 ビットに配置した
    /// 12 バイトのキーへ変換する。空間パスは `f` / `x` / `y` の各ビットを上位から順に交互に並べたものである。
    /// `BTreeMap` や順序付きKVSで Range スキャンを行うためのキーとして利用することを想定している。
    ///
    /// # 動作コスト
    ///
    /// ズームレベル `z` に比例して計算量が増加する。最大でも `ZoomLevel::MAX` に比例する。
    ///
    /// # 動作例
    ///
    /// エンコード結果の取得:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let bytes = id.spatial_encode();
    /// assert_eq!(bytes.len(), 12);
    /// ```
    pub fn spatial_encode(&self) -> [u8; 12] {
        let mut v: u128 = 0;
        let z = self.z();
        let f_shifted = (self.f() - ZoomLevel::new(z).unwrap().f_min()) as u128;
        let x = self.x() as u128;
        let y = self.y() as u128;

        let mut shift = 127;
        v |= ((f_shifted >> z) & 1) << shift;
        shift -= 1;

        for i in (0..z).rev() {
            v |= ((f_shifted >> i) & 1) << shift;
            shift -= 1;
            v |= ((x >> i) & 1) << shift;
            shift -= 1;
            v |= ((y >> i) & 1) << shift;
            shift -= 1;
        }

        v |= (z as u128) << 32;

        let mut bytes = [0u8; 12];
        bytes.copy_from_slice(&v.to_be_bytes()[0..12]);
        bytes
    }

    /// 前方一致検索の終端に使う最大値の固定長バイト列を返す。
    ///
    /// `spatial_encode()` が返すキーと組み合わせることで、`self` が含む子孫IDを
    /// `range(self.spatial_encode()..=self.spatial_encode_prefix_max())` のように探索できる。
    /// 返す値は、空間パス部分の未使用下位ビットをすべて 1 にしたものであり、ズームレベル `z` は維持する。
    ///
    /// # 動作コスト
    ///
    /// ズームレベル `z` に比例して計算量が増加する。最大でも `ZoomLevel::MAX` に比例する。
    ///
    /// # 動作例
    ///
    /// 範囲検索の終端キーの取得:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let end = id.spatial_encode_prefix_max();
    /// assert_eq!(end.len(), 12);
    /// ```
    pub fn spatial_encode_prefix_max(&self) -> [u8; 12] {
        let mut v: u128 = 0;
        let z = self.z();
        let f_shifted = (self.f() - ZoomLevel::new(z).unwrap().f_min()) as u128;
        let x = self.x() as u128;
        let y = self.y() as u128;

        let mut shift = 127;
        v |= ((f_shifted >> z) & 1) << shift;
        shift -= 1;

        for i in (0..z).rev() {
            v |= ((f_shifted >> i) & 1) << shift;
            shift -= 1;
            v |= ((x >> i) & 1) << shift;
            shift -= 1;
            v |= ((y >> i) & 1) << shift;
            shift -= 1;
        }

        if shift >= 37 {
            let mask = ((1u128 << (shift - 36)) - 1) << 37;
            v |= mask;
        }

        let mut bytes = [0u8; 12];
        bytes.copy_from_slice(&v.to_be_bytes()[0..12]);
        bytes
    }

    /// 固定長バイト列から [`SingleId`] を復元する。
    ///
    /// `spatial_encode()` が生成した 12 バイトのキーを元に、元の `SingleId` を再構築する。
    /// エンコードに使われているズームレベルが不正な場合はエラーを返す。
    ///
    /// # バリデーション
    ///
    /// - バイト列に含まれるズームレベルが [`crate::ZoomLevel::MAX`] を超える場合、[`crate::SpatialIdError::ZOutOfRange`] を返す。
    /// - 復元した `f` / `x` / `y` が範囲外になる場合は、各種範囲外エラーを返す。
    ///
    /// # 動作コスト
    ///
    /// ズームレベル `z` に比例して計算量が増加する。最大でも `ZoomLevel::MAX` に比例する。
    ///
    /// # 動作例
    ///
    /// 往復変換:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, 6, 9, 10).unwrap();
    /// let decoded = SingleId::spatial_decode(&id.spatial_encode()).unwrap();
    /// assert_eq!(decoded, id);
    /// ```
    pub fn spatial_decode(bytes: &[u8; 12]) -> Result<Self, crate::error::Error> {
        let mut v_bytes = [0u8; 16];
        v_bytes[0..12].copy_from_slice(bytes);
        let v = u128::from_be_bytes(v_bytes);
        let z_mask = 0b11111 << 32;
        let z = ((v & z_mask) >> 32) as u8;
        if z > ZoomLevel::MAX.get() {
            return Err(crate::error::Error::SpatialId(
                crate::SpatialIdError::ZOutOfRange { z },
            ));
        }

        let mut shift = 127;
        let mut f_shifted: u64 = 0;
        let mut x: u32 = 0;
        let mut y: u32 = 0;

        f_shifted |= ((v >> shift) & 1) as u64;
        shift -= 1;

        for _ in 0..z {
            f_shifted = (f_shifted << 1) | (((v >> shift) & 1) as u64);
            shift -= 1;
            x = (x << 1) | (((v >> shift) & 1) as u32);
            shift -= 1;
            y = (y << 1) | (((v >> shift) & 1) as u32);
            shift -= 1;
        }

        let f = (f_shifted as i64 + ZoomLevel::new(z).unwrap().f_min() as i64) as i32;

        crate::spatial_id::single_id::SingleId::new(z, f, x, y)
    }
}
