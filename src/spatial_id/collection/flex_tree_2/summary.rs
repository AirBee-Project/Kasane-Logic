use core::fmt;
use core::marker::PhantomData;

/// `Branch` が子孫の値についてキャッシュする情報。
pub trait Summary<V>: Clone + PartialEq {
    /// 値1つだけの[Summary]を作成する。
    fn new(value: &V) -> Self;

    /// 2つの[Summary]を合わせ、両方の値を満たす[Summary]を作成する。
    fn merge(&self, other: &Self) -> Self;
}

/// [Summary]に何もキャッシュしない場合の型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoSummary;

impl<V> Summary<V> for NoSummary {
    fn new(_: &V) -> Self {
        NoSummary
    }

    fn merge(&self, _: &Self) -> Self {
        NoSummary
    }
}

/// 値の最小値と最大値 `[min, max]` を持つ [Summary]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinMax<V> {
    min: V,
    max: V,
}

impl<V> MinMax<V> {
    pub fn min(&self) -> &V {
        &self.min
    }

    pub fn max(&self) -> &V {
        &self.max
    }
}

impl<V: Ord + Clone> Summary<V> for MinMax<V> {
    fn new(value: &V) -> Self {
        MinMax {
            min: value.clone(),
            max: value.clone(),
        }
    }

    fn merge(&self, other: &Self) -> Self {
        // 参照のまま比べて、選ばれた側だけをクローンする
        MinMax {
            min: (&self.min).min(&other.min).clone(),
            max: (&self.max).max(&other.max).clone(),
        }
    }
}

impl<V> AsRef<MinMax<V>> for MinMax<V> {
    fn as_ref(&self) -> &MinMax<V> {
        self
    }
}

/// 取りうる値が 64 通り以下の型。値を `0..COUNT` の番号へ対応させ、[ValueSet] のビットにする。
///
/// フィールドを持たない enum には `#[derive(BitMask)]` で実装できる。
/// バリアントの番号は定義順に `0, 1, 2, …` となり、判別値（`A = 5`）には左右されない。
///
/// ```
/// use kasane_logic::spatial_id::collection::flex_tree_2::BitMask;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, BitMask)]
/// enum Color {
///     Red,
///     Green = 10,
///     Blue,
/// }
///
/// assert_eq!(Color::COUNT, 3);
/// assert_eq!(Color::Green.index(), 1);
/// ```
pub trait BitMask: Copy {
    /// 取りうる値の数。64 以下。
    const COUNT: u32;

    /// 値の番号。`0..COUNT` の範囲。
    fn index(self) -> u32;
}

impl BitMask for bool {
    const COUNT: u32 = 2;

    fn index(self) -> u32 {
        self as u32
    }
}

impl<E: BitMask> BitMask for Option<E> {
    const COUNT: u32 = E::COUNT + 1;

    fn index(self) -> u32 {
        match self {
            None => 0,
            Some(e) => e.index() + 1,
        }
    }
}

/// 子孫に現れる値の集合を、[BitMask::index] 番目のビットで表したもの。
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ValueSet<V> {
    bits: u64,
    _value: PhantomData<fn() -> V>,
}

impl<V> ValueSet<V> {
    /// 空の集合。
    pub const EMPTY: Self = ValueSet::from_bits(0);

    const fn from_bits(bits: u64) -> Self {
        ValueSet {
            bits,
            _value: PhantomData,
        }
    }

    /// 生のビット列。
    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// 和集合。
    pub fn union(&self, other: &Self) -> Self {
        ValueSet::from_bits(self.bits | other.bits)
    }

    /// 共通の値を持たないなら true。
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.bits & other.bits == 0
    }

    /// 自身の値がすべて `other` にも含まれるなら true。
    pub fn is_subset(&self, other: &Self) -> bool {
        self.bits & !other.bits == 0
    }
}

impl<V: BitMask> ValueSet<V> {
    /// `V` の値が 64 通りを超えるならコンパイルエラーにする。
    const FITS_IN_U64: () = assert!(V::COUNT <= 64, "BitMask::COUNT は 64 以下にすること");

    /// 値1つだけの集合。
    pub fn single(value: V) -> Self {
        let () = Self::FITS_IN_U64;
        ValueSet::from_bits(1 << value.index())
    }

    /// `value` を含むなら true。
    pub fn contains(&self, value: V) -> bool {
        !self.is_disjoint(&ValueSet::single(value))
    }
}

impl<V: BitMask> FromIterator<V> for ValueSet<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        iter.into_iter()
            .fold(ValueSet::EMPTY, |set, v| set.union(&ValueSet::single(v)))
    }
}

impl<V: BitMask + PartialEq> Summary<V> for ValueSet<V> {
    fn new(value: &V) -> Self {
        ValueSet::single(*value)
    }

    fn merge(&self, other: &Self) -> Self {
        self.union(other)
    }
}

impl<V> AsRef<ValueSet<V>> for ValueSet<V> {
    fn as_ref(&self) -> &ValueSet<V> {
        self
    }
}

impl<V> fmt::Debug for ValueSet<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueSet({:#b})", self.bits)
    }
}

impl<V, A: Summary<V>, B: Summary<V>> Summary<V> for (A, B) {
    fn new(value: &V) -> Self {
        (A::new(value), B::new(value))
    }

    fn merge(&self, other: &Self) -> Self {
        (self.0.merge(&other.0), self.1.merge(&other.1))
    }
}

impl<V, B> AsRef<MinMax<V>> for (MinMax<V>, B) {
    fn as_ref(&self) -> &MinMax<V> {
        &self.0
    }
}

impl<V, A> AsRef<ValueSet<V>> for (A, ValueSet<V>) {
    fn as_ref(&self) -> &ValueSet<V> {
        &self.1
    }
}
