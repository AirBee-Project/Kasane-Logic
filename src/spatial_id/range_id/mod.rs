pub mod impls;
pub mod random;

#[cfg(any(test))]
use proptest::prelude::*;

use crate::{
    error::Error,
    spatial_id::{
        SpatialId,
        constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
        helpers,
        temporal_id::TemporalId,
    },
};

/// RangeIdは空間IDの範囲表現を表す型です。
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
    z: u8,
    f: [i32; 2],
    x: [u32; 2],
    y: [u32; 2],
    temporal_id: TemporalId,
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
    /// # use kasane_logic::RangeId;
    /// let id = RangeId::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(4, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(Error::FOutOfRange{z:4,f:29}));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(68, [-3,29], [8,9], [5,10]);
    /// assert_eq!(id, Err(Error::ZOutOfRange { z:68 }));
    /// ```
    pub fn new(z: u8, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> Result<RangeId, Error> {
        Self::new_with_temporal(z, f, x, y, TemporalId::whole())
    }

    pub fn new_with_temporal(
        z: u8,
        f: [i32; 2],
        x: [u32; 2],
        y: [u32; 2],

        temporal_id: TemporalId,
    ) -> Result<RangeId, Error> {
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

        Ok(RangeId {
            z,
            f,
            x,
            y,
            temporal_id,
        })
    }

    /// この `RangeId` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// ```
    pub fn z(&self) -> u8 {
        self.z
    }

    /// この `RangeId` が保持しているズームレベル `[f1,f2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.f(), [-3i32,29i32]);
    /// ```
    pub fn f(&self) -> [i32; 2] {
        self.f
    }

    /// この `RangeId` が保持しているズームレベル `[x1,x2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.x(), [8u32,9u32]);
    /// ```
    pub fn x(&self) -> [u32; 2] {
        self.x
    }

    /// この `RangeId` が保持しているズームレベル `[y1,y2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.y(), [5u32,10u32]);
    /// ```
    pub fn y(&self) -> [u32; 2] {
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
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children(1).unwrap();
    /// assert_eq!(result,  RangeId::new(6, [-6, 59], [16, 19], [10, 21] ).unwrap());
    ///
    /// ```
    ///
    /// ズームレベルの範囲外
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children(63);
    /// assert!(matches!(result, Err(Error::ZOutOfRange { z: 68 })));
    /// ```
    pub fn spatial_children(&self, difference: u8) -> Result<RangeId, Error> {
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

        Ok(RangeId {
            z,
            f,
            x,
            y,
            temporal_id: self.temporal().clone(),
        })
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
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [1,29], [8,9], [5,10]).unwrap();
    /// let parent = id.spatial_parent(1).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [0,14]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// Fが負の場合の挙動:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let parent = id.spatial_parent(1).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [-5,-3]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// ズームレベルの範囲外:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// // difference = 6 の場合は親が存在しないため None
    /// assert!(id.spatial_parent(6).is_none());
    /// ```
    pub fn spatial_parent(&self, difference: u8) -> Option<RangeId> {
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

        Some(RangeId {
            z,
            f,
            x,
            y,
            temporal_id: self.temporal().clone(),
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
    /// # use kasane_logic::RangeId;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { RangeId::new_unchecked(5, [-10,-5], [8,9], [5,10]) };
    ///
    /// assert_eq!(id.z(), 5);
    /// assert_eq!(id.f(), [-10,-5]);
    /// assert_eq!(id.x(), [8,9]);
    /// assert_eq!(id.y(), [5,10]);
    /// ```
    pub unsafe fn new_unchecked(z: u8, f: [i32; 2], x: [u32; 2], y: [u32; 2]) -> RangeId {
        unsafe { Self::new_with_temporal_unchecked(z, f, x, y, TemporalId::whole()) }
    }

    pub unsafe fn new_with_temporal_unchecked(
        z: u8,
        f: [i32; 2],
        x: [u32; 2],
        y: [u32; 2],
        temporal_id: TemporalId,
    ) -> RangeId {
        RangeId {
            z,
            f,
            x,
            y,
            temporal_id,
        }
    }
}
