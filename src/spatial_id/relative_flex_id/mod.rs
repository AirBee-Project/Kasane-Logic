use crate::{
    Error, FlexId, SpatialIdError,
    spatial_id::{
        dimension::Dimension,
        zoom_level::{TZoomLevel, ZoomLevel},
    },
};

/// ある祖先の [FlexId] を起点とした相対的な位置を表す[FlexId]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativeFlexId(FlexId);

impl FlexId {
    /// `ancestor` を原点とした[RelativeFlexId]を返す。
    /// `ancestor`が完全に自身を包含していなければ [`SpatialIdError::NotAncestor`] を返す。
    pub fn relative_to(&self, ancestor: &FlexId) -> Result<RelativeFlexId, Error> {
        if !ancestor.contains(self) {
            return Err(SpatialIdError::NotAncestor.into());
        }
        // 各次元で、祖先からの深さと、その深さぶんの下位ビットを求める
        let [(fz, f), (xz, x), (yz, y), (tz, t)] =
            zip_dimensions(self, ancestor, |(z, i), (az, ai)| {
                let dz = z - az;
                (dz, i - (ai << dz))
            });
        // SAFETY: 祖先に含まれるので、深さは自身のズーム以下、下位ビットは `0..2^深さ` に収まる
        let relative = unsafe { FlexId::new_unchecked(fz, f as i32, xz, x as u32, yz, y as u32) }
            .with_time_segment(tz, t as u64);
        Ok(RelativeFlexId(relative))
    }
}

impl RelativeFlexId {
    /// 祖先から F 次元において何段深いかを返す。
    pub fn f_depth(&self) -> u8 {
        self.0.f_zoomlevel()
    }

    /// 祖先から X 次元において何段深いかを返す。
    pub fn x_depth(&self) -> u8 {
        self.0.x_zoomlevel()
    }

    /// 祖先から Y 次元において何段深いかを返す。
    pub fn y_depth(&self) -> u8 {
        self.0.y_zoomlevel()
    }

    /// 祖先から T 次元において何段深いかを返す。
    pub fn t_depth(&self) -> u8 {
        self.0.t_zoomlevel()
    }

    /// 祖先から `dimension`において何段深いかを返す。
    pub fn depth_on(self, dimension: Dimension) -> u8 {
        self.0.zoomlevel_on(dimension)
    }

    /// 祖先より深くなっている次元の集合（[`Dimension::bit`] の OR）。
    pub fn deeper_dimensions(&self) -> u8 {
        Dimension::mask(|d| self.depth_on(d) > 0)
    }

    /// `ancestor` を原点として、[FlexId] を復元する。
    /// どれかの次元で最大ズームを超えるなら [`SpatialIdError::ZOutOfRange`] を返す。
    pub fn to_absolute(self, ancestor: &FlexId) -> Result<FlexId, Error> {
        for (((az, _), (rz, _)), max) in dimensions(ancestor)
            .into_iter()
            .zip(dimensions(&self.0))
            .zip(MAX_ZOOM)
        {
            if az + rz > max {
                return Err(SpatialIdError::ZOutOfRange { z: az + rz }.into());
            }
        }
        let [(fz, f), (xz, x), (yz, y), (tz, t)] =
            zip_dimensions(ancestor, &self.0, |(az, ai), (rz, ri)| {
                (az + rz, (ai << rz) + ri)
            });
        // SAFETY: ズームは上限以下。インデックスは祖先の区間を `2^深さ` 等分したうちの1つなので、そのズームの範囲に収まる
        let absolute = unsafe { FlexId::new_unchecked(fz, f as i32, xz, x as u32, yz, y as u32) }
            .with_time_segment(tz, t as u64);
        Ok(absolute)
    }

    /// [`encode`](Self::encode) / [`decode`](Self::decode) が扱う固定長バイト列の長さ。
    pub const ENCODED_LEN: usize = 20;

    /// 自身を固定長バイト列に変換する。
    ///
    /// # フォーマット
    ///
    /// ```text
    /// byte 0..=3  : F / X / Y / T の深さ
    /// byte 4..=19 : F / X / Y / T のインデックスを、各次元の最大ズームのビット幅ずつ
    ///               下位から詰めた u128（little-endian）
    /// ```
    ///
    /// インデックスは `0..2^深さ` なので、各次元の最大ズームのビット幅に収まる。
    ///
    /// # 動作例
    ///
    /// ```
    /// # use kasane_logic::{FlexId, RelativeFlexId};
    /// let ancestor = FlexId::new(2, 1, 1, 0, 3, 5).unwrap();
    /// let relative = FlexId::new(5, 12, 4, 7, 3, 5).unwrap().relative_to(&ancestor).unwrap();
    /// assert_eq!(RelativeFlexId::decode(&relative.encode()), Ok(relative));
    /// ```
    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        let dimensions = dimensions(&self.0);
        let index = dimensions
            .iter()
            .zip(MAX_ZOOM)
            .rev()
            .fold(0u128, |packed, (&(_, i), bits)| packed << bits | i as u128);
        let mut out = [0; Self::ENCODED_LEN];
        out[..4].copy_from_slice(&dimensions.map(|(z, _)| z));
        out[4..].copy_from_slice(&index.to_le_bytes());
        out
    }

    /// 固定長バイト列から [`RelativeFlexId`] を復元する（[`encode`](Self::encode) の逆変換）。
    ///
    /// 深さが各次元の最大ズームを超える、またはインデックスが `0..2^深さ` に収まらないならエラーを返す。
    pub fn decode(bytes: &[u8; Self::ENCODED_LEN]) -> Result<Self, Error> {
        let [fz, xz, yz, tz, ..] = *bytes;
        let mut index = u128::from_le_bytes(bytes[4..].try_into().unwrap());
        let [f, x, y, t] = MAX_ZOOM.map(|bits| {
            let i = index & ((1 << bits) - 1);
            index >>= bits;
            i as u64
        });
        // ビット幅が最大ズーム以下なので、空間3軸のインデックスは i32 / u32 に収まる
        let relative = FlexId::new(fz, f as i32, xz, x as u32, yz, y as u32)?.with_time(tz, t)?;
        Ok(RelativeFlexId(relative))
    }
}

