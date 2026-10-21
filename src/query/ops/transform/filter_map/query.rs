use super::FilterMap;
use crate::FlexId;
use crate::query::Query;
use crate::query::send_sync::{MaybeSendSync, SafeValue};
use alloc::boxed::Box;

impl<V: SafeValue + 'static> Query<V> {
    /// 各要素の [`FlexId`] と値を `f` に渡し、`Some` なら値を写し、`None` なら取り除く。
    /// [`FlexId`] は変えられない。値は別の型へ写せる。
    ///
    /// `f` に渡る [`FlexId`] は、同じ値を持つ領域の断片である。切れ方は木の状態や
    /// [`get`](Self::get) に渡す範囲によって変わるので、`f` の結果が切れ方に左右されないようにする。
    /// 値だけを見るなら [`filter_map_values`](Self::filter_map_values) を使う。
    pub fn filter_map<U, F>(self, f: F) -> Query<U>
    where
        U: SafeValue + 'static,
        F: Fn(FlexId, V) -> Option<U> + MaybeSendSync + 'static,
    {
        Query::Source(Box::new(FilterMap {
            inner: self,
            f,
            _marker: core::marker::PhantomData,
        }))
    }

    /// 各要素の [`FlexId`] と値を `f` で写す。[`FlexId`] の扱いは [`filter_map`](Self::filter_map) と同じ。
    pub fn map<U, F>(self, f: F) -> Query<U>
    where
        U: SafeValue + 'static,
        F: Fn(FlexId, V) -> U + MaybeSendSync + 'static,
    {
        self.filter_map(move |id, value| Some(f(id, value)))
    }

    /// 各要素の [`FlexId`] と値を `f` に渡し、`true` を返した要素だけを残す。
    /// [`FlexId`] の扱いは [`filter_map`](Self::filter_map) と同じ。
    pub fn filter<F>(self, f: F) -> Self
    where
        F: Fn(FlexId, &V) -> bool + MaybeSendSync + 'static,
    {
        self.filter_map(move |id, value| f(id, &value).then_some(value))
    }
}
