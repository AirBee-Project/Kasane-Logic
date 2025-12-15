// src/id/space_id/range.rs
use itertools::iproduct;
use std::{collections::btree_map::Range, fmt};

use crate::{
    bit_vec::BitVec,
    error::Error,
    geometry::coordinate::Coordinate,
    id::space_id::{
        SpaceID,
        constants::{F_MAX, F_MIN, XY_MAX},
        encode::EncodeID,
        helpers,
        segment::Segment,
        single::SingleID,
    },
};

/// RangeIDは拡張された空間 ID を表す型です。
///
/// 各インデックスを範囲で指定することができます。各次元の範囲を表す配列の順序には意味を持ちません。内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
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
    /// 指定された値から [`RangeID`] を構築します。
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
    pub fn new(z: u8, mut f: [i64; 2], mut x: [u64; 2], mut y: [u64; 2]) -> Result<RangeID, Error> {
        if z > 63 {
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

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
        if x[0] > x[1] {
            x.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }

        Ok(RangeID { z, f, x, y })
    }

    /// この `RangeID` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// ```
    pub fn as_z(&self) -> u8 {
        self.z
    }

    /// この `RangeID` が保持しているズームレベル `[f1,f2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_f(), [-3i64,29i64]);
    /// ```
    pub fn as_f(&self) -> [i64; 2] {
        self.f
    }

    /// この `RangeID` が保持しているズームレベル `[x1,x2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_x(), [8u64,9u64]);
    /// ```
    pub fn as_x(&self) -> [u64; 2] {
        self.x
    }

    /// この `RangeID` が保持しているズームレベル `[y1,y2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_y(), [5u64,10u64]);
    /// ```
    pub fn as_y(&self) -> [u64; 2] {
        self.y
    }

    pub fn set_f(&mut self, value: [i64; 2]) -> Result<(), Error> {
        todo!()
    }

    pub fn set_x(&mut self, value: [i64; 2]) -> Result<(), Error> {
        todo!()
    }

    pub fn set_y(&mut self, value: [i64; 2]) -> Result<(), Error> {
        todo!()
    }

    /// 指定したズームレベル差 `difference` に基づき、この `RangeID` が表す空間のすべての子 `RangeID` を生成します。
    ///
    /// # パラメータ
    /// * `difference` — 子 ID を計算する際に増加させるズームレベル差（差の値が0–63の範囲の場合に有効）
    ///
    /// # バリデーション
    /// - `self.z + difference` が `63` を超える場合、[`Error::ZOutOfRange`] を返します。
    ///
    /// `difference = 1` による細分化
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.children(1).unwrap();
    /// assert_eq!(result,  RangeID::new(6, [-6, 59], [16, 19], [10, 21] ).unwrap());
    ///
    /// ```
    ///
    /// ズームレベルの範囲外
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.children(63);
    /// assert!(matches!(result, Err(Error::ZOutOfRange { z: 68 })));
    /// ```
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

    /// 指定したズームレベル差 `difference` に基づき、この `RangeID` を含む最小の大きさの `RangeID` を返します。
    ///
    /// # パラメータ
    /// * `difference` — 親 ID を計算する際に減少させるズームレベル差
    ///
    /// # バリデーション
    /// - `self.z - difference < 0` の場合、親が存在しないため `None` を返します。
    ///
    /// `difference = 1` による上位層への移動
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [1,29], [8,9], [5,10]).unwrap();
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
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
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
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// // difference = 6 の場合は親が存在しないため None
    /// assert!(id.parent(6).is_none());
    /// ```
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

    /// [`RangeID`]を[`SingleID`]に分解し、イテレータとして提供します。
    pub fn to_single(&self) -> impl Iterator<Item = SingleID> + '_ {
        let f_range = self.f[0]..=self.f[1];
        let x_range = self.x[0]..=self.x[1];
        let y_range = self.y[0]..=self.y[1];

        iproduct!(f_range, x_range, y_range).map(move |(f, x, y)| SingleID { z: self.z, f, x, y })
    }

    /// 検証を行わずに [`RangeID`] を構築します。
    ///
    /// この関数は [`RangeID::new`] と異なり、与えられた `z`, `f1`, `f2`, `x1`,`x2`, `y1, `y2` に対して
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
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { RangeID::uncheck_new(5, [-10,-5], [8,9], [5,10]) };
    ///
    /// assert_eq!(id.as_z(), 5);
    /// assert_eq!(id.as_f(), [-10,-5]);
    /// assert_eq!(id.as_x(), [8,9]);
    /// assert_eq!(id.as_y(), [5,10]);
    /// ```
    pub unsafe fn uncheck_new(z: u8, f: [i64; 2], x: [u64; 2], y: [u64; 2]) -> RangeID {
        RangeID { z, f, x, y }
    }
}

