use crate::spatial_id::{dimension::Dimension, relative_flex_id::RelativeFlexId};
use crate::{Error, FlexId};
use alloc::boxed::Box;
use core::fmt;

/// Skip の行き先の [RelativeFlexId] と、Skip が割っている次元の集合を 8B に詰めたもの。
///
/// 行き先はほとんどが数段の深さなので、各次元の深さが [`INDEX_BITS`] 以下なら深さとインデックスを
/// 1つの `u64` に詰める。収まらない行き先だけを別に確保し、そのポインタを持つ。
/// インデックスは次元ごとに決まった位置に置き、各次元を他の次元の深さに関係なく取り出せるようにする
/// （木を辿るたびに復元するので、ここが遅いと全体が遅くなる）。
/// 最下位ビットが 1 なら詰めた値、0 なら確保した [Long] へのポインタ（アラインメントで最下位ビットは 0）。
///
/// 割っている次元の集合は子から求まるが、求めるには子の Branch を読みに行くことになり、
/// 木を辿る処理が遅くなるのでここに持つ。
///
/// 詰めた値のビット配置（下位から）:
/// - 0: タグ（1）
/// - 1..5: 深くなっている次元の集合（[`Dimension::bit`] の OR）
/// - 5..9: Skip が割っている次元の集合（深くなっている次元と子の集合を合わせたもの）
/// - 9..25: F / X / Y / T の深さ（4 ビットずつ）
/// - 25..64: F / X / Y / T のインデックス（[`INDEX_BITS`] ずつ）
pub(super) struct SkipPath(u64);

/// 詰めきれない行き先。
struct Long {
    path: RelativeFlexId,
    split_dimensions: u8,
}

const INLINE: u64 = 1;
const MASK_BITS: u64 = 0b1111;
const DEEPER_SHIFT: u32 = 1;
const SPLIT_SHIFT: u32 = 5;
const DEPTH_SHIFT: [u32; 4] = [9, 13, 17, 21];
/// F / X / Y / T のインデックスに使うビット数。詰められる深さの上限でもある。
const INDEX_BITS: [u32; 4] = [10, 10, 10, 9];
const INDEX_SHIFT: [u32; 4] = [25, 35, 45, 55];

// SAFETY: 確保した行き先は自身だけが所有する Box と同じなので、Box<Long> と同じく送れる
unsafe impl Send for SkipPath {}
unsafe impl Sync for SkipPath {}

impl SkipPath {
    /// `ancestor` から `region` への行き先。`child_split` は Skip の先の Node が割っている次元の集合。
    ///
    /// `region` は `ancestor` に含まれること。
    pub(super) fn new(region: &FlexId, ancestor: &FlexId, child_split: u8) -> Self {
        let parts = region
            .relative_parts(ancestor)
            .expect("Skip の行き先は祖先に含まれる");
        let deeper = Dimension::mask(|d| parts[d as usize].0 > 0);
        let split_dimensions = deeper | child_split;
        if Dimension::ALL.iter().any(|&d| u32::from(parts[d as usize].0) > INDEX_BITS[d as usize]) {
            let long = Box::new(Long {
                // SAFETY: `relative_parts` の値なので、深さもインデックスも範囲内
                path: unsafe { RelativeFlexId::from_parts_unchecked(parts) },
                split_dimensions,
            });
            return SkipPath(Box::into_raw(long) as usize as u64);
        }
        let mut bits = INLINE
            | u64::from(deeper) << DEEPER_SHIFT
            | u64::from(split_dimensions) << SPLIT_SHIFT;
        for (d, (z, index)) in parts.into_iter().enumerate() {
            bits |= u64::from(z) << DEPTH_SHIFT[d] | index << INDEX_SHIFT[d];
        }
        SkipPath(bits)
    }

    /// 別に確保した行き先。詰めた値なら [`None`]。
    fn boxed(&self) -> Option<&Long> {
        // SAFETY: タグが 0 なら `new` で Box から取り出したポインタで、drop まで有効
        (self.0 & INLINE == 0).then(|| unsafe { &*(self.0 as usize as *const Long) })
    }

    pub(super) fn get(&self) -> RelativeFlexId {
        if let Some(long) = self.boxed() {
            return long.path;
        }
        // SAFETY: `new` が有効な RelativeFlexId から詰めた値なので、深さもインデックスも範囲内
        unsafe { RelativeFlexId::from_parts_unchecked(self.inline_parts()) }
    }

    /// 詰めた値の F / X / Y / T の `(深さ, インデックス)`。
    fn inline_parts(&self) -> [(u8, u64); 4] {
        Dimension::ALL.map(|d| {
            let index = self.0 >> INDEX_SHIFT[d as usize] & ((1 << INDEX_BITS[d as usize]) - 1);
            (self.inline_depth(d), index)
        })
    }

