#[cfg(feature = "persist")]
use super::{ArchivedSpatialIdTable, SpatialIdTable};

#[cfg(feature = "persist")]
impl<V> SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue
        + Ord
        + rkyv::Archive
        + 'static,
    <V as rkyv::Archive>::Archived: Ord,
    for<'a> SpatialIdTable<V>: rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    ArchivedSpatialIdTable<V>: rkyv::Deserialize<SpatialIdTable<V>, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
{
    /// この [`SpatialIdTable`] を rkyv バイト列へ直列化する。
    pub fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, rkyv::rancor::Error> {
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(self)?.to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から復元する。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdTable::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedSpatialIdTable<V>>(bytes) };
        rkyv::deserialize::<Self, rkyv::rancor::Error>(archived)
    }
}
