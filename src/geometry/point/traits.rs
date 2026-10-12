use crate::{Ecef, Error, RangeId, SingleId};

///現実空間の点に対して共通で定義することができる性質
pub trait Point: Into<Ecef> {
    /// この点を、指定されたズームレベルの [`SingleId`] に変換する。
    ///
    /// 高度・経度・緯度が空間 ID の範囲の上端ちょうどにある点は、最後のインデックス値に丸められる。
    /// ズームレベルが範囲外の場合は [`Error`] を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::{Coordinate, Point, SingleId};
    /// let coord = Coordinate::new(34.9851603, 135.7584294, 20.0).unwrap();
    ///
    /// assert_eq!(
    ///     coord.single_id(24).unwrap(),
    ///     SingleId::new(24, 10, 14715409, 6646263).unwrap()
    /// );
    ///
    /// // ズームレベルが範囲外の場合はエラーになる
    /// assert!(coord.single_id(31).is_err());
    /// ```
    fn single_id(&self, z: impl Into<u8>) -> Result<SingleId, Error>;

    /// この点を、指定されたズームレベルの [`RangeId`] に変換する。
    ///
    /// 結果は、[`Point::single_id`] で得た [`SingleId`] と同じ範囲を表す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::{Coordinate, Point, RangeId};
    /// let coord = Coordinate::new(35.681382, 139.766084, 10.3).unwrap();
    ///
    /// assert_eq!(
    ///     coord.range_id(20).unwrap(),
    ///     RangeId::from(coord.single_id(20).unwrap())
    /// );
    /// ```
    fn range_id(&self, z: impl Into<u8>) -> Result<RangeId, Error> {
        Ok(RangeId::from(self.single_id(z)?))
    }
}
