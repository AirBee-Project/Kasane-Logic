// src/id/spatial_id/range.rs
use std::fmt;

use crate::{
    error::Error,
    geometry::{coordinate::Coordinate, ecef::Ecef},
    spatial_id::{
        SpatialId, SpatialIdEncode,
        constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
        encode::FlexId,
        helpers,
        segment::{Segment, encode::EncodeSegment},
        single::SingleId,
    },
};

/// RangeIdは拡張された空間 ID を表す型です。
///
/// 各インデックスを範囲で指定することができます。各次元の範囲を表す配列の順序には意味を持ちません。内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// pub struct RangeId {
///     z: u8,
///     f: [i32; 2],
///     x: [u32; 2],
///     y: [u32; 2],
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RangeId {
    pub(crate) z: u8,
    pub(crate) f: [i32; 2],
    pub(crate) x: [u32; 2],
    pub(crate) y: [u32; 2],
}

impl fmt::Display for RangeId {
    /// `RangeId` を文字列形式で表示します。
    ///
    /// 形式は `"{z}/{f1}:{f2}/{x1}:{x2}/{y1}:{y2}"` です。
    /// また、次元の範囲が単体の場合は自動的にその次元がSingle表示になります。
    ///
    /// 通常時の範囲表示
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use std::fmt::Write;
    /// let id = RangeId::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// Single範囲に自動圧縮（`f1=f2`）
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use std::fmt::Write;
    /// let id = RangeId::new(4, [-3,-3], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    ///  assert_eq!(s, "4/-3/8:9/5:10");;
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.z,
            format_dimension(self.f),
            format_dimension(self.x),
            format_dimension(self.y),
        )
    }
}

//次元の文字列を圧縮するための関数
fn format_dimension<T: PartialEq + fmt::Display>(dimension: [T; 2]) -> String {
    if dimension[0] == dimension[1] {
        format!("{}", dimension[0])
    } else {
        format!("{}:{}", dimension[0], dimension[1])
    }
}

