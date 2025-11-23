use crate::{
    encode_id::EncodeID,
    space_time_id::SpaceTimeID,
    space_time_id_set::{ReverseInfo, SpaceTimeIDSet},
};

pub struct SpaceTimeIDSetIter<'a> {
    reverse_iter: std::collections::hash_map::Iter<'a, usize, ReverseInfo>,
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

        let encode_id = EncodeID {
            f: reverse.f.clone(),
            x: reverse.x.clone(),
            y: reverse.y.clone(),
        };

        Some(encode_id.decode())
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
