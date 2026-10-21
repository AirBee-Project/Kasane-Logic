//! Query の値や演算子に課す制約。
//!
//! `rayon` 有効時は二項演算の左右を並列に実行するので `Send + Sync` が要る。
//! 無効時は並列にしないので要求しない。

/// [`Query`](crate::query::Query) が扱える値。`rayon` 有効時は `Send + Sync` も要る。条件を満たす型には自動で実装される。
#[cfg(not(feature = "rayon"))]
pub trait SafeValue: PartialEq + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: PartialEq + Clone> SafeValue for T {}

/// [`Query`](crate::query::Query) が扱える値。`rayon` 有効時は `Send + Sync` も要る。条件を満たす型には自動で実装される。
#[cfg(feature = "rayon")]
pub trait SafeValue: PartialEq + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: PartialEq + Clone + Send + Sync> SafeValue for T {}

/// `rayon` 有効時だけ `Send + Sync` を要求する制約。条件を満たす型には自動で実装される。
#[cfg(not(feature = "rayon"))]
pub trait MaybeSendSync {}
#[cfg(not(feature = "rayon"))]
impl<T: ?Sized> MaybeSendSync for T {}

/// `rayon` 有効時だけ `Send + Sync` を要求する制約。条件を満たす型には自動で実装される。
#[cfg(feature = "rayon")]
pub trait MaybeSendSync: Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: ?Sized + Send + Sync> MaybeSendSync for T {}

/// `rayon` 有効時だけ `Send` を要求する制約。条件を満たす型には自動で実装される。
#[cfg(not(feature = "rayon"))]
pub trait MaybeSend {}
#[cfg(not(feature = "rayon"))]
impl<T: ?Sized> MaybeSend for T {}

/// `rayon` 有効時だけ `Send` を要求する制約。条件を満たす型には自動で実装される。
#[cfg(feature = "rayon")]
pub trait MaybeSend: Send {}
#[cfg(feature = "rayon")]
impl<T: ?Sized + Send> MaybeSend for T {}
