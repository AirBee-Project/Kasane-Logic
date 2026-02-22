// src/id/spatial_id/range.rs
use std::fmt;

#[cfg(any(test))]
use proptest::prelude::*;

use crate::{
    Coordinate, SingleId,
    error::Error,
    spatial_id::{
        FlexIds, HyperRect, HyperRectSegments, SpatialId,
        constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
        flex_id::FlexId,
        helpers,
        segment::Segment,
    },
};
#[cfg(any(test, feature = "random"))]
use rand::Rng;
#[cfg(any(test, feature = "random"))]
use std::ops::RangeInclusive;

/// RangeIdは空間IDの範囲表現を表す型です。
///
/// 各インデックスを範囲で指定することができます。各次元の範囲を表す配列の順序には意味を持ちません。
/// 内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、
/// ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap`
/// などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// pub struct RangeId {
///     z: u8,
///     f: [i64; 2],
///     x: [u64; 2],
///     y: [u64; 2],
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RangeId {
    z: u8,
    f: [i64; 2],
    x: [u64; 2],
    y: [u64; 2],
}

impl fmt::Display for RangeId {
    /// `RangeId` を文字列形式で表示します。
    ///
    /// 形式は `"{z}/{f1}:{f2}/{x1}:{x2}/{y1}:{y2}"` です。
    /// また、次元の範囲が単体の場合は自動的にその次元がSingle表示になります。
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

fn format_dimension<T: PartialEq + fmt::Display>(dimension: [T; 2]) -> String {
    if dimension[0] == dimension[1] {
        format!("{}", dimension[0])
    } else {
        format!("{}:{}", dimension[0], dimension[1])
    }
}

impl RangeId {
    /// 指定された値から [`RangeId`] を構築します。
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–64の範囲が有効）  
    /// * `f` — 鉛直方向範囲の端点 [f1, f2]
    /// * `x` — 東西方向範囲の端点 [x1, x2]
    /// * `y` — 南北方向範囲の端点 [y1, y2]
    ///
    /// # バリデーション
    /// - `z` が 64 を超える場合、[`Error::ZOutOfRange`] を返します。  
    /// - 各インデックスがズームレベル `z` に対する許容範囲外の場合、対応するエラーを返します。  
    /// - `f` および `y` の端点は自動的に昇順 `[min, max]` に並び替えられます。
    /// - **X次元は循環を許容するため、`x1 > x2` の場合は日付変更線をまたぐ範囲として扱われ、並び替えは行われません。**
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, [-3, 6], [14, 1], [5, 10]).unwrap();
    /// assert_eq!(id.to_string(), "4/-3:6/14:1/5:10");
    /// ```
    pub fn new(z: u8, f: [i64; 2], x: [u64; 2], y: [u64; 2]) -> Result<RangeId, Error> {
        if z as usize > MAX_ZOOM_LEVEL {
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize] as i64;
        let f_max = F_MAX[z as usize] as i64;
        let xy_max = XY_MAX[z as usize] as u64;
        let mut f = f;
        let x = x;
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
    pub fn z(&self) -> u8 {
        self.z
    }

    /// この `RangeId` が保持している F 範囲 `[f1, f2]` を返します。
    pub fn f(&self) -> [i64; 2] {
        self.f
    }

    /// この `RangeId` が保持している X 範囲 `[x1, x2]` を返します。
    pub fn x(&self) -> [u64; 2] {
        self.x
    }

    /// この `RangeId` が保持している Y 範囲 `[y1, y2]` を返します。
    pub fn y(&self) -> [u64; 2] {
        self.y
    }

    /// F 範囲を更新します。
    /// 新しい端点は自動的に昇順に並び替えられます。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// let mut id = RangeId::new(4, [0, 0], [0, 0], [0, 0]).unwrap();
    /// id.set_f([5, -2]).unwrap();
    /// assert_eq!(id.f(), [-2, 5]);
    /// ```
    pub fn set_f(&mut self, value: [i64; 2]) -> Result<(), Error> {
        let f_min = self.min_f();
        let f_max = self.max_f();
        for &v in &value {
            if v < f_min || v > f_max {
                return Err(Error::FOutOfRange { f: v, z: self.z });
            }
        }
        let mut value = value;
        if value[0] > value[1] {
            value.swap(0, 1);
        }
        self.f = value;
        Ok(())
    }

    /// X 範囲を更新します。
    /// X次元は日付変更線のまたぎ（循環）を許容するため、値の並び替えは行われません。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// let mut id = RangeId::new(4, [0, 0], [0, 0], [0, 0]).unwrap();
    /// id.set_x([15, 2]).unwrap();
    /// assert_eq!(id.x(), [15, 2]); // そのまま保持される
    /// ```
    pub fn set_x(&mut self, value: [u64; 2]) -> Result<(), Error> {
        let max = self.max_xy();
        for &v in &value {
            if v > max {
                return Err(Error::XOutOfRange { x: v, z: self.z });
            }
        }
        self.x = value;
        Ok(())
    }

    /// Y 範囲を更新します。
    /// 新しい端点は自動的に昇順に並び替えられます。
    pub fn set_y(&mut self, value: [u64; 2]) -> Result<(), Error> {
        let max = self.max_xy();
        for &v in &value {
            if v > max {
                return Err(Error::YOutOfRange { y: v, z: self.z });
            }
        }
        let mut value = value;
        if value[0] > value[1] {
            value.swap(0, 1);
        }
        self.y = value;
        Ok(())
    }

    /// 指定したズームレベル差 `difference` に基づき、この `RangeId` が表す空間のすべての子 `RangeId` を生成します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(5, [-1, 1], [2, 3], [4, 5]).unwrap();
    /// let child = id.children(1).unwrap();
    /// assert_eq!(child.z(), 6);
    /// assert_eq!(child.f(), [-2, 3]);
    /// ```
    pub fn children(&self, difference: u8) -> Result<RangeId, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;
        if z as usize > MAX_ZOOM_LEVEL {
            return Err(Error::ZOutOfRange { z });
        }

        let (f, x, y) = if difference >= 64 {
            ([i64::MIN, i64::MAX], [0, u64::MAX], [0, u64::MAX])
        } else {
            let scale_u = 1_u64 << difference;
            let scale_i = scale_u as i64;
            let f = [
                self.f[0].saturating_mul(scale_i),
                self.f[1]
                    .saturating_mul(scale_i)
                    .saturating_add(scale_i - 1),
            ];
            let x = [
                self.x[0].saturating_mul(scale_u),
                self.x[1]
                    .saturating_mul(scale_u)
                    .saturating_add(scale_u - 1),
            ];
            let y = [
                self.y[0].saturating_mul(scale_u),
                self.y[1]
                    .saturating_mul(scale_u)
                    .saturating_add(scale_u - 1),
            ];
            (f, x, y)
        };
        Ok(RangeId { z, f, x, y })
    }

