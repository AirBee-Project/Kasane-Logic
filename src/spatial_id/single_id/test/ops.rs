#[cfg(test)]
mod tests {
    use crate::SingleId;

    #[test]
    fn intersection_returns_the_deeper_id_for_ancestor_descendant_pairs() {
        let ancestor = SingleId::new(20, 0, 931078, 413136).unwrap();
        let descendant = SingleId::new(23, 0, 7448630, 3305088).unwrap();

        assert_eq!(ancestor.intersection(&descendant).unwrap(), descendant);
        assert_eq!(descendant.intersection(&ancestor).unwrap(), descendant);
    }

    #[test]
    fn intersection_returns_none_for_disjoint_ids() {
        let left = SingleId::new(20, 0, 931078, 413136).unwrap();
        let right = SingleId::new(20, 1, 931078, 413136).unwrap();

        assert!(left.intersection(&right).is_none());
    }

    #[test]
    fn difference_returns_self_when_disjoint() {
        let left = SingleId::new(20, 0, 931078, 413136).unwrap();
        let right = SingleId::new(20, 1, 931078, 413136).unwrap();

        let diff: Vec<_> = left.difference(&right).collect();

        assert_eq!(diff, vec![left]);
    }

    #[test]
    fn difference_returns_empty_when_ids_match() {
        let id = SingleId::new(20, 0, 931078, 413136).unwrap();

        #[allow(clippy::needless_collect)]
        let diff: Vec<_> = id.difference(&id).collect();

        assert!(diff.is_empty());
    }

    #[test]
    fn difference_splits_into_remaining_siblings_for_descendant_overlap() {
        let parent = SingleId::new(20, 0, 931078, 413136).unwrap();
        let child = SingleId::new(23, 0, 7448630, 3305088).unwrap();

        let diff: Vec<_> = parent.difference(&child).collect();

        assert_eq!(diff.len(), 21);
        assert!(!diff.contains(&child));
        assert!(diff.iter().all(|id| id.intersection(&child).is_none()));
    }
}
