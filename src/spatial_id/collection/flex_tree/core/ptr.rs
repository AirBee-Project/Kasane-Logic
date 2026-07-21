#[cfg(not(feature = "rayon"))]
pub(crate) type SharedNode<T> = alloc::rc::Rc<T>;

#[cfg(not(feature = "rayon"))]
pub(crate) fn try_unwrap<T>(shared: SharedNode<T>) -> Result<T, SharedNode<T>> {
    alloc::rc::Rc::try_unwrap(shared)
}

#[cfg(feature = "rayon")]
pub(crate) type SharedNode<T> = alloc::sync::Arc<T>;

#[cfg(feature = "rayon")]
pub(crate) fn try_unwrap<T>(shared: SharedNode<T>) -> Result<T, SharedNode<T>> {
    alloc::sync::Arc::try_unwrap(shared)
}

#[cfg(not(feature = "rayon"))]
pub trait SafeValue: PartialEq + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: PartialEq + Clone> SafeValue for T {}

#[cfg(feature = "rayon")]
pub trait SafeValue: PartialEq + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: PartialEq + Clone + Send + Sync> SafeValue for T {}

#[cfg(not(feature = "rayon"))]
pub trait MaybeSend {}
#[cfg(not(feature = "rayon"))]
impl<T: ?Sized> MaybeSend for T {}

#[cfg(feature = "rayon")]
pub trait MaybeSend: Send {}
#[cfg(feature = "rayon")]
impl<T: ?Sized + Send> MaybeSend for T {}

#[cfg(not(feature = "rayon"))]
pub trait MaybeSendSync {}
#[cfg(not(feature = "rayon"))]
impl<T: ?Sized> MaybeSendSync for T {}

#[cfg(feature = "rayon")]
pub trait MaybeSendSync: Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: ?Sized + Send + Sync> MaybeSendSync for T {}

#[cfg(not(feature = "rayon"))]
pub trait MaybeSync {}
#[cfg(not(feature = "rayon"))]
impl<T: ?Sized> MaybeSync for T {}

#[cfg(feature = "rayon")]
pub trait MaybeSync: Sync {}
#[cfg(feature = "rayon")]
impl<T: ?Sized + Sync> MaybeSync for T {}
