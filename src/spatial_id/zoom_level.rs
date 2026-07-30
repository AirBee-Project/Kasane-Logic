use crate::{SpatialIdError, error::Error};
use core::fmt;

/// ズームレベルを表す型の土台。`LIMIT`は許容する最大ズームレベルで、空間軸（[`ZoomLevel`]、
/// `LIMIT=30`）と時間軸の生セル（[`TZoomLevel`]、
/// `LIMIT=62`）が生成・深化・範囲チェックの式を共有しつつ、`LIMIT`の値でRustの型として区別される
/// （`ZoomLevel`と`TZoomLevel`は異なる型なので取り違えられない）。
///
/// `Zoom`自体は非公開の実装詳細で、公開APIとしては`ZoomLevel`/`TZoomLevel`という具体化された
/// 別名だけを使う（`type ZoomLevel = Zoom<30>;`のように完全に確定した別名にすることで、
/// 呼び出し側で`LIMIT`の型推論が必要にならないようにしている）。
///
/// Fの符号付き範囲（[`ZoomLevel::f_min`]/[`ZoomLevel::f_max`]/[`ZoomLevel::check_f`]）は空間軸専用、
/// 秒数変換（`cell_seconds`、[`TZoomLevel`]側の拡張）は
/// 時間軸専用の拡張として、それぞれの具象化にだけ生やす。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Zoom<const LIMIT: u8>(u8);

/// ズームレベルを表す型。
/// ```
/// # use kasane_logic::{SpatialIdError, ZoomLevel};
/// let z = ZoomLevel::new(5).unwrap();
/// assert_eq!(z.get(), 5);
/// assert_eq!(ZoomLevel::new(255), Err(SpatialIdError::ZOutOfRange { z: 255 }.into()));
/// ```
pub type ZoomLevel = Zoom<30>;

/// 時間軸のズームレベルを表す型。
///
/// [`ZoomLevel`]（空間軸、`LIMIT = 30`）と同じ土台を共有する `Zoom<35>` の別名。
/// 1セルが `2^(35 - z)` 秒（最大 `2^35` 秒＝全期間、最小1秒）を表す。
///
/// `LIMIT` は [`Interval::WHOLE_POW`](crate::Interval::WHOLE_POW) と一致していなければ
/// ならない（最深ズームのセル幅が1秒＝全時間を `2^LIMIT` 秒とする前提）。下の
/// `const` アサーションで食い違いをコンパイル時に落とす。
///
/// ```
/// # use kasane_logic::spatial_id::zoom_level::TZoomLevel;
/// let z = TZoomLevel::new(5).unwrap();
/// assert_eq!(z.get(), 5);
/// assert_eq!(TZoomLevel::MAX.cell_seconds(), 1);
/// ```
pub type TZoomLevel = Zoom<35>;

const _: () = assert!(
    TZoomLevel::MAX.get() == crate::Interval::WHOLE_POW,
    "TZoomLevel の LIMIT と Interval::WHOLE_POW は一致していなければならない"
);

impl<const LIMIT: u8> fmt::Display for Zoom<LIMIT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<const LIMIT: u8> Zoom<LIMIT> {
    /// 最小のズームレベル（`0`）。
    pub const MIN: Self = Zoom(0);

    /// 最大のズームレベル。
    pub const MAX: Self = Zoom(LIMIT);

    /// `z` が `0..=`[`MAX`](Self::MAX) の範囲内であることを検証して生成する。
    ///
    /// # バリデーション
    /// - `z` が [`MAX`](Self::MAX) を超える場合は [`SpatialIdError::ZOutOfRange`] を返す。
    pub const fn new(z: u8) -> Result<Self, Error> {
        if z > LIMIT {
            return Err(Error::SpatialId(SpatialIdError::ZOutOfRange { z }));
        }
        Ok(Zoom(z))
    }

    /// 検証を行わずに生成する。
    ///
    /// # Safety
    /// 呼び出し側は `z <= `[`MAX`](Self::MAX) を保証しなければならない。これを破ると、
    /// 各種インデックス計算でパニックや未定義動作を引き起こす可能性がある。
    pub const unsafe fn new_unchecked(z: u8) -> Self {
        Zoom(z)
    }

