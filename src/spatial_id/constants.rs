///本ライブラリで扱うことができる最大のズームレベル
//理論上は63まで可能だが、境界オーバーフローの検証が十分ではないため60で固定
pub const MAX_ZOOM_LEVEL: usize = 31;

/// 各ズームレベルにおけるXYインデックスの最大値
///
/// ```
/// # use kasane_logic::spatial_id::single::SingleId;
/// # use kasane_logic::spatial_id::constants::XY_MAX;
/// # use crate::kasane_logic::spatial_id::SpatialId;
/// let id = SingleId::new(4, 6, 9, 10).unwrap();
/// assert_eq!(id.max_xy(), XY_MAX[4]);
/// ```
pub const XY_MAX: [u32; MAX_ZOOM_LEVEL + 1] = [
    0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535, 131071,
    262143, 524287, 1048575, 2097151, 4194303, 8388607, 16777215, 33554431, 67108863, 134217727,
    268435455, 536870911, 1073741823, 2147483647,
];

/// 各ズームレベルにおけるFインデックスの最小値
///
/// ```
/// # use kasane_logic::spatial_id::single::SingleId;
/// # use kasane_logic::spatial_id::constants::F_MIN;
/// # use crate::kasane_logic::spatial_id::SpatialId;
/// let id = SingleId::new(4, 6, 9, 10).unwrap();
/// assert_eq!(id.min_f(), F_MIN[4]);
/// ```
pub const F_MIN: [i32; MAX_ZOOM_LEVEL + 1] = [
    -1,
    -2,
    -4,
    -8,
    -16,
    -32,
    -64,
    -128,
    -256,
    -512,
    -1024,
    -2048,
    -4096,
    -8192,
    -16384,
    -32768,
    -65536,
    -131072,
    -262144,
    -524288,
    -1048576,
    -2097152,
    -4194304,
    -8388608,
    -16777216,
    -33554432,
    -67108864,
    -134217728,
    -268435456,
    -536870912,
    -1073741824,
    -2147483648,
];

/// 各ズームレベルにおけるFインデックスの最大値
///
/// ```
/// # use kasane_logic::spatial_id::single::SingleId;
/// # use kasane_logic::spatial_id::constants::F_MAX;
/// # use crate::kasane_logic::spatial_id::SpatialId;
/// let id = SingleId::new(4, 6, 9, 10).unwrap();
/// assert_eq!(id.max_f(), F_MAX[4]);
/// ```
pub const F_MAX: [i32; MAX_ZOOM_LEVEL + 1] = [
    0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535, 131071,
    262143, 524287, 1048575, 2097151, 4194303, 8388607, 16777215, 33554431, 67108863, 134217727,
    268435455, 536870911, 1073741823, 2147483647,
];
