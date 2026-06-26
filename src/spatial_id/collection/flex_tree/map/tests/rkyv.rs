/// DB 用途（値＝バイト列）の永続化。ジェネリック境界を避けるため `Vec<u8>` 固定で提供する。
#[cfg(feature = "persist")]
impl SpatialIdMap<Vec<u8>> {
    /// この [`SpatialIdMap`] を rkyv バイト列へ直列化する。
    pub fn to_bytes(&self) -> Result<Vec<u8>, rkyv::rancor::Error> {
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から復元する。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedSpatialIdMap<Vec<u8>>>(bytes) };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived)
    }
}

#[cfg(all(test, feature = "persist"))]
mod persist_tests {
    use super::SpatialIdMap;
    use crate::{RangeId, SingleId};
    use alloc::vec::Vec;

    fn sorted(map: &SpatialIdMap<Vec<u8>>) -> Vec<(crate::FlexId, Vec<u8>)> {
        let mut v: Vec<_> = map.iter().map(|(f, val)| (f, val.clone())).collect();
        v.sort();
        v
    }

    #[test]
    fn round_trip() {
        let mut map = SpatialIdMap::<Vec<u8>>::new();
        map.insert(SingleId::new(20, 0, 0, 0).unwrap(), b"alpha".to_vec());
        map.insert(SingleId::new(20, 0, 2, 3).unwrap(), b"alpha".to_vec());
        map.insert(SingleId::new(18, 1, 5, 7).unwrap(), b"beta".to_vec());
        map.insert(
            RangeId::new(5, [1, 4], [8, 9], [5, 6]).unwrap(),
            b"gamma".to_vec(),
        );

        let bytes = map.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };

        assert_eq!(sorted(&map), sorted(&restored));
        assert_eq!(map.count(), restored.count());
    }

    #[test]
    fn round_trip_empty() {
        let map = SpatialIdMap::<Vec<u8>>::new();
        let bytes = map.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };
        assert!(restored.is_empty());
    }
}