impl RangeId {
    /// 指定された値から [`RangeId`] を構築します。
    /// 与えられた `z`, `f1`, `f2`, `x1`, `x2`, `y1`, `y2` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    ///　**各次元の与えられた2つの値は自動的に昇順に並び替えられ、**
    /// **常に `[min, max]` の形で内部に保持されます。**
    ///
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–63の範囲が有効）  
    /// * `f1` — 鉛直方向範囲の端のFインデックス
    /// * `f2` — 鉛直方向範囲の端のFインデックス
    /// * `x1` — 東西方向範囲の端のXインデックス
    /// * `x2` — 東西方向範囲の端のXインデックス
    /// * `y1` — 南北方向範囲の端のYインデックス
    /// * `y2` — 南北方向範囲の端のYインデックス
    ///
    /// # バリデーション
    /// - `z` が 63 を超える場合、[`Error::ZOutOfRange`] を返します。  
    /// - `f1`,`f2` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、  
    ///   [`Error::FOutOfRange`] を返します。  
    /// - `x1`,`x2` または `y1`,`y2` が `0..=XY_MAX[z]` の範囲外の場合、  
    ///   それぞれ [`Error::XOutOfRange`]、[`Error::YOutOfRange`] を返します。
    ///
    ///
    /// IDの作成:
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(4, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(Error::FOutOfRange{z:4,f:29}));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(68, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(Error::ZOutOfRange { z:68 }));
    /// ```
    pub fn new(z: u8, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> Result<RangeId, Error> {
        if z as usize > MAX_ZOOM_LEVEL {
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];
        let mut f = f;
        let mut y = y;

        for i in 0..2 {
            if f[i] < f_min || f[i] > f_max {
                return Err(Error::FOutOfRange { f: f[i], z });
            }
            if x[i] > xy_max {
                return Err(Error::XOutOfRange { x: x[i], z });
            }
            if y[i] > xy_max {
                return Err(Error::YOutOfRange { y: y[i], z });
            }
        }

        if f[0] > f[1] {
            f.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }

        Ok(RangeId { z, f, x, y })
    }

    /// この `RangeId` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// ```
    pub fn as_z(&self) -> u8 {
        self.z
    }

    /// この `RangeId` が保持しているズームレベル `[f1,f2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_f(), [-3i32,29i32]);
    /// ```
    pub fn as_f(&self) -> [i32; 2] {
        self.f
    }

    /// この `RangeId` が保持しているズームレベル `[x1,x2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_x(), [8u32,9u32]);
    /// ```
    pub fn as_x(&self) -> [u32; 2] {
        self.x
    }

    /// この `RangeId` が保持しているズームレベル `[y1,y2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_y(), [5u32,10u32]);
    /// ```
    pub fn as_y(&self) -> [u32; 2] {
        self.y
    }

    pub fn set_f(&mut self, value: [i32; 2]) -> Result<(), Error> {
        let z = self.z;
        let mut value = value;
        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];

        for i in 0..2 {
            if value[i] < f_min || value[i] > f_max {
                return Err(Error::FOutOfRange { f: value[i], z });
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.f = value;
        Ok(())
    }

    pub fn set_x(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z;
        let xy_max = XY_MAX[z as usize];

        for i in 0..2 {
            if value[i] > xy_max {
                return Err(Error::XOutOfRange { x: value[i], z });
            }
        }

        self.x = value;
        Ok(())
    }

    pub fn set_y(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z;
        let mut value = value;
        let xy_max = XY_MAX[z as usize];

        for i in 0..2 {
            if value[i] > xy_max {
                return Err(Error::YOutOfRange { y: value[i], z });
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.y = value;
        Ok(())
    }

    /// 指定したズームレベル差 `difference` に基づき、この `RangeId` が表す空間のすべての子 `RangeId` を生成します。
    ///
    /// # パラメータ
    /// * `difference` — 子 ID を計算する際に増加させるズームレベル差（差の値が0–63の範囲の場合に有効）
    ///
    /// # バリデーション
    /// - `self.z + difference` が `63` を超える場合、[`Error::ZOutOfRange`] を返します。
    ///
    /// `difference = 1` による細分化
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.children(1).unwrap();
    /// assert_eq!(result,  RangeId::new(6, [-6, 59], [16, 19], [10, 21] ).unwrap());
    ///
    /// ```
    ///
    /// ズームレベルの範囲外
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.children(63);
    /// assert!(matches!(result, Err(Error::ZOutOfRange { z: 68 })));
    /// ```
    pub fn children(&self, difference: u8) -> Result<RangeId, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;
        if z > 63 {
            return Err(Error::ZOutOfRange { z });
        }

        let scale_f = 2_i32.pow(difference as u32);
        let scale_xy = 2_u32.pow(difference as u32);

        let f = helpers::scale_range_i32(self.f[0], self.f[1], scale_f);
        let x = helpers::scale_range_u32(self.x[0], self.x[1], scale_xy);
        let y = helpers::scale_range_u32(self.y[0], self.y[1], scale_xy);

        Ok(RangeId { z, f, x, y })
    }

    /// 指定したズームレベル差 `difference` に基づき、この `RangeId` を含む最小の大きさの `RangeId` を返します。
    ///
    /// # パラメータ
    /// * `difference` — 親 ID を計算する際に減少させるズームレベル差
    ///
    /// # バリデーション
    /// - `self.z - difference < 0` の場合、親が存在しないため `None` を返します。
    ///
    /// `difference = 1` による上位層への移動
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [1,29], [8,9], [5,10]).unwrap();
    /// let parent = id.parent(1).unwrap();
    ///
    /// assert_eq!(parent.as_z(), 4);
    /// assert_eq!(parent.as_f(), [0,14]);
    /// assert_eq!(parent.as_x(), [4,4]);
    /// assert_eq!(parent.as_y(), [2,5]);
    /// ```
    ///
    /// Fが負の場合の挙動:
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let parent = id.parent(1).unwrap();
    ///
    /// assert_eq!(parent.as_z(), 4);
    /// assert_eq!(parent.as_f(), [-5,-3]);
    /// assert_eq!(parent.as_x(), [4,4]);
    /// assert_eq!(parent.as_y(), [2,5]);
    /// ```
    ///
    /// ズームレベルの範囲外:
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// // difference = 6 の場合は親が存在しないため None
    /// assert!(id.parent(6).is_none());
    /// ```
    pub fn parent(&self, difference: u8) -> Option<RangeId> {
        let z = self.z.checked_sub(difference)?;
        let shift = difference as u32;

        let f = [
            if self.f[0] == -1 {
                -1
            } else {
                self.f[0] >> shift
            },
            if self.f[1] == -1 {
                -1
            } else {
                self.f[1] >> shift
            },
        ];

        let x = [self.x[0] >> shift, self.x[1] >> shift];
        let y = [self.y[0] >> shift, self.y[1] >> shift];

        Some(RangeId { z, f, x, y })
    }

    /// [`RangeId`]を[`SingleId`]に分解し、イテレータとして提供します。
    pub fn to_single(&self) -> impl Iterator<Item = SingleId> + '_ {
        let z = self.z;

        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];

        f_range.flat_map(move |f| {
            let y_range = y_range.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=self.max_xy())
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                y_range
                    .clone()
                    .map(move |y| unsafe { SingleId::new_unchecked(z, f, x, y) })
            })
        })
    }

    /// 検証を行わずに [`RangeId`] を構築します。
    ///
    /// この関数は [`RangeId::new`] と異なり、与えられた `z`, `f1`, `f2`, `x1`,`x2`, `y1, `y2` に対して
    /// 一切の範囲チェックや整合性チェックを行いません。
    /// そのため、高速に ID を生成できますが、**不正なパラメータを与えた場合の動作は未定義です**。
    ///
    /// # 注意
    /// 呼び出し側は、以下をすべて満たすことを保証しなければなりません。
    ///
    /// * `z` が有効なズームレベル（0–63）であること  
    /// * `f1`,`f2` が与えられた `z` に応じて `F_MIN[z]..=F_MAX[z]` の範囲内であること  
    /// * `x1`,`x2` および `y1`,`y2` が `0..=XY_MAX[z]` の範囲内であること  
    ///
    /// これらが保証されない場合、本構造体の他のメソッド（範囲を前提とした計算）が
    /// パニック・不正メモリアクセス・未定義動作を引き起こす可能性があります。
    ///
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { RangeId::new_unchecked(5, [-10,-5], [8,9], [5,10]) };
    ///
    /// assert_eq!(id.as_z(), 5);
    /// assert_eq!(id.as_f(), [-10,-5]);
    /// assert_eq!(id.as_x(), [8,9]);
    /// assert_eq!(id.as_y(), [5,10]);
    /// ```
    pub unsafe fn new_unchecked(z: u8, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> RangeId {
        RangeId { z, f, x, y }
    }
}

