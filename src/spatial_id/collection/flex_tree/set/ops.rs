use std::ops::{BitAnd, BitOr, Sub};

use crate::FlexTreeSet;

impl BitOr<&FlexTreeSet> for &FlexTreeSet {
    type Output = FlexTreeSet;

    fn bitor(self, rhs: &FlexTreeSet) -> Self::Output {
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

impl BitAnd<&FlexTreeSet> for &FlexTreeSet {
    type Output = FlexTreeSet;

    fn bitand(self, rhs: &FlexTreeSet) -> Self::Output {
        let (smaller, larger) = if self.count() <= rhs.count() {
            (self, rhs)
        } else {
            (rhs, self)
        };

        let mut output = FlexTreeSet::new();

        for flex_id in smaller.iter() {
            for intersect_id in larger.get(&flex_id) {
                output.insert(intersect_id);
            }
        }

        output
    }
}

impl Sub<&FlexTreeSet> for &FlexTreeSet {
    type Output = FlexTreeSet;

    fn sub(self, rhs: &FlexTreeSet) -> Self::Output {
        if self.is_empty() {
            return FlexTreeSet::new();
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

impl BitOr for FlexTreeSet {
    type Output = FlexTreeSet;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        for flex_id in rhs.iter() {
            self.insert(flex_id.clone());
        }
        self
    }
}

impl BitAnd for FlexTreeSet {
    type Output = FlexTreeSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

impl Sub for FlexTreeSet {
    type Output = FlexTreeSet;

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