    /// 1段深いズームレベルを返す。[`MAX`](Self::MAX) なら [`None`]。
    ///
    /// 自身が有効なので `+1` が [`MAX`](Self::MAX) 以下であることをここで確かめきれる。
    /// 呼び出し側は改めて範囲検証をする必要がない。
    pub const fn deeper(self) -> Option<Self> {
        if self.0 >= LIMIT {
            None
        } else {
            Some(Zoom(self.0 + 1))
        }
    }

    /// 保持しているズームレベルを `u8` として返す。
    pub const fn get(self) -> u8 {
        self.0
    }

    /// このズームレベルにおけるインデックスの最大値（`2^z - 1`）。
    /// 空間軸のX/Yと時間軸の生セルとで共有する式。
    const fn max_index_u64(self) -> u64 {
        (1u64 << self.0) - 1
    }
}

impl<const LIMIT: u8> TryFrom<u8> for Zoom<LIMIT> {
    type Error = Error;

    fn try_from(z: u8) -> Result<Self, Error> {
        Zoom::new(z)
    }
}

impl<const LIMIT: u8> From<Zoom<LIMIT>> for u8 {
    fn from(z: Zoom<LIMIT>) -> u8 {
        z.0
    }
}

// ---- 空間軸専用（`ZoomLevel` = `Zoom<30>`） ----
impl Zoom<30> {
    /// このズームレベルにおける X / Y インデックスの最大値（`unsafe { ZoomLevel::new_unchecked(z as u8) }.xy_max()`）。
    pub const fn xy_max(self) -> u32 {
        self.max_index_u64() as u32
    }

    /// このズームレベルにおける F インデックスの最小値（`unsafe { ZoomLevel::new_unchecked(z as u8) }.f_min()`）。
    pub const fn f_min(self) -> i32 {
        -(1i64 << self.0) as i32
    }

    /// このズームレベルにおける F インデックスの最大値（`unsafe { ZoomLevel::new_unchecked(z as u8) }.f_max()`）。
    pub const fn f_max(self) -> i32 {
        self.max_index_u64() as i32
    }

    /// `f` がこのズームレベルの F 範囲に収まるか検証する。
    ///
    /// # バリデーション
    /// - 範囲外の場合は [`SpatialIdError::FOutOfRange`] を返す。
    pub const fn check_f(self, f: i32) -> Result<(), Error> {
        if f < self.f_min() || f > self.f_max() {
            return Err(Error::SpatialId(SpatialIdError::FOutOfRange {
                z: self.0,
                f,
            }));
        }
        Ok(())
    }

    /// `x` がこのズームレベルの X 範囲に収まるか検証する。
    ///
    /// # バリデーション
    /// - 範囲外の場合は [`SpatialIdError::XOutOfRange`] を返す。
    pub const fn check_x(self, x: u32) -> Result<(), Error> {
        if x > self.xy_max() {
            return Err(Error::SpatialId(SpatialIdError::XOutOfRange {
                z: self.0,
                x,
            }));
        }
        Ok(())
    }

    /// `y` がこのズームレベルの Y 範囲に収まるか検証する。
    ///
    /// # バリデーション
    /// - 範囲外の場合は [`SpatialIdError::YOutOfRange`] を返す。
    pub const fn check_y(self, y: u32) -> Result<(), Error> {
        if y > self.xy_max() {
            return Err(Error::SpatialId(SpatialIdError::YOutOfRange {
                z: self.0,
                y,
            }));
        }
        Ok(())
    }
}

// ---- 時間軸（生セル）専用（`TZoomLevel` = `Zoom<35>`） ----
impl Zoom<35> {
    /// このズームレベルにおけるインデックスの最大値（`2^z - 1`）。
    pub const fn max_index(self) -> u64 {
        self.max_index_u64()
    }

    /// このズームレベルにおける1セルの秒数（`2^(MAX - z)`）。
    pub const fn cell_seconds(self) -> u64 {
        1u64 << (Self::MAX.get() - self.0)
    }

    /// `index` がこのズームレベルの範囲に収まるか検証する。
    pub const fn check_index(self, index: u64) -> Result<(), Error> {
        if index > self.max_index() {
            return Err(Error::SpatialId(SpatialIdError::TOutOfRange {
                i: self.cell_seconds(),
                t: index,
            }));
        }
        Ok(())
    }
}