    /// 指定したズームレベル差 `difference` に基づき、この `RangeId` を含む最小の大きさの `RangeId` を返します。
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
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, [0, 0], [15, 0], [0, 0]).unwrap(); // X軸循環
    /// let count = id.single_ids().count();
    /// assert_eq!(count, 2); // 15 と 0 の2つ
    /// ```
    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        let z = self.z;
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];

        f_range.flat_map(move |f| {
            let y_range = y_range.clone();
            let x_iter: Vec<u64> = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect()
            } else {
                (self.x[0]..=self.max_xy()).chain(0..=self.x[1]).collect()
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
    /// # Safety
    /// 呼び出し側は、`z` および各インデックスが対応するズームレベルの範囲内であることを保証しなければなりません。
    pub unsafe fn new_unchecked(z: u8, f: [i64; 2], x: [u64; 2], y: [u64; 2]) -> RangeId {
        RangeId { z, f, x, y }
    }

    /// 全空間のズームレベル範囲からランダムに [`RangeId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random() -> Self {
        Self::random_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    /// 特定のズームレベル `z` でランダムな [`RangeId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_at(z: u8) -> Self {
        Self::random_within(z..=z)
    }

    /// 指定されたズームレベル範囲内でランダムな [`RangeId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_within(z_range: RangeInclusive<u8>) -> Self {
        let mut rng = rand::rng();
        Self::random_within_using(&mut rng, z_range)
    }

    /// 外部の乱数生成器を使用してランダムな [`RangeId`] を生成します。
    #[cfg(any(test, feature = "random"))]
    pub fn random_within_using<R: Rng>(rng: &mut R, z_range: RangeInclusive<u8>) -> Self {
        let start = *z_range.start();
        let end = (*z_range.end()).min(MAX_ZOOM_LEVEL as u8);
        let z = if start > end {
            end
        } else {
            rng.random_range(start..=end)
        };
        let z_idx = z as usize;

        let f1 = rng.random_range(F_MIN[z_idx] as i64..=F_MAX[z_idx] as i64);
        let f2 = rng.random_range(F_MIN[z_idx] as i64..=F_MAX[z_idx] as i64);
        let x1 = rng.random_range(0..=XY_MAX[z_idx] as u64);
        let x2 = rng.random_range(0..=XY_MAX[z_idx] as u64);
        let y1 = rng.random_range(0..=XY_MAX[z_idx] as u64);
        let y2 = rng.random_range(0..=XY_MAX[z_idx] as u64);

        RangeId::new(z, [f1, f2], [x1, x2], [y1, y2]).expect("Invalid random RangeId")
    }

    #[cfg(any(test))]
    pub fn arb() -> impl Strategy<Value = Self> {
        Self::arb_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    #[cfg(any(test))]
    pub fn arb_within(z_range: RangeInclusive<u8>) -> impl Strategy<Value = Self> {
        z_range.prop_flat_map(|z| {
            let z_idx = z as usize;
            let f_strat = (
                F_MIN[z_idx] as i64..=F_MAX[z_idx] as i64,
                F_MIN[z_idx] as i64..=F_MAX[z_idx] as i64,
            );
            let x_strat = (0..=XY_MAX[z_idx] as u64, 0..=XY_MAX[z_idx] as u64);
            let y_strat = (0..=XY_MAX[z_idx] as u64, 0..=XY_MAX[z_idx] as u64);
            (Just(z), f_strat, x_strat, y_strat).prop_map(
                move |(z, (f1, f2), (x1, x2), (y1, y2))| {
                    RangeId::new(z, [f1, f2], [x1, x2], [y1, y2]).unwrap()
                },
            )
        })
    }
}

