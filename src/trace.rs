macro_rules! trace_span {
    ($($arg:tt)*) => {
        #[cfg(feature = "unstable")]
        let _kasane_logic_span = ::tracing::debug_span!($($arg)*).entered();
    };
}
pub(crate) use trace_span;
