use crate::spatial_id::collection::traits::SpatialIdSet;
use crate::{SpatialIds, VBitSet};
use std::mem;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

// =====================================================================
// [1] 両方消費パターン (Self OP Self) - 演算のコアロジック
// =====================================================================

impl BitOr for VBitSet {
    type Output = Self;
    fn bitor(mut self, mut rhs: Self) -> Self::Output {
        if self.size() < rhs.size() {
            for flex_id in self.flex_ids() {
                rhs.insert(flex_id.clone());
            }
            rhs
        } else {
            for flex_id in rhs.flex_ids() {
                self.insert(flex_id.clone());
            }
            self
        }
    }
}

impl BitAnd for VBitSet {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut result = Self::default();
        let (smaller, larger) = if self.size() <= rhs.size() {
            (&self, &rhs)
        } else {
            (&rhs, &self)
        };

        for flex_id in smaller.flex_ids() {
            let intersection_set = larger.get(flex_id.clone());
            for intersect_id in intersection_set.flex_ids() {
                result.insert(intersect_id.clone());
            }
        }
        result
    }
}

impl Sub for VBitSet {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        if rhs.is_empty() || self.is_empty() {
            return self;
        }
        for flex_id in rhs.flex_ids() {
            let _ = self.remove(flex_id.clone());
        }
        self
    }
}

impl BitXor for VBitSet {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        // XOR (対称差集合) = (A ∪ B) - (A ∩ B)
        let intersection = self.clone() & rhs.clone();
        let union_set = self | rhs;
        union_set - intersection
    }
}

// =====================================================================
// [2] & [4] 借用パターン (&A OP B, A OP &B, &A OP &B) 自動生成マクロ
// =====================================================================

macro_rules! impl_op_variants {
    ($trait:ident, $method:ident) => {
        // パターン: A OP &B
        impl $trait<&VBitSet> for VBitSet {
            type Output = VBitSet;
            #[inline]
            fn $method(self, rhs: &VBitSet) -> Self::Output {
                $trait::$method(self, rhs.clone())
            }
        }

        // パターン: &A OP B
        impl $trait<VBitSet> for &VBitSet {
            type Output = VBitSet;
            #[inline]
            fn $method(self, rhs: VBitSet) -> Self::Output {
                $trait::$method(self.clone(), rhs)
            }
        }

        // パターン: &A OP &B (トレイトの where 句で要求されるもの)
        impl $trait<&VBitSet> for &VBitSet {
            type Output = VBitSet;
            #[inline]
            fn $method(self, rhs: &VBitSet) -> Self::Output {
                $trait::$method(self.clone(), rhs.clone())
            }
        }
    };
}

impl_op_variants!(BitOr, bitor);
impl_op_variants!(BitAnd, bitand);
impl_op_variants!(Sub, sub);
impl_op_variants!(BitXor, bitxor);

// =====================================================================
// [3] 破壊的代入パターン (Self OP= Self / &Self)
// =====================================================================

// --- BitOrAssign (|=) ---
impl BitOrAssign<VBitSet> for VBitSet {
    #[inline]
    fn bitor_assign(&mut self, mut rhs: VBitSet) {
        if self.size() < rhs.size() {
            for flex_id in self.flex_ids() {
                rhs.insert(flex_id.clone());
            }
            // selfの中身を巨大なrhsとすり替える (ポインタの付け替えのみで高速)
            *self = rhs;
        } else {
            for flex_id in rhs.flex_ids() {
                self.insert(flex_id.clone());
            }
        }
    }
}
impl BitOrAssign<&VBitSet> for VBitSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: &VBitSet) {
        for flex_id in rhs.flex_ids() {
            self.insert(flex_id.clone());
        }
    }
}

// --- SubAssign (-=) ---
impl SubAssign<VBitSet> for VBitSet {
    #[inline]
    fn sub_assign(&mut self, rhs: VBitSet) {
        *self -= &rhs;
    }
}
impl SubAssign<&VBitSet> for VBitSet {
    #[inline]
    fn sub_assign(&mut self, rhs: &VBitSet) {
        if rhs.is_empty() || self.is_empty() {
            return;
        }
        for flex_id in rhs.flex_ids() {
            let _ = self.remove(flex_id.clone());
        }
    }
}

// --- BitAndAssign (&=) ---
impl BitAndAssign<VBitSet> for VBitSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: VBitSet) {
        *self = mem::take(self) & rhs;
    }
}
impl BitAndAssign<&VBitSet> for VBitSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: &VBitSet) {
        *self = mem::take(self) & rhs.clone();
    }
}

// --- BitXorAssign (^=) ---
impl BitXorAssign<VBitSet> for VBitSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: VBitSet) {
        *self = mem::take(self) ^ rhs;
    }
}
impl BitXorAssign<&VBitSet> for VBitSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &VBitSet) {
        *self = mem::take(self) ^ rhs.clone();
    }
}

impl PartialEq for VBitSet {
    fn eq(&self, other: &Self) -> bool {
        (self.clone() - other.clone()).is_empty()
    }
}

impl Eq for VBitSet {}