impl SpatialId for RangeId {
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize] as i64
    }
    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize] as i64
    }
    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize] as u64
    }

    /// 指定したインデックス差 `by` に基づき、この `RangeId` を垂直上下方向に動かします。
    fn move_f(&mut self, by: i64) -> Result<(), Error> {
        let (min, max, z) = (self.min_f(), self.max_f(), self.z);
        let ns = self.f[0]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i64::MAX, z })?;
        let ne = self.f[1]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i64::MAX, z })?;
        if ns < min || ns > max || ne < min || ne > max {
            return Err(Error::FOutOfRange { f: ns, z });
        }
        self.f = [ns, ne];
        Ok(())
    }

    /// 指定したインデックス差 `by` に基づき、この `RangeId` を東西方向に動かします。
    /// WEBメルカトル図法において、東西方向は循環しているためラップアラウンド計算が行われます。
    ///
    /// ```
    /// # use kasane_logic::{RangeId, SpatialId};
    /// let mut id = RangeId::new(4, [0, 0], [15, 15], [0, 0]).unwrap();
    /// id.move_x(1);
    /// assert_eq!(id.x(), [0, 0]); // 15+1=16 -> 0 (z=4の時)
    /// ```
    fn move_x(&mut self, by: i64) {
        let wrap_limit = self.max_xy() as i64 + 1;
        self.x[0] = (self.x[0] as i64 + by).rem_euclid(wrap_limit) as u64;
        self.x[1] = (self.x[1] as i64 + by).rem_euclid(wrap_limit) as u64;
    }

    /// 指定したインデックス差 `by` に基づき、この `RangeId` を南北方向に動かします。
    fn move_y(&mut self, by: i64) -> Result<(), Error> {
        let (max, z) = (self.max_xy(), self.z);
        let move_logic = |val: u64| {
            if by >= 0 {
                val.checked_add(by as u64)
                    .ok_or(Error::YOutOfRange { y: u64::MAX, z })
            } else {
                val.checked_sub(-by as u64)
                    .ok_or(Error::YOutOfRange { y: 0, z })
            }
        };
        let ns = move_logic(self.y[0])?;
        let ne = move_logic(self.y[1])?;
        if ns > max || ne > max {
            return Err(Error::YOutOfRange { y: ns, z });
        }
        self.y = [ns, ne];
        Ok(())
    }

    /// [`RangeId`] の中心座標を返します。
    /// X次元の循環（x1 > x2）を検出し、最短距離での幾何学的な中心を正しく算出します。
    fn center(&self) -> Coordinate {
        let z = self.z;
        let max_x = self.max_xy() as f64;
        let x0 = self.x[0] as f64;
        let mut x1 = self.x[1] as f64;
        if x0 > x1 {
            x1 += max_x + 1.0;
        }
        let mut xf = (x0 + x1) / 2.0 + 0.5;
        if xf > max_x + 1.0 {
            xf -= max_x + 1.0;
        }

        let yf = (self.y[0] + self.y[1]) as f64 / 2.0 + 0.5;
        let ff = (self.f[0] + self.f[1]) as f64 / 2.0 + 0.5;
        unsafe {
            Coordinate::new_unchecked(
                helpers::latitude(yf, z),
                helpers::longitude(xf, z),
                helpers::altitude(ff, z),
            )
        }
    }

    /// [`RangeId`] の最も外側の頂点8点の座標を返します。
    fn vertices(&self) -> [Coordinate; 8] {
        let z = self.z;
        let max_x = self.max_xy() as f64;
        let x0 = self.x[0] as f64;
        let mut x1 = self.x[1] as f64 + 1.0;
        if x0 > x1 {
            x1 += max_x + 1.0;
        }

        let xs = [x0, x1];
        let ys = [self.y[0] as f64, (self.y[1] + 1) as f64];
        let fs = [self.f[0] as f64, (self.f[1] + 1) as f64];

        let lons = [helpers::longitude(xs[0], z), helpers::longitude(xs[1], z)];
        let lats = [helpers::latitude(ys[0], z), helpers::latitude(ys[1], z)];
        let alts = [helpers::altitude(fs[0], z), helpers::altitude(fs[1], z)];

        let mut out = [Coordinate::default(); 8];
        let mut i = 0;
        for fi in 0..2 {
            for yi in 0..2 {
                for xi in 0..2 {
                    let _ = out[i].set_altitude(alts[fi]);
                    let _ = out[i].set_latitude(lats[yi]);
                    let _ = out[i].set_longitude(lons[xi]);
                    i += 1;
                }
            }
        }
        out
    }

    /// その [`RangeId`] のF（鉛直）方向の総延長をメートル単位で返します。
    fn length_f(&self) -> f64 {
        let one = 2.0_f64.powi(25 - self.z() as i32);
        let range = (self.f[1] - self.f[0]).abs() as f64 + 1.0;
        one * range
    }

    /// その [`RangeId`] のX（東西）方向の長さをメートル単位で算出します。
    fn length_x(&self) -> f64 {
        todo!()
    }

    /// その [`RangeId`] のY（南北）方向の長さをメートル単位で算出します。
    fn length_y(&self) -> f64 {
        todo!()
    }
}

impl HyperRect for RangeId {
    fn segmentation(&self) -> HyperRectSegments {
        let f = Segment::split_f(self.z(), self.f()).collect();
        let x = if self.x[0] <= self.x[1] {
            Segment::split_xy(self.z(), self.x()).collect()
        } else {
            Segment::split_xy(self.z(), [self.x[0], self.max_xy()])
                .chain(Segment::split_xy(self.z(), [0, self.x[1]]))
                .collect()
        };
        let y = Segment::split_xy(self.z(), self.y()).collect();
        HyperRectSegments { f, x, y }
    }
}

impl From<SingleId> for RangeId {
    fn from(id: SingleId) -> Self {
        RangeId {
            z: id.z(),
            f: [id.f(), id.f()],
            x: [id.x(), id.x()],
            y: [id.y(), id.y()],
        }
    }
}