impl SpatialId for RangeId {
    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::spatial_id::SpatialId;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.min_f(), -32i32);
    /// ```
    fn min_f(&self) -> i32 {
        F_MIN[self.z as usize]
    }

    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::spatial_id::SpatialId;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.max_f(), 31i32);
    /// ```
    fn max_f(&self) -> i32 {
        F_MAX[self.z as usize]
    }

    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::spatial_id::range::RangeId;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::spatial_id::SpatialId;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.max_xy(), 31u32);
    /// ```
    fn max_xy(&self) -> u32 {
        XY_MAX[self.z as usize]
    }

    fn move_f(&mut self, by: i32) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        let z = self.z;

        let ns = self.f[0]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i32::MAX, z })?;
        let ne = self.f[1]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i32::MAX, z })?;

        if ns < min || ns > max {
            return Err(Error::FOutOfRange { f: ns, z });
        }
        if ne < min || ne > max {
            return Err(Error::FOutOfRange { f: ne, z });
        }

        self.f = [ns, ne];
        Ok(())
    }

    fn move_x(&mut self, by: i32) {
        let new = (self.x[0] as i32 + by).rem_euclid(self.max_xy().try_into().unwrap());
        self.x[0] = new as u32;

        let new = (self.x[1] as i32 + by).rem_euclid(self.max_xy().try_into().unwrap());
        self.x[1] = new as u32;
    }

    fn move_y(&mut self, by: i32) -> Result<(), Error> {
        if by >= 0 {
            let byu = by as u32;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.y[0]
                .checked_add(byu)
                .ok_or(Error::YOutOfRange { y: u32::MAX, z })?;
            let ne = self.y[1]
                .checked_add(byu)
                .ok_or(Error::YOutOfRange { y: u32::MAX, z })?;

            if ns > max {
                return Err(Error::YOutOfRange { y: ns, z });
            }
            if ne > max {
                return Err(Error::YOutOfRange { y: ne, z });
            }

            self.y = [ns, ne];
            Ok(())
        } else {
            // south
            let byu = (-by) as u32;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.y[0]
                .checked_sub(byu)
                .ok_or(Error::YOutOfRange { y: 0, z })?;
            let ne = self.y[1]
                .checked_sub(byu)
                .ok_or(Error::YOutOfRange { y: 0, z })?;

            if ns > max {
                return Err(Error::YOutOfRange { y: ns, z });
            }
            if ne > max {
                return Err(Error::YOutOfRange { y: ne, z });
            }

            self.y = [ns, ne];
            Ok(())
        }
    }

    /// [`RangeId`] の中心座標を[`Coordinate`]型で返します。
    ///
    /// 中心座標は空間IDの最も外側の頂点の8点の平均座標です。現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    fn center(&self) -> Coordinate {
        let z = self.z;

        let xf = (self.x[0] + self.x[1]) as f64 / 2.0 + 0.5;
        let yf = (self.y[0] + self.y[1]) as f64 / 2.0 + 0.5;
        let ff = (self.f[0] + self.f[1]) as f64 / 2.0 + 0.5;

        unsafe {
            Coordinate::new_unchecked(
                helpers::longitude(xf, z),
                helpers::latitude(yf, z),
                helpers::altitude(ff, z),
            )
        }
    }

    /// [`RangeId`] の最も外側の頂点の8点の座標を[`Coordinate`]型の配列として返します。
    ///
    /// 現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    fn vertices(&self) -> [Coordinate; 8] {
        let z = self.z;

        // 2 点ずつの端点
        let xs = [self.x[0] as f64, (self.x[1] + 1) as f64];
        let ys = [self.y[0] as f64, (self.y[1] + 1) as f64];
        let fs = [self.f[0] as f64, (self.f[1] + 1) as f64];

        // 各軸方向の計算は 2 回だけにする
        let longitudes: [f64; 2] = [helpers::longitude(xs[0], z), helpers::longitude(xs[1], z)];

        let latitudes: [f64; 2] = [helpers::latitude(ys[0], z), helpers::latitude(ys[1], z)];

        let altitudes: [f64; 2] = [helpers::altitude(fs[0], z), helpers::altitude(fs[1], z)];

        let mut out = [Coordinate::default(); 8];

        let mut i = 0;
        for fi in 0..2 {
            for yi in 0..2 {
                for xi in 0..2 {
                    let _ = out[i].set_altitude(altitudes[fi]);
                    let _ = out[i].set_latitude(latitudes[yi]);
                    let _ = out[i].set_longitude(longitudes[xi]);
                    i += 1;
                }
            }
        }

        out
    }

    ///その空間IDのＦ方向の長さをメートル単位で計算する関数
    fn length_f(&self) -> f64 {
        //Z=25のとき、ちょうど高さが1mとなる
        let one = 2_i32.pow(25 - self.as_z() as u32) as f64;

        //このRangeIdの高さ方向の幅を計算
        let range = (self.as_f()[0] - self.as_f()[1]).abs() as f64;

        //かけ合わせて答えを返却
        (one * range).into()
    }

    ///その空間IDのX方向の長さをメートル単位で計算する関数
    fn length_x(&self) -> f64 {
        todo!()
    }

    ///その空間IDのY方向の長さをメートル単位で計算する関数
    fn length_y(&self) -> f64 {
        todo!()
    }
}

