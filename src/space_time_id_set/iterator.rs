use crate::{encode_id::EncodeID, space_time_id::SpaceTimeID, space_time_id_set::SpaceTimeIDSet};

pub struct SpaceTimeIDSetIter<'a> {
    reverse_iter: std::collections::hash_map::Iter<'a, usize, EncodeID>,
}

impl SpaceTimeIDSet {
    pub fn iter(&'_ self) -> SpaceTimeIDSetIter<'_> {
        SpaceTimeIDSetIter {
            reverse_iter: self.reverse.iter(),
        }
    }
}

impl<'a> Iterator for SpaceTimeIDSetIter<'a> {
    type Item = SpaceTimeID;

    fn next(&mut self) -> Option<Self::Item> {
        let (_index, reverse) = self.reverse_iter.next()?; // <-- ここが(usize, ReverseInfo)

        Some(reverse.decode())
    }
}

impl<'a> IntoIterator for &'a SpaceTimeIDSet {
    type Item = SpaceTimeID;
    type IntoIter = SpaceTimeIDSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> ExactSizeIterator for SpaceTimeIDSetIter<'a> {
    fn len(&self) -> usize {
        self.reverse_iter.len()
    }
}
