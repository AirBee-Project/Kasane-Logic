use itertools::iproduct;
use std::{fmt, u64};

use crate::{
    error::Error,
    geometry::coordinate::Coordinate,
    id::space_id::{
        SpaceID,
        constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
        encode::EncodeID,
        helpers,
        range::RangeID,
        segment::Segment,
    },
};

/// SingleIDは標準的な空間 ID を表す型です。
/// 内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// pub struct SingleID {
///     z: u8,
///     f: i64,
///     x: u64,
///     y: u64,
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct SingleID {
    pub(crate) z: u8,
    pub(crate) f: i64,
    pub(crate) x: u64,
    pub(crate) y: u64,
}

impl fmt::Display for SingleID {
    /// `SingleID` を文字列形式で表示します。
    ///
    /// 形式は `"{z}/{f}/{x}/{y}"` です。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use std::fmt::Write;
    /// let id = SingleID::new(4, 6, 9, 10).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/6/9/10");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}/{}", self.z, self.f, self.x, self.y)
    }
}

impl SingleID {
    /// 指定された値から [`SingleID`] を構築します。このコンストラクタは、与えられた `z`, `f`, `x`, `y` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–63の範囲が有効）  
    /// * `f` — Fインデックス（鉛直方向）
    /// * `x` — Xインデックス（東西方向）
    /// * `y` — Yインデックス（南北方向）
    ///
    /// # バリデーション
    /// - `z` が 63 を超える場合、[`Error::ZOutOfRange`] を返します。  
    /// - `f` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、  
    ///   [`Error::FOutOfRange`] を返します。  
    /// - `x` または `y` が `0..=XY_MAX[z]` の範囲外の場合、  
    ///   それぞれ [`Error::XOutOfRange`]、[`Error::YOutOfRange`] を返します。
    ///
    ///
    /// IDの作成:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.to_string(), "5/3/2/10".to_string());
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let id = SingleID::new(3, 3, 2, 10);
    /// assert_eq!(id, Err(Error::YOutOfRange{z:3,y:10}));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let id = SingleID::new(68, 3, 2, 10);
    /// assert_eq!(id, Err(Error::ZOutOfRange { z:68 }));
    /// ```
    pub fn new(z: u8, f: i64, x: u64, y: u64) -> Result<SingleID, Error> {
        //todo
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        if f < f_min || f > f_max {
            return Err(Error::FOutOfRange { f, z });
        }
        if x > xy_max {
            return Err(Error::XOutOfRange { x, z });
        }
        if y > xy_max {
            return Err(Error::YOutOfRange { y, z });
        }

        Ok(SingleID { z, f, x, y })
    }

    /// この `SingleID` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// ```
    pub fn as_z(&self) -> u8 {
        self.z
    }

    /// この `SingleID` が保持している F インデックス `f` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_f(), 3i64);
    /// ```
    pub fn as_f(&self) -> i64 {
        self.f
    }

    /// この `SingleID` が保持している X インデックス `x` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_x(), 2u64);
    /// ```
    pub fn as_x(&self) -> u64 {
        self.x
    }

    /// この `SingleID` が保持している Y インデックス `y` を返します。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_y(), 10u64);
    /// ```
    pub fn as_y(&self) -> u64 {
        self.y
    }