impl SpatialIdEncode for RangeId {
    ///[FlexId]に変換を行う関数
    fn encode(&self) -> impl Iterator<Item = FlexId> + '_ {
        let f_segments = Segment::<i32>::new(self.as_z(), self.as_f());
        let x_segments: Vec<_> = if self.x[0] <= self.x[1] {
            Segment::<u32>::new(self.z, self.as_x()).collect()
        } else {
            let max = self.max_xy();
            Segment::<u32>::new(self.z, [self.x[0], max])
                .chain(Segment::<u32>::new(self.z, [0, self.x[1]]))
                .collect()
        };
        let y_segments = Segment::<u32>::new(self.z, self.as_y()).collect();

        let x_segments: Vec<_> = x_segments;
        let y_segments: Vec<_> = y_segments;

        f_segments.flat_map(move |f| {
            let x_segments = x_segments.clone();
            let y_segments = y_segments.clone();

            x_segments.into_iter().flat_map(move |x| {
                let y_segments = y_segments.clone();

                y_segments.into_iter().map(move |y| {
                    FlexId::new(
                        EncodeSegment::from(f.clone()),
                        EncodeSegment::from(x.clone()),
                        EncodeSegment::from(y.clone()),
                    )
                })
            })
        })
    }
}

impl From<SingleId> for RangeId {
    ///`SingleId`を[`RangeId`]に変換します。表す物理的な範囲に変化はありません。
    fn from(id: SingleId) -> Self {
        RangeId {
            z: id.as_z(),
            f: [id.as_f(), id.as_f()],
            x: [id.as_x(), id.as_x()],
            y: [id.as_y(), id.as_y()],
        }
    }
}
