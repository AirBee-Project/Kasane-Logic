#![allow(dead_code)]

use super::node::{Branch, Skip};
use core::fmt;
use core::marker::PhantomData;
use triomphe::Arc;

/// Tagged Pointer を用いた 8 バイトのノード表現。
///
/// ヒープポインタのアライメント（8バイト境界、下位3ビットが0）を利用し、
/// 下位2ビットをタグとして識別することで、`Node` のサイズを 16 バイトから 8 バイトへ半減させる。
#[repr(transparent)]
pub(crate) struct CompactNode<V, S> {
    raw: usize,
    _marker: PhantomData<(V, S)>,
}

const TAG_MASK: usize = 0b11;
const TAG_EMPTY: usize = 0b00;
const TAG_BRANCH: usize = 0b01;
const TAG_SKIP: usize = 0b10;
const TAG_LEAF: usize = 0b11;

pub(crate) enum CompactNodeRef<'a, V, S> {
    Empty,
    Leaf(u32),
    Branch(&'a Arc<Branch<V, S>>),
    Skip(&'a Arc<Skip<V, S>>),
}

impl<V, S> Default for CompactNode<V, S> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<V, S> CompactNode<V, S> {
    pub const fn empty() -> Self {
        Self {
            raw: TAG_EMPTY,
            _marker: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        (self.raw & TAG_MASK) == TAG_EMPTY
    }

    pub fn from_branch(branch: Arc<Branch<V, S>>) -> Self {
        let ptr = Arc::into_raw(branch) as usize;
        debug_assert_eq!(ptr & TAG_MASK, 0, "Arc pointer must be 8-byte aligned");
        Self {
            raw: ptr | TAG_BRANCH,
            _marker: PhantomData,
        }
    }

    pub fn from_skip(skip: Arc<Skip<V, S>>) -> Self {
        let ptr = Arc::into_raw(skip) as usize;
        debug_assert_eq!(ptr & TAG_MASK, 0, "Arc pointer must be 8-byte aligned");
        Self {
            raw: ptr | TAG_SKIP,
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> CompactNodeRef<'_, V, S> {
        match self.raw & TAG_MASK {
            TAG_EMPTY => CompactNodeRef::Empty,
            TAG_BRANCH => {
                let ptr = (self.raw & !TAG_MASK) as *const Branch<V, S>;
                // SAFETY: raw に有効な Arc ポインタを保持している間は参照が有効
                CompactNodeRef::Branch(unsafe { &*(ptr as *const Arc<Branch<V, S>>) })
            }
            TAG_SKIP => {
                let ptr = (self.raw & !TAG_MASK) as *const Skip<V, S>;
                CompactNodeRef::Skip(unsafe { &*(ptr as *const Arc<Skip<V, S>>) })
            }
            TAG_LEAF => CompactNodeRef::Leaf((self.raw >> 32) as u32),
            _ => unreachable!(),
        }
    }
}

impl<S> CompactNode<u32, S> {
    pub fn from_leaf_u32(val: u32) -> Self {
        let raw = ((val as usize) << 32) | TAG_LEAF;
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub fn leaf_value_u32(&self) -> Option<u32> {
        if (self.raw & TAG_MASK) == TAG_LEAF {
            Some((self.raw >> 32) as u32)
        } else {
            None
        }
    }
}

impl<S> CompactNode<(), S> {
    pub fn from_leaf_unit() -> Self {
        Self {
            raw: TAG_LEAF,
            _marker: PhantomData,
        }
    }
}

impl<V, S> Drop for CompactNode<V, S> {
    fn drop(&mut self) {
        match self.raw & TAG_MASK {
            TAG_BRANCH => {
                let ptr = (self.raw & !TAG_MASK) as *const Branch<V, S>;
                unsafe {
                    drop(Arc::from_raw(ptr));
                }
            }
            TAG_SKIP => {
                let ptr = (self.raw & !TAG_MASK) as *const Skip<V, S>;
                unsafe {
                    drop(Arc::from_raw(ptr));
                }
            }
            _ => {}
        }
    }
}

impl<V, S> Clone for CompactNode<V, S> {
    fn clone(&self) -> Self {
        match self.raw & TAG_MASK {
            TAG_BRANCH => {
                let ptr = (self.raw & !TAG_MASK) as *const Branch<V, S>;
                let arc = unsafe { Arc::from_raw(ptr) };
                let cloned = arc.clone();
                core::mem::forget(arc);
                let new_ptr = Arc::into_raw(cloned) as usize;
                Self {
                    raw: new_ptr | TAG_BRANCH,
                    _marker: PhantomData,
                }
            }
            TAG_SKIP => {
                let ptr = (self.raw & !TAG_MASK) as *const Skip<V, S>;
                let arc = unsafe { Arc::from_raw(ptr) };
                let cloned = arc.clone();
                core::mem::forget(arc);
                let new_ptr = Arc::into_raw(cloned) as usize;
                Self {
                    raw: new_ptr | TAG_SKIP,
                    _marker: PhantomData,
                }
            }
            _ => Self {
                raw: self.raw,
                _marker: PhantomData,
            },
        }
    }
}

impl<V, S> fmt::Debug for CompactNode<V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.raw & TAG_MASK {
            TAG_EMPTY => write!(f, "Empty"),
            TAG_BRANCH => write!(f, "Branch({:#x})", self.raw & !TAG_MASK),
            TAG_SKIP => write!(f, "Skip({:#x})", self.raw & !TAG_MASK),
            TAG_LEAF => write!(f, "Leaf({:#x})", self.raw >> 32),
            _ => unreachable!(),
        }
    }
}

impl<V, S> PartialEq for CompactNode<V, S> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<V, S> Eq for CompactNode<V, S> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::dimension::Dimension;
    use crate::id::relative_flex_id::RelativeFlexId;
    use core::mem::size_of;

    struct CompactBranch<V, S> {
        _dimension: Dimension,
        _split_dimensions: u8,
        _height: u8,
        _count: u32,
        _summary: S,
        _lower: CompactNode<V, S>,
        _upper: CompactNode<V, S>,
    }

    struct CompactSkip<V, S> {
        _path: RelativeFlexId,
        _split_dimensions: u8,
        _child: CompactNode<V, S>,
    }

    #[test]
    fn compact_node_is_8_bytes() {
        assert_eq!(size_of::<CompactNode<u32, ()>>(), 8);
        assert_eq!(size_of::<CompactNode<(), ()>>(), 8);
        assert_eq!(size_of::<CompactBranch<u32, ()>>(), 24);
        assert_eq!(size_of::<CompactBranch<(), ()>>(), 24);

        #[cfg(feature = "temporal_id")]
        {
            assert_eq!(size_of::<CompactSkip<u32, ()>>(), 40);
            assert_eq!(size_of::<CompactSkip<(), ()>>(), 40);
        }
        #[cfg(not(feature = "temporal_id"))]
        {
            assert_eq!(size_of::<CompactSkip<u32, ()>>(), 32);
            assert_eq!(size_of::<CompactSkip<(), ()>>(), 32);
        }
    }
}
