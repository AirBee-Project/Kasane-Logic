use serde::Serialize;
pub mod format;
pub mod random;
pub mod z_range;

use crate::{
    error::Error,
    space_time_id::z_range::{F_MAX, F_MIN, XY_MAX},
};

/// 時空間IDを表す構造体
///
/// 3次元空間（F: 高度、X: 経度、Y: 緯度）と時間軸（T）を持つ時空間位置を表現する。
/// 各次元は範囲として表現され、階層的なズームレベル（Z）を持つ。
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceTimeId {
    pub z: u8,
    pub f: [i64; 2],
    pub x: [u64; 2],
    pub y: [u64; 2],
    pub i: u32,
    pub t: [u64; 2],
}

impl SpaceTimeId {
    /// 時空間IDを生成する
    ///
    /// 各座標値の範囲を確認し、正規化した上で時空間IDを作成する。
    /// None を指定した場合は、その次元の最小値または最大値が自動的に設定される。
    ///
    /// # 引数
    /// * `z` - ズームレベル (0..=60)
    /// * `f` - F座標（高度）の範囲 [開始, 終了]
    /// * `x` - X座標（経度）の範囲 [開始, 終了]
    /// * `y` - Y座標（緯度）の範囲 [開始, 終了]
    /// * `i` - インデックス番号
    /// * `_t` - 時間の範囲 [開始, 終了] (未実装)
    ///
    /// # エラー
    /// 座標値が範囲外の場合や、ズームレベルが無効な場合にエラーを返す。
    pub fn new(
        z: u8,
        f: [Option<i64>; 2],
        x: [Option<u64>; 2],
        y: [Option<u64>; 2],
        i: u32,
        _t: [Option<u32>; 2],
    ) -> Result<Self, Error> {
        if z > 60 {
            return Err(Error::ZoomLevelOutOfRange { zoom_level: z });
        }

        let f_max = F_MAX[z as usize];
        let f_min = F_MIN[z as usize];
        let xy_max = XY_MAX[z as usize];

        // 空間の次元を全て値に変換
        let new_f = normalize_dimension(f, f_min, f_max, valid_range_f, z)?;
        let new_x = normalize_dimension(x, 0, xy_max, valid_range_x, z)?;
        let new_y = normalize_dimension(y, 0, xy_max, valid_range_y, z)?;

        //時間軸の順番を入れ替え
        //Todo時間に関する処理を行う
        //いったん、3次元の処理を優先的に行う

        Ok(SpaceTimeId {
            z,
            f: new_f,
            x: new_x,
            y: new_y,
            i,
            t: [0, u64::MAX],
        })
    }
}

/// 次元の値が正しいかを判定し、正規化する
///
/// Option型の次元範囲を検証し、Noneの場合は最小値/最大値で埋めて、
/// 開始と終了の順序を正しくする。
///
/// # 引数
/// * `dim` - 次元の範囲 [開始, 終了] (Option型)
/// * `min` - この次元の最小値
/// * `max` - この次元の最大値
/// * `validate` - 検証関数
/// * `z` - ズームレベル
fn normalize_dimension<T>(
    dim: [Option<T>; 2],
    min: T,
    max: T,
    validate: impl Fn(T, T, T, u8) -> Result<(), Error>,
    z: u8,
) -> Result<[T; 2], Error>
where
    T: PartialOrd + Copy,
{
    //値が範囲内なのかをチェックする
    if let Some(s) = dim[0] {
        validate(s, min, max, z)?;
    }
    if let Some(e) = dim[1] {
        validate(e, min, max, z)?;
    }

    //値を変換して代入する
    let start = match dim[0] {
        Some(v) => v,
        None => min,
    };

    let end = match dim[1] {
        Some(v) => v,
        None => max,
    };

    //順序を正しくする
    if end > start {
        Ok([start, end])
    } else {
        Ok([end, start])
    }
}

/// 座標値が指定範囲内にあるかを検証する汎用関数
fn validate_coordinate_range<T>(
    num: T,
    min: T,
    max: T,
    z: u8,
    error_fn: impl FnOnce(T, u8) -> Error,
) -> Result<(), Error>
where
    T: PartialOrd + Copy,
{
    if (min..=max).contains(&num) {
        Ok(())
    } else {
        Err(error_fn(num, z))
    }
}

/// F座標の範囲が正しいかを確認する
fn valid_range_f(num: i64, min: i64, max: i64, z: u8) -> Result<(), Error> {
    validate_coordinate_range(num, min, max, z, |f, z| Error::FOutOfRange { f, z })
}

/// X座標の範囲が正しいかを確認する
fn valid_range_x(num: u64, min: u64, max: u64, z: u8) -> Result<(), Error> {
    validate_coordinate_range(num, min, max, z, |x, z| Error::XOutOfRange { x, z })
}

/// Y座標の範囲が正しいかを確認する
fn valid_range_y(num: u64, min: u64, max: u64, z: u8) -> Result<(), Error> {
    validate_coordinate_range(num, min, max, z, |y, z| Error::YOutOfRange { y, z })
}
