#[cfg(any(test, feature = "random"))]
use rand::Rng;
#[cfg(any(test, feature = "random"))]
use std::ops::RangeInclusive;

#[cfg(any(test))]
use proptest::prelude::*;

pub mod impls;

use crate::{
    error::Error,
    spatial_id::{
        SpatialId,
        constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
        temporal_id::TemporalId,
    },
};

/// SingleIdは標準的な時空間 ID を表す型。
///
/// 内部的には下記のような構造体で構成されている。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用であり、実際の空間的な「大小」を意味するものではない。
///
/// ```
/// # use kasane_logic::TemporalId;
/// pub struct SingleId {
///     z: u8,
///     f: i32,
///     x: u32,
///     y: u32,
///     temporal_id: TemporalId,
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct SingleId {
    z: u8,
    f: i32,
    x: u32,
    y: u32,
    temporal_id: TemporalId,
}

impl SingleId {
    /// 指定された値から [`SingleId`] を構築します。このコンストラクタは、与えられた `z`, `f`, `x`, `y` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返す
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–63の範囲が有効）  
    /// * `f` — Fインデックス（鉛直方向）
    /// * `x` — Xインデックス（東西方向）
    /// * `y` — Yインデックス（南北方向）
    ///
    /// # バリデーション
    /// - `z` が 63 を超える場合、[`Error::ZOutOfRange`] を返す。  
    /// - `f` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、  
    ///   [`Error::FOutOfRange`] を返す。  
    /// - `x` または `y` が `0..=XY_MAX[z]` の範囲外の場合、  
    ///   それぞれ [`Error::XOutOfRange`]、[`Error::YOutOfRange`] を返す。
    ///
    ///
    /// IDの作成:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.to_string(), "5/3/2/10".to_string());
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Error;
    /// let id = SingleId::new(3, 3, 2, 10);
    /// assert_eq!(id, Err(Error::YOutOfRange{z:3,y:10}));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Error;
    /// let id = SingleId::new(68, 3, 2, 10);
    /// assert_eq!(id, Err(Error::ZOutOfRange { z:68 }));
    /// ```
    pub fn new(z: u8, f: i32, x: u32, y: u32) -> Result<SingleId, Error> {
        Self::new_with_temporal(z, f, x, y, TemporalId::whole())
    }

    pub fn new_with_temporal(
        z: u8,
        f: i32,
        x: u32,
        y: u32,
        temporal_id: TemporalId,
    ) -> Result<SingleId, Error> {
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

        Ok(SingleId {
            z,
            f,
            x,
            y,
            temporal_id,
        })
    }

    /// この `SingleId` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// ```
    pub fn z(&self) -> u8 {
        self.z
    }

    /// この `SingleId` が保持している F インデックス `f` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.f(), 3i32);
    /// ```
    pub fn f(&self) -> i32 {
        self.f
    }

    /// この `SingleId` が保持している X インデックス `x` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.x(), 2u32);
    /// ```
    pub fn x(&self) -> u32 {
        self.x
    }

