// src/id/space_id/range.rs
use itertools::iproduct;
use std::fmt;

use crate::{
    bit_vec::BitVec,
    error::Error,
    geometry::point::coordinate::Coordinate,
    id::space_id::{
        SpaceID,
        constants::{F_MAX, F_MIN, XY_MAX},
        encode::EncodeID,
        helpers,
        segment::Segment,
        single::SingleID,
    },
};

/// RangeIDは拡張された空間 ID を表す型です。各インデックスを範囲で指定することができます。 各次元の範囲には順序が意味を持ち、通常は `[α, β]` の形で `α <= β` を想定します。ただし `α >= β` の場合は区間が反転し、空間の循環方向を表します。例えば Xインデックス ではWEB メルカトル法に基づく経度の循環性のため、これは実空間上の連続性と一致します。
///
/// 一方 Yインデックス は WEB メルカトル法の制約（高緯度の非対応）により、反転区間は必ずしも実空間上の連続性を意味しません。Fインデックス については XYインデックス との対称性を考慮し境界循環を定義していますが、`α >= β` の場合は実空間的な連続性を保証しません。
///
/// 内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、
/// ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// pub struct RangeID {
///     z: u8,
///     f: [i64; 2],
///     x: [u64; 2],
///     y: [u64; 2],
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RangeID {
    pub(crate) z: u8,
    pub(crate) f: [i64; 2],
    pub(crate) x: [u64; 2],
    pub(crate) y: [u64; 2],
}

impl fmt::Display for RangeID {
    /// `RangeID` を文字列形式で表示します。
    ///
    /// 形式は `"{z}/{f1}:{f2}/{x1}:{x2}/{y1}:{y2}"` です。
    /// また、次元の範囲が単体の場合は自動的にその次元がSingle表示になります。
    ///
    /// 通常時の範囲表示
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use std::fmt::Write;
    /// let id = RangeID::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// Single範囲に自動圧縮（`f1=f2`）
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use std::fmt::Write;
    /// let id = RangeID::new(4, [-3,-3], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    ///  assert_eq!(s, "4/-3/8:9/5:10");;
    /// ```
    ///
    /// 反転範囲の表現（Xインデックス）
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use std::fmt::Write;
    /// let id = RangeID::new(4, [-3,-3], [9,1], [5,10]).unwrap();
    /// let s = format!("{}", id);
    ///  assert_eq!(s, "4/-3/9:1/5:10");;
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

impl RangeID {
    /// 指定された値から [`RangeID`] を構築します。このコンストラクタは、与えられた `z`, `f1`, `f2`, `x1`, `x2`, `y1`, `y2` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–63の範囲が有効）  
    /// * `f1` — 鉛直方向範囲の端のFインデックス
    /// * `f2` — 鉛直方向範囲の端のFインデックス
    /// * `x1` — 東西方向範囲の端のXインデックス
    /// * `x2` — 東西方向範囲の端のXインデックス
    /// * `y1` — 南北方向範囲の端のXインデックス
    /// * `y2` — 南北方向範囲の端のXインデックス
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
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(4, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(Error::FOutOfRange{z:4,f:29}));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(68, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(Error::ZOutOfRange { z:68 }));
    /// ```
    pub fn new(z: u8, f: [i64; 2], x: [u64; 2], y: [u64; 2]) -> Result<RangeID, Error> {
        if z > 63u8 {
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        if f[0] < f_min || f[0] > f_max {
            return Err(Error::FOutOfRange { f: f[0], z });
        }
        if f[1] < f_min || f[1] > f_max {
            return Err(Error::FOutOfRange { f: f[1], z });
        }

        if x[0] > xy_max {
            return Err(Error::XOutOfRange { x: x[0], z });
        }
        if x[1] > xy_max {
            return Err(Error::XOutOfRange { x: x[1], z });
        }

        if y[0] > xy_max {
            return Err(Error::YOutOfRange { y: y[0], z });
        }
        if y[1] > xy_max {
            return Err(Error::YOutOfRange { y: y[1], z });
        }

        Ok(RangeID { z, f, x, y })
    }

    pub fn as_z(&self) -> u8 {
        self.z
    }
    pub fn as_f(&self) -> [i64; 2] {
        self.f
    }
    pub fn as_x(&self) -> [u64; 2] {
        self.x
    }
    pub fn as_y(&self) -> [u64; 2] {
        self.y
    }

    pub fn children(&self, difference: u8) -> Result<RangeID, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;
        if z > 63 {
            return Err(Error::ZOutOfRange { z });
        }

        let scale_f = 2_i64.pow(difference as u32);
        let scale_xy = 2_u64.pow(difference as u32);

        let f = helpers::scale_range_i64(self.f[0], self.f[1], scale_f);
        let x = helpers::scale_range_u64(self.x[0], self.x[1], scale_xy);
        let y = helpers::scale_range_u64(self.y[0], self.y[1], scale_xy);

        Ok(RangeID { z, f, x, y })
    }