impl SpaceID for RangeID {
    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.min_f(), -32i64);
    /// ```
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize]
    }

    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.max_f(), 31i64);
    /// ```
    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize]
    }

    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.max_xy(), 31u64);
    /// ```
    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize]
    }

    /// 指定したインデックス差 `by` に基づき、この [`RangeID`] を垂直上方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスがのいずれかが範囲外になる場合は[`Error::FOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_f(), [-10,-5]);
    ///
    /// let _ = id.move_up(4).unwrap();
    /// assert_eq!(id.as_f(), [-6, -1]);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_f(), [-10,-5]);
    ///
    /// assert_eq!(id.move_up(50), Err(Error::FOutOfRange { z: 5, f: 40 }));
    /// ```
    ///
    fn move_up(&mut self, by: u64) -> Result<(), Error> {
        self.move_f(by as i64)
    }

    /// 指定したインデックス差 `by` に基づき、この [`RangeID`] を垂直下方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスがのいずれかが範囲外になる場合は[`Error::FOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_f(), [-10,-5]);
    ///
    /// let _ = id.move_down(4).unwrap();
    /// assert_eq!(id.as_f(), [-14, -9]);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_f(), [-10,-5]);
    ///
    /// assert_eq!(id.move_down(50), Err(Error::FOutOfRange { z: 5, f: -60 }));
    /// ```
    ///
    fn move_down(&mut self, by: u64) -> Result<(), Error> {
        self.move_f(-(by as i64))
    }

    /// 指定したインデックス差 `by` に基づき、この [`RangeID`] を北方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`Error::YOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_y(), [5,10]);
    ///
    /// let _ = id.move_north(4).unwrap();
    /// assert_eq!(id.as_y(), [1, 6]);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_y(), [5,10]);
    ///
    /// assert_eq!(id.move_north(50), Err(Error::YOutOfRange { z: 5, y: 0 }));
    /// ```
    fn move_north(&mut self, by: u64) -> Result<(), Error> {
        self.move_y(-(by as i64))
    }

    /// 指定したインデックス差 `by` に基づき、この [`RangeID`] を南方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`Error::YOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_y(), [5,10]);
    ///
    /// let _ = id.move_south(4).unwrap();
    /// assert_eq!(id.as_y(), [9, 14]);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let mut id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.as_y(), [5,10]);
    ///
    /// assert_eq!(id.move_south(50), Err(Error::YOutOfRange { z: 5, y: 55 }));
    /// ```
    fn move_south(&mut self, by: u64) -> Result<(), Error> {
        self.move_y(by as i64)
    }

    fn move_east(&mut self, by: u64) -> Result<(), Error> {
        self.move_x(by as i64)
    }

    fn move_west(&mut self, by: u64) -> Result<(), Error> {
        self.move_x(-(by as i64))
    }

    fn move_f(&mut self, by: i64) -> Result<(), Error> {
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

    fn move_x(&mut self, by: i64) -> Result<(), Error> {
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

    fn move_y(&mut self, by: i64) -> Result<(), Error> {
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

    /// [`RangeID`] の中心座標を[`Coordinate`]型で返します。
    ///
    /// 中心座標は空間IDの最も外側の頂点の8点の平均座標です。現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let center: Coordinate = id.center();
    /// println!("{:?}", center);
    /// // Coordinate { latitude: 66.51326044311186, longitude: -78.75, altitude: -7340032.0 }
    /// ```
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

    /// [`RangeID`] の最も外側の頂点の8点の座標を[`Coordinate`]型の配列として返します。
    ///
    /// 現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    /// ```
    /// # use kasane_logic::id::space_id::range::RangeID;
    /// # use kasane_logic::error::Error;
    /// # use crate::kasane_logic::id::space_id::SpaceID;
    /// let id = RangeID::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let vertices: [Coordinate; 8] = id.vertices();
    /// println!("{:?}", vertices);
    /// // [Coordinate { latitude: 76.84081641443098, longitude: -90.0, altitude: -10485760.0 }, Coordinate { latitude: 76.84081641443098, longitude: -67.5, altitude: -10485760.0 },....]
    /// ```
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
    ///`SingleID`を[`EncodeID`]に変換します。物理的な範囲に変化はありません。
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
