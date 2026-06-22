#[cfg(not(feature = "rayon"))]
pub(crate) type SharedNode<T> = alloc::rc::Rc<T>;

#[cfg(feature = "rayon")]
pub(crate) type SharedNode<T> = alloc::sync::Arc<T>;

#[cfg(not(feature = "rayon"))]
pub trait SafeValue: PartialEq + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: PartialEq + Clone> SafeValue for T {}

#[cfg(feature = "rayon")]
pub trait SafeValue: PartialEq + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: PartialEq + Clone + Send + Sync> SafeValue for T {}