    /// この `SingleId` が保持している Y インデックス `y` を返します。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.y(), 10u32);
    /// ```
    pub fn y(&self) -> u32 {
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
    /// # use kasane_logic::SingleId;
    /// let mut id = SingleId::new(5, 3, 2, 10).unwrap();
    /// id.set_f(4).unwrap();
    /// assert_eq!(id.f(), 4);
    /// ```
    ///
    /// 範囲外の検知:
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Error;
    /// let mut id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_f(999);
    /// assert!(matches!(result, Err(Error::FOutOfRange { z: 3, f: 999 })));
    /// ```
    pub fn set_f(&mut self, value: i32) -> Result<(), Error> {
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
    /// # use kasane_logic::SingleId;
    /// let mut id = SingleId::new(5, 3, 2, 10).unwrap();
    /// id.set_x(4).unwrap();
    /// assert_eq!(id.x(), 4);
    /// ```
    ///
    /// 範囲外の検知
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Error;
    /// let mut id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_x(999);
    /// assert!(matches!(result, Err(Error::XOutOfRange { z: 3, x: 999 })));
    /// ```
    pub fn set_x(&mut self, value: u32) -> Result<(), Error> {
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
    /// # use kasane_logic::SingleId;
    /// let mut id = SingleId::new(5, 3, 2, 10).unwrap();
    /// id.set_y(8).unwrap();
    /// assert_eq!(id.y(), 8);
    /// ```
    ///
    /// 範囲外の検知
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Error;
    /// let mut id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.set_y(999);
    /// assert!(matches!(result, Err(Error::YOutOfRange { z: 3, y: 999 })));
    /// ```
    pub fn set_y(&mut self, value: u32) -> Result<(), Error> {
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

    /// 指定したズームレベル差 `difference` に基づき、この `SingleId` が表す空間のすべての子 `SingleId` を生成します。
    ///
    /// # パラメータ
    /// * `difference` — 子 ID を計算する際に増加させるズームレベル差（差の値が0–63の範囲の場合に有効）
    ///
    /// # バリデーション
    /// - `self.z + difference` が `63` を超える場合、[`Error::ZOutOfRange`] を返します。
    ///
    /// `difference = 1` による細分化
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    ///
    /// // difference = 1 のため F, X, Y はそれぞれ 2 分割される
    /// let children: Vec<_> = id.spatial_children(1).unwrap().collect();
    ///
    /// assert_eq!(children.len(), 8); // 2 × 2 × 2
    ///
    /// // 最初の要素を確認（f, x, y の下限側）
    /// let first = &children[0];
    /// assert_eq!(first.z(), 4);
    /// assert_eq!(first.f(), 3 * 2);   // 2
    /// assert_eq!(first.x(), 2 * 2);   // 6
    /// assert_eq!(first.y(), 7 * 2);   // 8
    /// ```
    ///
    /// ズームレベルの範囲外
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Error;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    /// let result = id.spatial_children(63);
    /// assert!(matches!(result, Err(Error::ZOutOfRange { z: 66 })));
    /// ```
    pub fn spatial_children(
        &self,
        difference: u8,
    ) -> Result<impl Iterator<Item = SingleId>, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;

        if z as usize > MAX_ZOOM_LEVEL {
            return Err(Error::ZOutOfRange { z });
        }

        let scale_f = 2_i32.pow(difference as u32);
        let scale_xy = 2_u32.pow(difference as u32);

        let f_start = self.f * scale_f;
        let x_start = self.x * scale_xy;
        let y_start = self.y * scale_xy;

        let f_range = f_start..=f_start + scale_f - 1;
        let x_range = x_start..=x_start + scale_xy - 1;
        let y_range = y_start..=y_start + scale_xy - 1;

        Ok(f_range.flat_map(move |f| {
            let x_range = x_range.clone();
            let y_range = y_range.clone();

            x_range.flat_map(move |x| {
                y_range.clone().map(move |y| SingleId {
                    z,
                    f,
                    x,
                    y,
                    temporal_id: self.temporal().clone(),
                })
            })
        }))
    }

    /// 指定したズームレベル差 `difference` に基づき、この `SingleId` の親 `SingleId` を返します。
    ///
    /// # パラメータ
    /// * `difference` — 親 ID を計算する際に減少させるズームレベル差
    ///
    /// # バリデーション
    /// - `self.z - difference < 0` の場合、親が存在しないため `None` を返します。
    ///
    /// `difference = 1` による上位層への移動
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, 6, 9, 14).unwrap();
    ///
    /// let parent = id.spatial_parent(1).unwrap();
    ///
    /// assert_eq!(parent.z(), 3u8);
    /// assert_eq!(parent.f(), 3i32);
    /// assert_eq!(parent.x(), 4u32);
    /// assert_eq!(parent.y(), 7u32);
    /// ```
    ///
    /// Fが負の場合の挙動
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(4, -1, 8, 12).unwrap();
    ///
    /// let parent = id.spatial_parent(1).unwrap();
    ///
    /// assert_eq!(parent.z(), 3u8);
    /// assert_eq!(parent.f(), -1i32);
    /// assert_eq!(parent.x(), 4u32);
    /// assert_eq!(parent.y(), 6u32);
    /// ```
    ///
    /// ズームレベルの範囲外:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(3, 3, 2, 7).unwrap();
    ///
    /// // difference = 4 の場合は親が存在しないため None
    /// assert!(id.spatial_parent(4).is_none());
    /// ```
    pub fn spatial_parent(&self, difference: u8) -> Option<SingleId> {
        let z = self.z.checked_sub(difference)?;
        let f = if self.f == -1 {
            -1
        } else {
            self.f >> difference
        };
        let x = self.x >> (difference as u32);
        let y = self.y >> (difference as u32);
        Some(SingleId {
            z,
            f,
            x,
            y,
            temporal_id: self.temporal().clone(),
        })
    }

    /// 検証を行わずに [`SingleId`] を構築します。
    ///
    /// この関数は [`SingleId::new`] と異なり、与えられた `z`, `f`, `x`, `y` に対して
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
    /// # use kasane_logic::SingleId;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { SingleId::new_unchecked(5, 3, 2, 10) };
    ///
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.f(), 3i32);
    /// assert_eq!(id.x(), 2u32);
    /// assert_eq!(id.y(), 10u32);
    /// ```
    pub unsafe fn new_unchecked(z: u8, f: i32, x: u32, y: u32) -> SingleId {
        unsafe { Self::new_with_temporal_unchecked(z, f, x, y, TemporalId::whole()) }
    }

    pub unsafe fn new_with_temporal_unchecked(
        z: u8,
        f: i32,
        x: u32,
        y: u32,
        temporal_id: TemporalId,
    ) -> SingleId {
        SingleId {
            z,
            f,
            x,
            y,
            temporal_id,
        }
    }

    ///ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random() -> Self {
        Self::random_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    ///特定のズームレベルにおいて、ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_at(z: u8) -> Self {
        Self::random_within(z..=z)
    }

