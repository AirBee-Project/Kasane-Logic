/// 時空間IDの次元。F/X/Yは空間3軸（各最大ズーム30）、Tは時間軸の生の2分岐Segment
/// （最大ズーム`TZoomLevel::MAX`=35）。並び順（F→X→Y→T）は木が次元を選ぶときの規約で、
/// 他に意味は無い。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[repr(u8)]
pub enum Dimension {
    F = 0,
    X = 1,
    Y = 2,
    T = 3,
}

impl Dimension {
    pub const ALL: [Dimension; 4] = [Dimension::F, Dimension::X, Dimension::Y, Dimension::T];

    pub(crate) const fn bit(self) -> u8 {
        1 << self as u8
    }

    pub(crate) fn mask(predicate: impl Fn(Dimension) -> bool) -> u8 {
        Dimension::ALL
            .into_iter()
            .filter(|&d| predicate(d))
            .fold(0, |set, d| set | d.bit())
    }
}