    /// F インデックスを更新します。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `F_MIN[z]..=F_MAX[z]` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `value` — 新しい F インデックス
    ///
    /// # バリデーション
    /// - `value` が許容範囲外の場合、[`Error::FOutOfRange`] を返します。
    ///
    /// 正常な更新:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let mut id = SingleID::new(5, 3, 2, 10).unwrap();
    /// id.set_f(4).unwrap();
    /// assert_eq!(id.as_f(), 4);
    /// ```
    ///
    /// 範囲外の検知:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_f(999);
    /// assert!(matches!(result, Err(Error::FOutOfRange { z: 3, f: 999 })));
    /// ```
    pub fn set_f(&mut self, value: i64) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        if value < min || value > max {
            return Err(Error::FOutOfRange {
                f: value,
                z: self.z,
            });
        }
        self.f = value;
        Ok(())
    }

    /// X インデックスを更新します。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `0..=XY_MAX[z]` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `value` — 新しい X インデックス
    ///
    /// # バリデーション
    /// - `value` が許容範囲外の場合、[`Error::XOutOfRange`] を返します。
    ///
    /// 正常な更新:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let mut id = SingleID::new(5, 3, 2, 10).unwrap();
    /// id.set_x(4).unwrap();
    /// assert_eq!(id.as_x(), 4);
    /// ```
    ///
    /// 範囲外の検知
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_x(999);
    /// assert!(matches!(result, Err(Error::XOutOfRange { z: 3, x: 999 })));
    /// ```
    pub fn set_x(&mut self, value: u64) -> Result<(), Error> {
        let max = self.max_xy();
        if value > max {
            return Err(Error::XOutOfRange {
                x: value,
                z: self.z,
            });
        }
        self.x = value;
        Ok(())
    }

    /// Y インデックスを更新します。
    ///
    /// 与えられた `value` が、現在のズームレベル `z` に対応する
    /// `0..=XY_MAX[z]` の範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `value` — 新しい Y インデックス
    ///
    /// # バリデーション
    /// - `value` が許容範囲外の場合、[`Error::YOutOfRange`] を返します。
    ///
    /// 正常な更新
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let mut id = SingleID::new(5, 3, 2, 10).unwrap();
    /// id.set_y(8).unwrap();
    /// assert_eq!(id.as_y(), 8);
    /// ```
    ///
    /// 範囲外の検知
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_y(999);
    /// assert!(matches!(result, Err(Error::YOutOfRange { z: 3, y: 999 })));
    /// ```
    pub fn set_y(&mut self, value: u64) -> Result<(), Error> {
        let max = self.max_xy();
        if value > max {
            return Err(Error::YOutOfRange {
                y: value,
                z: self.z,
            });
        }
        self.y = value;
        Ok(())
    }

    /// 指定したズームレベル差 `difference` に基づき、この `SingleID` が表す空間のすべての子 `SingleID` を生成します。
    ///
    /// # パラメータ
    /// * `difference` — 子 ID を計算する際に増加させるズームレベル差（差の値が0–63の範囲の場合に有効）
    ///
    /// # バリデーション
    /// - `self.z + difference` が `63` を超える場合、[`Error::ZOutOfRange`] を返します。
    ///
    /// `difference = 1` による細分化
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(3, 3, 2, 7).unwrap();
    ///
    /// // difference = 1 のため F, X, Y はそれぞれ 2 分割される
    /// let children: Vec<_> = id.children(1).unwrap().collect();
    ///
    /// assert_eq!(children.len(), 8); // 2 × 2 × 2
    ///
    /// // 最初の要素を確認（f, x, y の下限側）
    /// let first = &children[0];
    /// assert_eq!(first.as_z(), 4);
    /// assert_eq!(first.as_f(), 3 * 2);   // 2
    /// assert_eq!(first.as_x(), 2 * 2);   // 6
    /// assert_eq!(first.as_y(), 7 * 2);   // 8
    /// ```
    ///
    /// ズームレベルの範囲外
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let id = SingleID::new(3, 3, 2, 7).unwrap();
    /// let result = id.children(63);
    /// assert!(matches!(result, Err(Error::ZOutOfRange { z: 66 })));
    /// ```
    pub fn children(&self, difference: u8) -> Result<impl Iterator<Item = SingleID>, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;

        if z > 63 {
            return Err(Error::ZOutOfRange { z });
        }

        let scale_f = 2_i64.pow(difference as u32);
        let scale_xy = 2_u64.pow(difference as u32);

        let f_range = self.f * scale_f..=self.f * scale_f + scale_f - 1;
        let x_range = self.x * scale_xy..=self.x * scale_xy + scale_xy - 1;
        let y_range = self.y * scale_xy..=self.y * scale_xy + scale_xy - 1;

        Ok(iproduct!(f_range, x_range, y_range).map(move |(f, x, y)| SingleID { z, f, x, y }))
    }

    /// 指定したズームレベル差 `difference` に基づき、この `SingleID` の親 `SingleID` を返します。
    ///
    /// # パラメータ
    /// * `difference` — 親 ID を計算する際に減少させるズームレベル差
    ///
    /// # バリデーション
    /// - `self.z - difference < 0` の場合、親が存在しないため `None` を返します。
    ///
    /// `difference = 1` による上位層への移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(4, 6, 9, 14).unwrap();
    ///
    /// let parent = id.parent(1).unwrap();
    ///
    /// assert_eq!(parent.as_z(), 3);
    /// assert_eq!(parent.as_f(), 3);
    /// assert_eq!(parent.as_x(), 4);
    /// assert_eq!(parent.as_y(), 7);
    /// ```
    ///
    /// Fが負の場合の挙動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(4, -1, 8, 12).unwrap();
    ///
    /// let parent = id.parent(1).unwrap();
    ///
    /// assert_eq!(parent.as_z(), 3);
    /// assert_eq!(parent.as_f(), -1);
    /// assert_eq!(parent.as_x(), 4);
    /// assert_eq!(parent.as_y(), 6);
    /// ```
    ///
    /// ズームレベルの範囲外:
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(3, 3, 2, 7).unwrap();
    ///
    /// // difference = 4 の場合は親が存在しないため None
    /// assert!(id.parent(4).is_none());
    /// ```
    pub fn parent(&self, difference: u8) -> Option<SingleID> {
        let z = self.z.checked_sub(difference)?;
        let f = if self.f == -1 {
            -1
        } else {
            self.f >> difference
        };
        let x = self.x >> (difference as u32);
        let y = self.y >> (difference as u32);
        Some(SingleID { z, f, x, y })
    }

    /// 検証を行わずに [`SingleID`] を構築します。
    ///
    /// この関数は [`SingleID::new`] と異なり、与えられた `z`, `f`, `x`, `y` に対して
    /// 一切の範囲チェックや整合性チェックを行いません。
    /// そのため、高速に ID を生成できますが、**不正なパラメータを与えた場合の動作は未定義です**。
    ///
    /// # 注意
    /// 呼び出し側は、以下をすべて満たすことを保証しなければなりません。
    ///
    /// * `z` が有効なズームレベル（0–63）であること  
    /// * `f` が与えられた `z` に応じて `F_MIN[z]..=F_MAX[z]` の範囲内であること  
    /// * `x` および `y` が `0..=XY_MAX[z]` の範囲内であること  
    ///
    /// これらが保証されない場合、本構造体の他のメソッド（範囲を前提とした計算）が
    /// パニック・不正メモリアクセス・未定義動作を引き起こす可能性があります。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { SingleID::uncheck_new(5, 3, 2, 10) };
    ///
    /// assert_eq!(id.as_z(), 5);
    /// assert_eq!(id.as_f(), 3);
    /// assert_eq!(id.as_x(), 2);
    /// assert_eq!(id.as_y(), 10);
    /// ```
    pub unsafe fn uncheck_new(z: u8, f: i64, x: u64, y: u64) -> SingleID {
        SingleID { z, f, x, y }
    }
}

