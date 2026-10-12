//! Query の値や演算子に課す制約。
//!
//! `rayon` 有効時は二項演算の左右を並列に実行するので `Send + Sync` が要る。
//! 無効時は並列にしないので要求しない。

#[cfg(not(feature = "rayon"))]
pub trait SafeValue: PartialEq + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: PartialEq + Clone> SafeValue for T {}

#[cfg(feature = "rayon")]
pub trait SafeValue: PartialEq + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: PartialEq + Clone + Send + Sync> SafeValue for T {}

#[cfg(not(feature = "rayon"))]
pub trait MaybeSendSync {}
#[cfg(not(feature = "rayon"))]
impl<T: ?Sized> MaybeSendSync for T {}

#[cfg(feature = "rayon")]
pub trait MaybeSendSync: Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: ?Sized + Send + Sync> MaybeSendSync for T {}