    ///特定のズームレベル間において、ランダムな[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_within(z: RangeInclusive<u8>) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let start = *z.start();
        let end = (*z.end()).min(MAX_ZOOM_LEVEL as u8);

        let z = if start > end {
            end
        } else {
            rng.random_range(start..=end)
        };

        let z_idx = z as usize;
        let f = rng.random_range(F_MIN[z_idx]..=F_MAX[z_idx]);
        let x = rng.random_range(0..=XY_MAX[z_idx]);
        let y = rng.random_range(0..=XY_MAX[z_idx]);

        SingleId::new(z, f, x, y).expect("Failed to generate random SingleId")
    }

    #[cfg(any(test))]
    pub fn arb() -> impl Strategy<Value = Self> {
        Self::arb_within(0..=MAX_ZOOM_LEVEL as u8)
    }

    #[cfg(any(test))]
    pub fn arb_at(z: u8) -> impl Strategy<Value = Self> {
        Self::arb_within(z..=z)
    }

    #[cfg(any(test))]
    pub fn arb_within(z_range: RangeInclusive<u8>) -> impl Strategy<Value = Self> {
        z_range.prop_flat_map(|z| {
            let z_idx = z as usize;

            let f_strategy = F_MIN[z_idx]..=F_MAX[z_idx];
            let x_strategy = 0..=XY_MAX[z_idx];
            let y_strategy = 0..=XY_MAX[z_idx];

            (Just(z), f_strategy, x_strategy, y_strategy).prop_map(|(z, f, x, y)| {
                Self::new(z, f, x, y).expect("Strategy generated invalid ID")
            })
        })
    }

    #[cfg(any(test, feature = "random"))]
    /// 外部から渡された乱数生成器を使用して、特定のズームレベルの[SingleId]を作成する
    pub fn random_using<R: Rng>(rng: &mut R) -> Self {
        Self::random_within_using(rng, 0..=MAX_ZOOM_LEVEL as u8)
    }

    /// 外部から渡された乱数生成器を使用して、特定のズームレベルの[SingleId]を作成する
    #[cfg(any(test, feature = "random"))]
    pub fn random_at_using<R: Rng>(rng: &mut R, z: u8) -> Self {
        Self::random_within_using(rng, z..=z)
    }

    /// 外部から渡された乱数生成器を使用して、特定範囲の[SingleId]を作成する
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

        // F, X, Y の範囲生成も渡された rng を使用
        let f = rng.random_range(F_MIN[z_idx]..=F_MAX[z_idx]);
        let x = rng.random_range(0..=XY_MAX[z_idx]);
        let y = rng.random_range(0..=XY_MAX[z_idx]);

        SingleId::new(z, f, x, y).expect("Failed to generate random SingleId")
    }
}
