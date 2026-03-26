use std::{
    collections::BTreeMap,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign},
};

use roaring::{MultiOps, RoaringBitmap};

use crate::FlexIdRank;

#[derive(Debug, Clone, Default)]
pub struct FlexIdRankList {
    pub map: BTreeMap<u128, RoaringBitmap>,
}

impl FlexIdRankList {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, value: FlexIdRank) -> bool {
        let (high_128, low_32) = value.split();
        let bitmap = self.map.entry(high_128).or_insert_with(RoaringBitmap::new);
        bitmap.insert(low_32)
    }

    pub fn remove(&mut self, value: FlexIdRank) -> bool {
        let (high_128, low_32) = value.split();
        if let Some(bitmap) = self.map.get_mut(&high_128) {
            let removed = bitmap.remove(low_32);
            if bitmap.is_empty() {
                self.map.remove(&high_128);
            }
            removed
        } else {
            false
        }
    }

    pub fn contains(&self, value: FlexIdRank) -> bool {
        let (high_128, low_32) = value.split();
        self.map
            .get(&high_128)
            .map_or(false, |bitmap| bitmap.contains(low_32))
    }

    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = BTreeMap::new();
        for (high_128, bitmap_a) in &self.map {
            if let Some(bitmap_b) = other.map.get(high_128) {
                let intersected_bitmap = bitmap_a & bitmap_b;
                if !intersected_bitmap.is_empty() {
                    result.insert(*high_128, intersected_bitmap);
                }
            }
        }
        Self { map: result }
    }

    pub fn iter(&self) -> impl Iterator<Item = FlexIdRank> + '_ {
        self.map.iter().flat_map(|(&high_128, bitmap)| {
            bitmap
                .iter()
                .map(move |low_32| FlexIdRank::from_parts(high_128, low_32))
        })
    }
}

// =====================================================================
// FlexIdRankListMultiOps & Operator Overloads
// =====================================================================

pub trait FlexIdRankListMultiOps {
    fn union(self) -> FlexIdRankList;
    fn intersection(self) -> FlexIdRankList;
    fn difference(self) -> FlexIdRankList;
    fn symmetric_difference(self) -> FlexIdRankList;
}

impl<'a, I> FlexIdRankListMultiOps for I
where
    I: IntoIterator<Item = &'a FlexIdRankList>,
{
    fn union(self) -> FlexIdRankList {
        let mut grouped: BTreeMap<u128, Vec<&RoaringBitmap>> = BTreeMap::new();
        for list in self {
            for (high_128, bitmap) in &list.map {
                grouped.entry(*high_128).or_default().push(bitmap);
            }
        }

        let mut final_map = BTreeMap::new();
        for (high_128, bitmaps) in grouped {
            let union_bitmap = bitmaps.into_iter().union();
            if !union_bitmap.is_empty() {
                final_map.insert(high_128, union_bitmap);
            }
        }
        FlexIdRankList { map: final_map }
    }

    fn intersection(self) -> FlexIdRankList {
        let mut iter = self.into_iter();
        let Some(first) = iter.next() else {
            return FlexIdRankList::new();
        };
        let mut current = first.clone();

        for next_list in iter {
            current.map.retain(|high_128, current_bitmap| {
                if let Some(next_bitmap) = next_list.map.get(high_128) {
                    *current_bitmap &= next_bitmap;
                    !current_bitmap.is_empty()
                } else {
                    false
                }
            });
            if current.map.is_empty() {
                break;
            }
        }
        current
    }

    fn difference(self) -> FlexIdRankList {
        let mut iter = self.into_iter();
        let Some(first) = iter.next() else {
            return FlexIdRankList::new();
        };
        let mut current = first.clone();

        for next_list in iter {
            current.map.retain(|high_128, current_bitmap| {
                if let Some(next_bitmap) = next_list.map.get(high_128) {
                    *current_bitmap -= next_bitmap;
                    !current_bitmap.is_empty()
                } else {
                    true
                }
            });
            if current.map.is_empty() {
                break;
            }
        }
        current
    }

    fn symmetric_difference(self) -> FlexIdRankList {
        let mut result = FlexIdRankList::new();
        for list in self {
            for (high_128, bitmap) in &list.map {
                let entry = result
                    .map
                    .entry(*high_128)
                    .or_insert_with(RoaringBitmap::new);
                *entry ^= bitmap;
            }
        }
        result.map.retain(|_, bitmap| !bitmap.is_empty());
        result
    }
}

impl BitOr for &FlexIdRankList {
    type Output = FlexIdRankList;
    fn bitor(self, rhs: Self) -> Self::Output {
        [self, rhs].union()
    }
}

impl BitAnd for &FlexIdRankList {
    type Output = FlexIdRankList;
    fn bitand(self, rhs: Self) -> Self::Output {
        [self, rhs].intersection()
    }
}

impl Sub for &FlexIdRankList {
    type Output = FlexIdRankList;
    fn sub(self, rhs: Self) -> Self::Output {
        [self, rhs].difference()
    }
}

impl BitXor for &FlexIdRankList {
    type Output = FlexIdRankList;
    fn bitxor(self, rhs: Self) -> Self::Output {
        [self, rhs].symmetric_difference()
    }
}

impl BitOrAssign<&FlexIdRankList> for FlexIdRankList {
    fn bitor_assign(&mut self, rhs: &FlexIdRankList) {
        for (high_128, rhs_bitmap) in &rhs.map {
            let entry = self.map.entry(*high_128).or_insert_with(RoaringBitmap::new);
            *entry |= rhs_bitmap;
        }
    }
}

impl BitAndAssign<&FlexIdRankList> for FlexIdRankList {
    fn bitand_assign(&mut self, rhs: &FlexIdRankList) {
        self.map.retain(|high_128, current_bitmap| {
            if let Some(rhs_bitmap) = rhs.map.get(high_128) {
                *current_bitmap &= rhs_bitmap;
                !current_bitmap.is_empty()
            } else {
                false
            }
        });
    }
}

impl SubAssign<&FlexIdRankList> for FlexIdRankList {
    fn sub_assign(&mut self, rhs: &FlexIdRankList) {
        self.map.retain(|high_128, current_bitmap| {
            if let Some(rhs_bitmap) = rhs.map.get(high_128) {
                *current_bitmap -= rhs_bitmap;
                !current_bitmap.is_empty()
            } else {
                true
            }
        });
    }
}

impl BitXorAssign<&FlexIdRankList> for FlexIdRankList {
    fn bitxor_assign(&mut self, rhs: &FlexIdRankList) {
        for (high_128, rhs_bitmap) in &rhs.map {
            let entry = self.map.entry(*high_128).or_insert_with(RoaringBitmap::new);
            *entry ^= rhs_bitmap;
        }
        self.map.retain(|_, bitmap| !bitmap.is_empty());
    }
}
