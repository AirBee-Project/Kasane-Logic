use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
enum Inner {
    /// 割り当てなしの、決してキャンセルされない状態。
    Never,
    Shared(Arc<AtomicBool>),
}

/// クエリ実行への協調的キャンセルを伝えるトークン。複製は状態を共有する。
#[derive(Clone, Debug)]
pub struct CancellationToken(Inner);

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        Self(Inner::Shared(Arc::new(AtomicBool::new(false))))
    }

    /// 決してキャンセルされないトークン。`cancel()` は無視される。
    pub fn never() -> Self {
        Self(Inner::Never)
    }

    pub fn cancel(&self) {
        if let Inner::Shared(flag) = &self.0 {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_cancelled(&self) -> bool {
        match &self.0 {
            Inner::Never => false,
            Inner::Shared(flag) => flag.load(Ordering::Relaxed),
        }
    }
}
