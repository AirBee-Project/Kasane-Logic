use std::ops::{BitAnd, BitOr, Sub};

use crate::SpatialIdSet;

impl BitOr<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitor(self, rhs: &SpatialIdSet) -> Self::Output {
        let (first, second) = if self.count() >= rhs.count() {
            (self, rhs)
        } else {
            (rhs, self)
        };

        let mut output = first.clone();

        for flex_id in second.iter() {
            output.insert(flex_id);
        }

        output
    }
}

impl BitAnd<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitand(self, rhs: &SpatialIdSet) -> Self::Output {
        let (smaller, larger) = if self.count() <= rhs.count() {
            (self, rhs)
        } else {
            (rhs, self)
        };

        let mut output = SpatialIdSet::new();

        for flex_id in smaller.iter() {
            for intersect_id in larger.get(&flex_id) {
                output.insert(intersect_id);
            }
        }

        output
    }
}

impl Sub<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn sub(self, rhs: &SpatialIdSet) -> Self::Output {
        if self.is_empty() {
            return SpatialIdSet::new();
        }

        if rhs.is_empty() {
            return self.clone();
        }

        let mut output = self.clone();

        if rhs.count() <= self.count() {
            for rhs_id in rhs.iter() {
                let _ = output.remove(&rhs_id);
            }
        } else {
            let intersection = self & rhs;
            for inter_id in intersection.iter() {
                let _ = output.remove(&inter_id);
            }
        }

        output
    }
}

impl BitOr for SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        for flex_id in rhs.iter() {
            self.insert(flex_id.clone());
        }
        self
    }
}

impl BitAnd for SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

impl Sub for SpatialIdSet {
    type Output = SpatialIdSet;

    fn sub(mut self, rhs: Self) -> Self::Output {
        if rhs.count() <= self.count() {
            for rhs_id in rhs.iter() {
                let _ = self.remove(&rhs_id);
            }
        } else {
            let intersection = &self & &rhs;
            for inter_id in intersection.iter() {
                let _ = self.remove(&inter_id);
            }
        }
        self
    }
}