/// F / X / Y / T の各次元の最大ズーム。
const MAX_ZOOM: [u8; 4] = [
    ZoomLevel::MAX.get(),
    ZoomLevel::MAX.get(),
    ZoomLevel::MAX.get(),
    TZoomLevel::MAX.get(),
];

/// F / X / Y / T の各次元の `(ズーム, インデックス)`。
type Dimensions = [(u8, i64); 4];

fn dimensions(id: &FlexId) -> Dimensions {
    [
        (id.f_zoomlevel(), id.f_index() as i64),
        (id.x_zoomlevel(), id.x_index() as i64),
        (id.y_zoomlevel(), id.y_index() as i64),
        (id.t_zoomlevel(), id.t() as i64),
    ]
}

/// 2つの ID の各次元を `f` で組み合わせる。
fn zip_dimensions(
    a: &FlexId,
    b: &FlexId,
    f: impl Fn((u8, i64), (u8, i64)) -> (u8, i64),
) -> Dimensions {
    let (a, b) = (dimensions(a), dimensions(b));
    core::array::from_fn(|i| f(a[i], b[i]))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 自身を包含しない ID を祖先に渡すとエラーになる。
    #[test]
    fn relative_to_non_ancestor_is_error() {
        let id = FlexId::new(3, 1, 3, 2, 3, 3).unwrap();
        let sibling = FlexId::new(3, 0, 3, 2, 3, 3).unwrap();
        let child = FlexId::new(4, 2, 3, 2, 3, 3).unwrap();
        assert_eq!(
            id.relative_to(&sibling),
            Err(SpatialIdError::NotAncestor.into())
        );
        assert!(id.relative_to(&child).is_err());
    }

    /// 相対 ID は、作ったときの祖先から絶対 ID へ戻せる。
    #[test]
    fn to_absolute_restores_original() {
        let ancestor = FlexId::new(2, 1, 1, 0, 3, 5).unwrap();
        let id = FlexId::new(5, 12, 4, 7, 3, 5).unwrap();
        let relative = id.relative_to(&ancestor).unwrap();
        assert_eq!(relative.to_absolute(&ancestor), Ok(id));
    }

    /// 深い祖先に当てはめて最大ズームを超えるならエラーになる。
    #[test]
    fn to_absolute_beyond_max_zoom_is_error() {
        let deep = FlexId::new(30, 0, 0, 0, 0, 0).unwrap();
        let relative = deep.relative_to(&FlexId::UPPER_MAX).unwrap();
        assert_eq!(
            relative.to_absolute(&deep),
            Err(SpatialIdError::ZOutOfRange { z: 60 }.into())
        );
    }

    /// 各次元で最も深く、インデックスが最大の相対 ID も符号化して戻せる。
    #[cfg(feature = "temporal_id")]
    #[test]
    fn encode_round_trips_deepest() {
        let max = (1u32 << 30) - 1;
        let deepest = FlexId::new(30, max as i32, 30, max, 30, max)
            .unwrap()
            .with_time(TZoomLevel::MAX.get(), (1 << 35) - 1)
            .unwrap();
        let relative = deepest.relative_to(&FlexId::UPPER_MAX).unwrap();
        assert_eq!(RelativeFlexId::decode(&relative.encode()), Ok(relative));
    }

    /// クレートルートから RelativeFlexId および Dimension が利用でき、深さ取得が正しく動作することを検証。
    #[test]
    fn test_relative_flex_id_depths_and_root_export() {
        use crate::{Dimension, RelativeFlexId};
        let ancestor = FlexId::new(2, 1, 1, 0, 3, 5).unwrap();
        let descendant = FlexId::new(5, 12, 4, 7, 3, 5).unwrap();
        let rel: RelativeFlexId = descendant.relative_to(&ancestor).unwrap();
        assert_eq!(rel.f_depth(), 3);
        assert_eq!(rel.x_depth(), 3);
        assert_eq!(rel.y_depth(), 0);
        assert_eq!(rel.depth_on(Dimension::F), 3);
        assert_eq!(rel.depth_on(Dimension::X), 3);
        assert_eq!(rel.depth_on(Dimension::Y), 0);
        assert_eq!(rel.to_absolute(&ancestor).unwrap(), descendant);
    }
}