impl crate::id::space_id::SpaceID for SingleID {
    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.min_f(), -32i64);
    /// ```
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize]
    }

    /// このIDのズームレベルにおける最大の F インデックスを返す
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.max_f(), 31i64);
    /// ```
    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize]
    }

    /// このIDのズームレベルにおける最大の XY インデックスを返す
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.as_z(), 5u8);
    /// assert_eq!(id.max_xy(), 31u64);
    /// ```
    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize]
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を垂直上方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスが範囲外になる場合は[`Error::FOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 14).unwrap();
    /// assert_eq!(id.as_f(), 6);
    ///
    /// let _ = id.move_up(4).unwrap();
    /// assert_eq!(id.as_f(), 10);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 14).unwrap();
    /// assert_eq!(id.as_f(), 6);
    ///
    /// assert_eq!(id.move_up(20), Err(Error::FOutOfRange { z: 4, f: 26 }));
    /// ```
    ///
    fn move_up(&mut self, by: u64) -> Result<(), Error> {
        self.move_f(by as i64)
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を垂直下方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスが範囲外になる場合は[`Error::FOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 14).unwrap();
    /// assert_eq!(id.as_f(), 6);
    ///
    /// let _ = id.move_down(4).unwrap();
    /// assert_eq!(id.as_f(), 2);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 14).unwrap();
    /// assert_eq!(id.as_f(), 6);
    /// assert_eq!(id.move_down(50), Err(Error::FOutOfRange { z: 4, f: -44 }));
    /// ```
    fn move_down(&mut self, by: u64) -> Result<(), Error> {
        self.move_f(-(by as i64))
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を北方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`Error::YOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_y(), 10);
    ///
    /// let _ = id.move_north(4).unwrap();
    /// assert_eq!(id.as_y(), 6);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_f(), 6);
    /// assert_eq!(id.move_north(50), Err(Error::YOutOfRange { z: 4, y: 0 }));
    /// ```
    fn move_north(&mut self, by: u64) -> Result<(), Error> {
        self.move_y(-(by as i64))
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を南方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`Error::YOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_y(), 10);
    ///
    /// let _ = id.move_south(4).unwrap();
    /// assert_eq!(id.as_y(), 14);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_f(), 6);
    /// assert_eq!(id.move_south(50), Err(Error::YOutOfRange { z: 4, y: 60 }));
    /// ```
    fn move_south(&mut self, by: u64) -> Result<(), Error> {
        self.move_y(by as i64)
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を東方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Xインデックスが範囲外になる場合は[`Error::XOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_x(), 9);
    ///
    /// let _ = id.move_east(4).unwrap();
    /// assert_eq!(id.as_x(), 13);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_x(), 9);
    /// assert_eq!(id.move_east(50), Err(Error::XOutOfRange { z: 4, x: 59 }));
    /// ```
    fn move_east(&mut self, by: u64) -> Result<(), Error> {
        self.move_x(by as i64)
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を西方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Xインデックスが範囲外になる場合は[`Error::XOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_x(), 9);
    ///
    /// let _ = id.move_west(4).unwrap();
    /// assert_eq!(id.as_x(), 5);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_x(), 9);
    /// assert_eq!(id.move_west(50), Err(Error::XOutOfRange { z: 4, x: 0 }));
    /// ```
    fn move_west(&mut self, by: u64) -> Result<(), Error> {
        self.move_x(-(by as i64))
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を垂直上下方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスが範囲外になる場合は[`Error::FOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_f(), 6);
    ///
    /// let _ = id.move_f(-10).unwrap();
    /// assert_eq!(id.as_f(), -4);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_f(), 6);
    /// assert_eq!(id.move_f(50), Err(Error::FOutOfRange { z: 4, f: 56 }));
    /// ```
    fn move_f(&mut self, by: i64) -> Result<(), Error> {
        let new = self.f.checked_add(by).ok_or(Error::FOutOfRange {
            f: if by >= 0 { i64::MAX } else { i64::MIN },
            z: self.z,
        })?;

        if new < self.min_f() || new > self.max_f() {
            return Err(Error::FOutOfRange { f: new, z: self.z });
        }

        self.f = new;

        Ok(())
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を東西方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Xインデックスが範囲外になる場合は[`Error::XOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_x(), 9);
    ///
    /// let _ = id.move_x(-3).unwrap();
    /// assert_eq!(id.as_x(), 6);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_x(), 9);
    /// assert_eq!(id.move_x(-10), Err(Error::XOutOfRange { z: 4, x: 0 }));
    /// ```
    fn move_x(&mut self, by: i64) -> Result<(), Error> {
        let new = if by >= 0 {
            self.x.checked_add(by as u64).ok_or(Error::XOutOfRange {
                x: u64::MAX,
                z: self.z,
            })?
        } else {
            self.x
                .checked_sub(-by as u64)
                .ok_or(Error::XOutOfRange { x: 0, z: self.z })?
        };

        if new > self.max_xy() {
            return Err(Error::XOutOfRange { x: new, z: self.z });
        }

        self.x = new;

        Ok(())
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleID` を南北方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`Error::YOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_y(), 10);
    ///
    /// let _ = id.move_y(-3).unwrap();
    /// assert_eq!(id.as_y(), 7);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::error::Error;
    /// let mut id = SingleID::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.as_y(), 10);
    /// assert_eq!(id.move_y(-20), Err(Error::YOutOfRange { z: 4, y: 0 }));
    /// ```
    fn move_y(&mut self, by: i64) -> Result<(), Error> {
        let new = if by >= 0 {
            self.y.checked_add(by as u64).ok_or(Error::YOutOfRange {
                y: u64::MAX,
                z: self.z,
            })?
        } else {
            self.y
                .checked_sub(-by as u64)
                .ok_or(Error::YOutOfRange { y: 0, z: self.z })?
        };

        if new > self.max_xy() {
            return Err(Error::YOutOfRange { y: new, z: self.z });
        }

        self.y = new;

        Ok(())
    }

    /// `SingleID` の中心座標を[`Coordinate`]型で返します。
    ///
    /// 中心座標は空間IDの最も外側の頂点の8点の平均座標です。現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::geometry::coordinate::Coordinate;
    /// let id = SingleID::new(4, 6, 9, 14).unwrap();
    /// let center: Coordinate = id.center();
    /// println!("{:?}", center);
    /// // Coordinate { latitude: -81.09321385260839, longitude: 33.75, altitude: 13631488.0 }
    /// ```
    fn center(&self) -> Coordinate {
        unsafe {
            Coordinate::uncheck_new(
                helpers::latitude(self.y as f64 + 0.5, self.z),
                helpers::longitude(self.x as f64 + 0.5, self.z),
                helpers::altitude(self.f as f64 + 0.5, self.z),
            )
        }
    }

    /// `SingleID` の最も外側の頂点の8点の座標を[`Coordinate`]型の配列として返します。
    ///
    /// 現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    ///
    /// ```
    /// # use kasane_logic::id::space_id::SpaceID;
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::geometry::coordinate::Coordinate;
    /// let id = SingleID::new(4, 6, 9, 14).unwrap();
    /// let vertices: [Coordinate; 8] = id.vertices();
    /// println!("{:?}", vertices);
    ///
    ///  //[Coordinate { latitude: -79.17133464081945, longitude: 22.5, altitude: 12582912.0 }, Coordinate { latitude: -79.17133464081945, longitude: 45.0, altitude: 12582912.0 }, Coordinate { latitude: -82.67628497834903, longitude: 22.5, altitude: 12582912.0 }, Coordinate { latitude: -82.67628497834903, longitude: 45.0, altitude: 12582912.0 }, Coordinate { latitude: -79.17133464081945, longitude: 22.5, altitude: 14680064.0 }, Coordinate { latitude: -79.17133464081945, longitude: 45.0, altitude: 14680064.0 }, Coordinate { latitude: -82.67628497834903, longitude: 22.5, altitude: 14680064.0 }, Coordinate { latitude: -82.67628497834903, longitude: 45.0, altitude: 14680064.0 }]
    /// ```
    fn vertices(&self) -> [Coordinate; 8] {
        use itertools::iproduct;

        let xs = [self.x as f64, self.x as f64 + 1.0];
        let ys = [self.y as f64, self.y as f64 + 1.0];
        let fs = [self.f as f64, self.f as f64 + 1.0];

        // 各端点の値を前計算しておく（計算コスト削減）
        let lon2 = [
            helpers::longitude(xs[0], self.z),
            helpers::longitude(xs[1], self.z),
        ];
        let lat2 = [
            helpers::latitude(ys[0], self.z),
            helpers::latitude(ys[1], self.z),
        ];
        let alt2 = [
            helpers::altitude(fs[0], self.z),
            helpers::altitude(fs[1], self.z),
        ];

        // 結果配列（Default を利用）
        let mut out = [Coordinate::default(); 8];

        for (i, (f_i, y_i, x_i)) in iproduct!(0..2, 0..2, 0..2).enumerate() {
            out[i]
                .set_longitude(lon2[x_i])
                .expect("longitude must be within valid range");
            out[i]
                .set_latitude(lat2[y_i])
                .expect("latitude must be within valid range");
            out[i]
                .set_altitude(alt2[f_i])
                .expect("altitude must be within valid range");
        }

        out
    }
}

impl From<SingleID> for EncodeID {
    ///`SingleID`を[`EncodeID`]に変換します。物理的な範囲に変化はありません。
    fn from(id: SingleID) -> Self {
        let f_bitvec = Segment { z: id.z, dim: id.f }.into();
        let x_bitvec = Segment { z: id.z, dim: id.x }.into();
        let y_bitvec = Segment { z: id.z, dim: id.y }.into();

        EncodeID {
            f: vec![f_bitvec],
            x: vec![x_bitvec],
            y: vec![y_bitvec],
        }
    }
}

impl From<SingleID> for RangeID {
    ///`SingleID`を[`RangeID`]に変換します。表す物理的な範囲に変化はありません。
    fn from(id: SingleID) -> Self {
        RangeID {
            z: id.z,
            f: [id.f, id.f],
            x: [id.x, id.x],
            y: [id.y, id.y],
        }
    }
}