    fn inline_depth(&self, dimension: Dimension) -> u8 {
        (self.0 >> DEPTH_SHIFT[dimension as usize] & MASK_BITS) as u8
    }

    /// 祖先から `dimension` において何段深いか。
    pub(super) fn depth_on(&self, dimension: Dimension) -> u8 {
        match self.boxed() {
            Some(long) => long.path.depth_on(dimension),
            None => self.inline_depth(dimension),
        }
    }

    /// 祖先より深くなっている次元の集合（[`Dimension::bit`] の OR）。
    pub(super) fn deeper_dimensions(&self) -> u8 {
        match self.boxed() {
            Some(long) => long.path.deeper_dimensions(),
            None => (self.0 >> DEEPER_SHIFT & MASK_BITS) as u8,
        }
    }

    /// Skip が割っている次元の集合。深くなっている次元と、Skip の先の Node の集合を合わせたもの。
    pub(super) fn split_dimensions(&self) -> u8 {
        match self.boxed() {
            Some(long) => long.split_dimensions,
            None => (self.0 >> SPLIT_SHIFT & MASK_BITS) as u8,
        }
    }

    /// Skip の先の Node が変わったときに、割っている次元の集合を合わせる。
    pub(super) fn set_child_split(&mut self, child_split: u8) {
        let split_dimensions = self.deeper_dimensions() | child_split;
        if self.0 & INLINE == 0 {
            // SAFETY: タグが 0 なら `new` で Box から取り出したポインタで、&mut self なので他から参照されていない
            unsafe { (*(self.0 as usize as *mut Long)).split_dimensions = split_dimensions };
        } else {
            self.0 = self.0 & !(MASK_BITS << SPLIT_SHIFT) | u64::from(split_dimensions) << SPLIT_SHIFT;
        }
    }

    /// `ancestor` を原点として、[FlexId] を復元する。
    pub(super) fn to_absolute(&self, ancestor: &FlexId) -> Result<FlexId, Error> {
        match self.boxed() {
            Some(long) => long.path.to_absolute(ancestor),
            None => RelativeFlexId::absolute_from_parts(ancestor, self.inline_parts()),
        }
    }
}

impl Drop for SkipPath {
    fn drop(&mut self) {
        if self.0 & INLINE == 0 {
            // SAFETY: タグが 0 なら `new` で Box から取り出したポインタで、ここで一度だけ戻す
            drop(unsafe { Box::from_raw(self.0 as usize as *mut Long) });
        }
    }
}

impl Clone for SkipPath {
    fn clone(&self) -> Self {
        match self.boxed() {
            Some(long) => {
                let long = Box::new(Long {
                    path: long.path,
                    split_dimensions: long.split_dimensions,
                });
                SkipPath(Box::into_raw(long) as usize as u64)
            }
            None => SkipPath(self.0),
        }
    }
}

// 詰めるかどうかは深さだけで決まるので、同じ行き先は必ず同じ表し方になる
impl PartialEq for SkipPath {
    fn eq(&self, other: &Self) -> bool {
        match (self.boxed(), other.boxed()) {
            (Some(a), Some(b)) => a.path == b.path && a.split_dimensions == b.split_dimensions,
            (None, None) => self.0 == other.0,
            _ => false,
        }
    }
}

impl Eq for SkipPath {}

impl fmt::Debug for SkipPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 詰めた行き先も別に確保した行き先も、元の [RelativeFlexId] に戻る。
    #[test]
    fn round_trips_inline_and_boxed() {
        let root = FlexId::UPPER_MAX;
        let cases = [
            FlexId::new(1, 1, 0, 0, 0, 0).unwrap(),
            FlexId::new(3, 5, 4, 9, 2, 3).unwrap(),
            // 各次元の深さがちょうど INDEX_BITS
            FlexId::new(10, 1023, 10, 1000, 10, 777).unwrap(),
            // 1つの次元だけ収まらないので別に確保する
            FlexId::new(11, 2047, 1, 1, 0, 0).unwrap(),
            FlexId::new(25, 1 << 24, 25, (1 << 25) - 1, 25, 3).unwrap(),
        ];
        for id in cases {
            let relative = id.relative_to(&root).unwrap();
            let mut path = SkipPath::new(&id, &root, Dimension::T.bit());
            assert_eq!(path.get(), relative);
            assert_eq!(path.split_dimensions(), relative.deeper_dimensions() | Dimension::T.bit());
            path.set_child_split(0);
            assert_eq!(path.split_dimensions(), relative.deeper_dimensions());
            assert_eq!(path.clone(), path);
            assert_eq!(path.to_absolute(&root), Ok(id));
            for d in Dimension::ALL {
                assert_eq!(path.depth_on(d), relative.depth_on(d));
            }
        }
    }

    /// Skip が 8B の行き先と子だけを持つ大きさに収まる。
    #[test]
    fn fits_in_one_word() {
        assert_eq!(size_of::<SkipPath>(), 8);
    }
}