    pub fn parent(&self, difference: u8) -> Option<RangeID> {
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

        Some(RangeID { z, f, x, y })
    }

    pub fn to_single_id(&self) -> impl Iterator<Item = SingleID> + '_ {
        let f_range = self.f[0]..=self.f[1];
        let x_range = self.x[0]..=self.x[1];
        let y_range = self.y[0]..=self.y[1];

        iproduct!(f_range, x_range, y_range).map(move |(f, x, y)| SingleID { z: self.z, f, x, y })
    }
}
impl SpaceID for RangeID {
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize]
    }

    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize]
    }

    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize]
    }

    fn bound_up(&mut self, by: u64) -> Result<(), Error> {
        self.bound_f(by as i64)
    }

    fn bound_down(&mut self, by: u64) -> Result<(), Error> {
        self.bound_f(-(by as i64))
    }

    fn bound_north(&mut self, by: u64) -> Result<(), Error> {
        self.bound_x(by as i64)
    }

    fn bound_south(&mut self, by: u64) -> Result<(), Error> {
        self.bound_x(-(by as i64))
    }

    fn bound_east(&mut self, by: u64) -> Result<(), Error> {
        self.bound_y(by as i64)
    }

    fn bound_west(&mut self, by: u64) -> Result<(), Error> {
        self.bound_y(-(by as i64))
    }

    fn bound_f(&mut self, by: i64) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        let z = self.z;

        let ns = self.f[0]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i64::MAX, z })?;
        let ne = self.f[1]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i64::MAX, z })?;

        if ns < min || ns > max {
            return Err(Error::FOutOfRange { f: ns, z });
        }
        if ne < min || ne > max {
            return Err(Error::FOutOfRange { f: ne, z });
        }

        self.f = [ns, ne];
        Ok(())
    }

    fn bound_x(&mut self, by: i64) -> Result<(), Error> {
        if by >= 0 {
            let byu = by as u64;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.x[0]
                .checked_add(byu)
                .ok_or(Error::XOutOfRange { x: u64::MAX, z })?;
            let ne = self.x[1]
                .checked_add(byu)
                .ok_or(Error::XOutOfRange { x: u64::MAX, z })?;

            if ns > max {
                return Err(Error::XOutOfRange { x: ns, z });
            }
            if ne > max {
                return Err(Error::XOutOfRange { x: ne, z });
            }

            self.x = [ns, ne];
            Ok(())
        } else {
            let byu = (-by) as u64;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.x[0]
                .checked_sub(byu)
                .ok_or(Error::XOutOfRange { x: 0, z })?;
            let ne = self.x[1]
                .checked_sub(byu)
                .ok_or(Error::XOutOfRange { x: 0, z })?;

            if ns > max {
                return Err(Error::XOutOfRange { x: ns, z });
            }
            if ne > max {
                return Err(Error::XOutOfRange { x: ne, z });
            }

            self.x = [ns, ne];
            Ok(())
        }
    }

    fn bound_y(&mut self, by: i64) -> Result<(), Error> {
        if by >= 0 {
            let byu = by as u64;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.y[0]
                .checked_add(byu)
                .ok_or(Error::YOutOfRange { y: u64::MAX, z })?;
            let ne = self.y[1]
                .checked_add(byu)
                .ok_or(Error::YOutOfRange { y: u64::MAX, z })?;

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
            let byu = (-by) as u64;
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

    fn wrap_up(&mut self, by: u64) {
        self.wrap_f(by as i64);
    }

    fn wrap_down(&mut self, by: u64) {
        self.wrap_f(-(by as i64));
    }

    fn wrap_north(&mut self, by: u64) {
        self.wrap_y(by as i64);
    }

    fn wrap_south(&mut self, by: u64) {
        self.wrap_y(-(by as i64));
    }

    fn wrap_east(&mut self, by: u64) {
        self.wrap_x(by as i64);
    }

    fn wrap_west(&mut self, by: u64) {
        self.wrap_x(-(by as i64));
    }

    fn wrap_f(&mut self, by: i64) {
        let min = self.min_f();
        let max = self.max_f();
        let width = (max - min + 1) as i128;

        for v in &mut self.f {
            let offset = (*v - min) as i128;
            let new = ((offset + (by as i128)) % width + width) % width;
            *v = (min as i128 + new) as i64;
        }
    }

    fn wrap_x(&mut self, by: i64) {
        let max = self.max_xy();
        let ring = max + 1;

        let shift = if by >= 0 {
            (by as u64) % ring
        } else {
            (ring - ((-by as u64) % ring)) % ring
        };

        for v in &mut self.x {
            *v = (*v + shift) % ring;
        }
    }

    fn wrap_y(&mut self, by: i64) {
        let max = self.max_xy();
        let ring = max + 1;

        let shift = if by >= 0 {
            (by as u64) % ring
        } else {
            (ring - ((-by as u64) % ring)) % ring
        };

        for v in &mut self.y {
            *v = (*v + shift) % ring;
        }
    }

    fn center(&self) -> Coordinate {
        let z = self.z;

        let xf = (self.x[0] + self.x[1]) as f64 / 2.0 + 0.5;
        let yf = (self.y[0] + self.y[1]) as f64 / 2.0 + 0.5;
        let ff = (self.f[0] + self.f[1]) as f64 / 2.0 + 0.5;

        Coordinate {
            longitude: helpers::longitude(xf, z),
            latitude: helpers::latitude(yf, z),
            altitude: helpers::altitude(ff, z),
        }
    }

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

        let mut out = [Coordinate {
            longitude: 0.0,
            latitude: 0.0,
            altitude: 0.0,
        }; 8];

        for (i, (fi, yi, xi)) in iproduct!(0..2, 0..2, 0..2).enumerate() {
            out[i] = Coordinate {
                longitude: longitudes[xi],
                latitude: latitudes[yi],
                altitude: altitudes[fi],
            };
        }

        out
    }
}

impl From<RangeID> for EncodeID {
    fn from(id: RangeID) -> Self {
        let f_segment = Segment::<i64>::new(id.z, id.f);
        let x_segment = Segment::<u64>::new(id.z, id.x);
        let y_segment = Segment::<u64>::new(id.z, id.y);

        let f_bitvec: Vec<BitVec> = f_segment.iter().map(|f| (*f).into()).collect();
        let x_bitvec: Vec<BitVec> = x_segment.iter().map(|x| (*x).into()).collect();
        let y_bitvec: Vec<BitVec> = y_segment.iter().map(|y| (*y).into()).collect();

        EncodeID {
            f: f_bitvec,
            x: x_bitvec,
            y: y_bitvec,
        }
    }
}
