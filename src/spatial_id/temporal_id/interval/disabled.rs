use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Interval;

impl Interval {
    /// このライブラリが扱える全時間の秒数（`2^62`秒、約1,460億年）。
    ///
    /// 有効版の[`TZoomLevel::MAX`](crate::spatial_id::temporal_id::zoom_level::TZoomLevel::MAX)と
    /// 一致させること（`TemporalSegment::WHOLE`が表す絶対秒区間と一致させるため）。
    pub const WHOLE_SECONDS: u64 = 1u64 << Self::WHOLE_POW;

    /// 最も粗い時間区間を表す二進層の指数。
    pub const WHOLE_POW: u8 = 62;

    /// 全時間（`2^62`秒）。`temporal_id` feature 無効時に唯一有効な値。
    pub const WHOLE: Interval = Interval;

    /// 秒数から [`Interval`] を作成する。
    ///
    /// `temporal_id` feature 無効時は [`WHOLE_SECONDS`](Self::WHOLE_SECONDS) のみ受け付ける。
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        if seconds == Self::WHOLE_SECONDS {
            Ok(Interval)
        } else {
            Err(crate::SpatialIdError::TIntervalError { i: seconds }.into())
        }
    }

    /// この間隔の秒数。
    pub const fn seconds(self) -> u64 {
        Self::WHOLE_SECONDS
    }
}

impl TryFrom<u64> for Interval {
    type Error = Error;
    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        Self::new(seconds)
    }
}

macro_rules! impl_try_from_unsigned {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for Interval {
                type Error = Error;

                fn try_from(seconds: $t) -> Result<Self, Self::Error> {
                    Self::try_from(seconds as u64)
                }
            }
        )*
    };
}

impl_try_from_unsigned!(u8, u16, u32, u128, usize);
