//! メソッドチェーンを「1つのデータ型」として表現する遅延プラン。
//!
//! [`Plan`] は演算チェーン全体を1個の値（再帰的 enum）として保持し、同時に
//! メソッドチェーンのビルダーでもある。各ノードが演算の種類と引数をそのまま持つため、
//! 構造をパターンマッチで走査・比較・書き換えできる。これは後段の等式証明
//! （`execution(lhs) == execution(rhs)` の検証や書き換え規則の健全性確認）の土台になる。
//!
//! # 演算子の拡張（Expression Problem への対処）
//! 既知の演算は [`UnaryOp`] / [`BinaryOp`] の**バリアント（データ）**として持ち、最適化・
//! 証明の対象にできる。一方、任意の新しい演算は enum を編集せずに `Custom` バリアント
//! （[`UnaryKernel`] / [`BinaryKernel`] のトレイトオブジェクト）として追加できる。
//! `Custom` は最適化・証明にとって不透明な「バリア」として扱われる。
//!
//! # カスタムパラメータの扱い
//! 各演算がトレイトで持つ関連型 [`UnaryOperator::CustomParameter`] /
//! [`BinaryOperator::CustomParameter`] は、enum 側では「そのバリアントのペイロード」に
//! 一対一で対応する。既存の `ShiftParam` / `StretchParam` 等をそのまま再利用する。
//!
//! # 総称性
//! [`Plan`] はコレクション型 `C: SpatialIdCollection` で総称化されており、
//! `SpatialIdTable` / `SpatialIdSet` のどちらでもチェーンを組める。入口は
//! [`SpatialIdCollection::plan`](crate::SpatialIdCollection::plan) の既定実装。

use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::{BinaryOperator, ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::binary::set::difference::Difference;
use super::binary::set::intersection::Intersection;
use super::binary::set::mask::Mask;
use super::binary::set::symmetric_difference::SymmetricDifference;
use super::binary::set::union::Union;
use super::unary::fill::FillDefault;
use super::unary::shift::ShiftParam;
use super::unary::shift::shift_f::FShift;
use super::unary::shift::shift_x::XShift;
use super::unary::shift::shift_y::YShift;
use super::unary::stretch::StretchParam;
use super::unary::stretch::stretch_f::FStretch;
use super::unary::stretch::stretch_x::XStretch;
use super::unary::stretch::stretch_y::YStretch;

/// enum を編集せずに追加できる単項演算の拡張点。
///
/// `Plan::apply` でチェーンに差し込む。最適化・証明にとっては不透明なバリアとなる。
pub trait UnaryKernel<C: SpatialIdCollection> {
    /// 入力コレクションへ演算を適用する。
    fn run(self: Box<Self>, input: &C) -> Result<C, Error>;

    /// 恒等変換なら `true`（最適化で畳んでよい）。既定は `false`。
    fn is_identity(&self) -> bool {
        false
    }
}

/// enum を編集せずに追加できる二項演算の拡張点。
///
/// `Plan::combine` でチェーンに差し込む。最適化・証明にとっては不透明なバリアとなる。
pub trait BinaryKernel<C: SpatialIdCollection> {
    /// 2つのコレクションへ演算を適用する。
    fn run(self: Box<Self>, lhs: &C, rhs: &C) -> Result<C, Error>;
}

/// 既存の [`UnaryOperator`] を、そのまま [`UnaryKernel`] として使うためのアダプタ。
///
/// 演算子型 `Op` とそのカスタムパラメータを束ね、`run` で `Op::execution` へ委譲する。
/// これにより「[`UnaryOperator`] を実装した型」は手書きの [`UnaryKernel`] 実装なしで
/// `Plan::apply_op` から lazy チェーンへ載せられる。
struct UnaryOpKernel<Op, P> {
    param: P,
    _op: PhantomData<fn() -> Op>,
}

impl<C, Op> UnaryKernel<C> for UnaryOpKernel<Op, <Op as UnaryOperator<C::Value>>::CustomParameter>
where
    C: SpatialIdCollection,
    Op: UnaryOperator<C::Value, ResultValue = C::Value>,
{
    fn run(self: Box<Self>, input: &C) -> Result<C, Error> {
        <Op as UnaryOperator<C::Value>>::execution::<C, C>(input, self.param)
    }
}

/// 既存の [`BinaryOperator`] を、そのまま [`BinaryKernel`] として使うためのアダプタ。
///
/// 「[`BinaryOperator`] を実装した型」は手書きの [`BinaryKernel`] 実装なしで
/// `Plan::combine_op` から lazy チェーンへ載せられる。
struct BinaryOpKernel<Op, P> {
    param: P,
    _op: PhantomData<fn() -> Op>,
}

impl<C, Op> BinaryKernel<C>
    for BinaryOpKernel<Op, <Op as BinaryOperator<C::Value, C::Value>>::CustomParameter>
where
    C: SpatialIdCollection,
    Op: BinaryOperator<C::Value, C::Value, ResultValue = C::Value>,
{
    fn run(self: Box<Self>, lhs: &C, rhs: &C) -> Result<C, Error> {
        <Op as BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, self.param)
    }
}

/// 単項演算を「値」として列挙したもの。
///
/// 既知のバリアントはペイロードがその演算のカスタムパラメータそのもの。`Custom` は
/// 任意の [`UnaryKernel`] を載せる拡張点。
pub enum UnaryOp<C: SpatialIdCollection> {
    /// `CustomParameter = ShiftParam`
    ShiftF(ShiftParam),
    ShiftX(ShiftParam),
    ShiftY(ShiftParam),
    /// `CustomParameter = StretchParam<C::Value>`
    StretchF(StretchParam<C::Value>),
    StretchX(StretchParam<C::Value>),
    StretchY(StretchParam<C::Value>),
    /// `CustomParameter = C::Value`（隙間へ割り当てる既定値）
    Fill(C::Value),
    /// 拡張点：enum を編集せずに足せる任意の単項演算。
    Custom(Box<dyn UnaryKernel<C>>),
}

/// 二項演算を「値」として列挙したもの。
pub enum BinaryOp<C: SpatialIdCollection> {
    /// `CustomParameter = ConflictPolicy<C::Value>`
    Union(ConflictPolicy<C::Value>),
    Intersection(ConflictPolicy<C::Value>),
    /// `CustomParameter = ()`
    Difference,
    SymmetricDifference,
    Mask,
    /// 拡張点：enum を編集せずに足せる任意の二項演算。
    Custom(Box<dyn BinaryKernel<C>>),
}

/// メソッドチェーン全体を表す単一のデータ型（再帰的 AST）。
///
/// 葉から結果まで、ひとつのコレクション型 `C` が一貫して流れる。
pub enum Plan<C: SpatialIdCollection> {
    /// 葉：演算の起点となるコレクション。
    Source(C),
    /// 単項演算ノード。
    Unary { op: UnaryOp<C>, input: Box<Plan<C>> },
    /// 二項演算ノード。
    Binary {
        op: BinaryOp<C>,
        lhs: Box<Plan<C>>,
        rhs: Box<Plan<C>>,
    },
}

impl<C: SpatialIdCollection> UnaryOp<C> {
    /// 恒等変換になる演算か（最適化で畳める）。`index == 0` の平行移動・引き延ばしが該当。
    fn is_identity(&self) -> bool {
        match self {
            UnaryOp::ShiftF(p) | UnaryOp::ShiftX(p) | UnaryOp::ShiftY(p) => p.index == 0,
            UnaryOp::StretchF(p) | UnaryOp::StretchX(p) | UnaryOp::StretchY(p) => p.index == 0,
            UnaryOp::Fill(_) => false,
            UnaryOp::Custom(kernel) => kernel.is_identity(),
        }
    }

    /// 既存カーネルへ委譲して単項演算を実行する。
    /// 既知のバリアントはペイロード（＝カスタムパラメータ）を `execution` へ素通しし、
    /// `Custom` はトレイトオブジェクトへ委譲する。
    fn run(self, input: &C) -> Result<C, Error> {
        match self {
            UnaryOp::ShiftF(param) => FShift::execution::<C, C>(input, param),
            UnaryOp::ShiftX(param) => XShift::execution::<C, C>(input, param),
            UnaryOp::ShiftY(param) => YShift::execution::<C, C>(input, param),
            UnaryOp::StretchF(param) => FStretch::execution::<C, C>(input, param),
            UnaryOp::StretchX(param) => XStretch::execution::<C, C>(input, param),
            UnaryOp::StretchY(param) => YStretch::execution::<C, C>(input, param),
            UnaryOp::Fill(default) => FillDefault::execution::<C, C>(input, default),
            UnaryOp::Custom(kernel) => kernel.run(input),
        }
    }
}

impl<C: SpatialIdCollection> BinaryOp<C> {
    /// 可換な演算か。最適化での評価順入れ替えの判断に使う。
    /// `Custom` は中身が不明なため保守的に非可換とみなす。
    pub fn is_commutative(&self) -> bool {
        match self {
            BinaryOp::Union(policy) | BinaryOp::Intersection(policy) => {
                matches!(policy, ConflictPolicy::Min | ConflictPolicy::Max)
            }
            BinaryOp::SymmetricDifference => true,
            BinaryOp::Difference | BinaryOp::Mask => false,
            BinaryOp::Custom(_) => false,
        }
    }

    /// 既存カーネルへ委譲して二項演算を実行する。
    fn run(self, lhs: &C, rhs: &C) -> Result<C, Error> {
        match self {
            BinaryOp::Union(policy) => {
                <Union as BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(
                    lhs, rhs, policy,
                )
            }
            BinaryOp::Intersection(policy) => <Intersection as BinaryOperator<
                C::Value,
                C::Value,
            >>::execution::<C, C, C>(lhs, rhs, policy),
            BinaryOp::Difference => {
                <Difference as BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(
                    lhs,
                    rhs,
                    (),
                )
            }
            BinaryOp::SymmetricDifference => <SymmetricDifference as BinaryOperator<
                C::Value,
                C::Value,
            >>::execution::<C, C, C>(lhs, rhs, ()),
            BinaryOp::Mask => {
                <Mask as BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, ())
            }
            BinaryOp::Custom(kernel) => kernel.run(lhs, rhs),
        }
    }
}

impl<C: SpatialIdCollection> Plan<C> {
    // ---- 構築（チェーンビルダー） ----

    /// コレクションをチェーンの起点に持ち上げる。
    /// 通常は [`SpatialIdCollection::plan`](crate::SpatialIdCollection::plan) 経由で呼ぶ。
    pub fn source(collection: C) -> Self {
        Plan::Source(collection)
    }

    fn unary(self, op: UnaryOp<C>) -> Self {
        Plan::Unary {
            op,
            input: Box::new(self),
        }
    }

    fn binary(self, op: BinaryOp<C>, other: Plan<C>) -> Self {
        Plan::Binary {
            op,
            lhs: Box::new(self),
            rhs: Box::new(other),
        }
    }

    pub fn shift_f(self, z: u8, index: i32) -> Self {
        self.unary(UnaryOp::ShiftF(ShiftParam { z, index }))
    }
    pub fn shift_x(self, z: u8, index: i32) -> Self {
        self.unary(UnaryOp::ShiftX(ShiftParam { z, index }))
    }
    pub fn shift_y(self, z: u8, index: i32) -> Self {
        self.unary(UnaryOp::ShiftY(ShiftParam { z, index }))
    }
    pub fn stretch_x(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        self.unary(UnaryOp::StretchX(StretchParam { z, index, conflict }))
    }
    pub fn fill(self, default: C::Value) -> Self {
        self.unary(UnaryOp::Fill(default))
    }

    /// enum を編集せずに任意の単項演算を差し込む拡張点（手書きカーネル用）。
    pub fn apply(self, kernel: impl UnaryKernel<C> + 'static) -> Self {
        self.unary(UnaryOp::Custom(Box::new(kernel)))
    }

    /// 既存の [`UnaryOperator`] をそのまま差し込む。
    ///
    /// `Op` が [`UnaryOperator`] を実装していれば、手書きの [`UnaryKernel`] なしで載る。
    /// 例：`plan.apply_op::<XShift>(ShiftParam { z, index })`
    pub fn apply_op<Op>(self, param: <Op as UnaryOperator<C::Value>>::CustomParameter) -> Self
    where
        Op: UnaryOperator<C::Value, ResultValue = C::Value> + 'static,
        <Op as UnaryOperator<C::Value>>::CustomParameter: 'static,
    {
        self.apply(UnaryOpKernel::<Op, _> {
            param,
            _op: PhantomData,
        })
    }

    pub fn union(self, other: Plan<C>, conflict: ConflictPolicy<C::Value>) -> Self {
        self.binary(BinaryOp::Union(conflict), other)
    }
    pub fn intersection(self, other: Plan<C>, conflict: ConflictPolicy<C::Value>) -> Self {
        self.binary(BinaryOp::Intersection(conflict), other)
    }
    pub fn difference(self, other: Plan<C>) -> Self {
        self.binary(BinaryOp::Difference, other)
    }
    pub fn symmetric_difference(self, other: Plan<C>) -> Self {
        self.binary(BinaryOp::SymmetricDifference, other)
    }
    pub fn mask(self, other: Plan<C>) -> Self {
        self.binary(BinaryOp::Mask, other)
    }

    /// enum を編集せずに任意の二項演算を差し込む拡張点（手書きカーネル用）。
    pub fn combine(self, other: Plan<C>, kernel: impl BinaryKernel<C> + 'static) -> Self {
        self.binary(BinaryOp::Custom(Box::new(kernel)), other)
    }

    /// 既存の [`BinaryOperator`] をそのまま差し込む。
    ///
    /// `Op` が [`BinaryOperator`] を実装していれば、手書きの [`BinaryKernel`] なしで載る。
    pub fn combine_op<Op>(
        self,
        other: Plan<C>,
        param: <Op as BinaryOperator<C::Value, C::Value>>::CustomParameter,
    ) -> Self
    where
        Op: BinaryOperator<C::Value, C::Value, ResultValue = C::Value> + 'static,
        <Op as BinaryOperator<C::Value, C::Value>>::CustomParameter: 'static,
    {
        self.combine(
            other,
            BinaryOpKernel::<Op, _> {
                param,
                _op: PhantomData,
            },
        )
    }

    // ---- 最適化・実行 ----

    /// AST を意味を保ったまま書き換える（最適化）。現状は恒等演算の畳み込みのみ。
    /// `Custom` は不透明なため最適化対象から外れる（ただし自己申告の `is_identity` は尊重）。
    pub fn optimize(self) -> Self {
        match self {
            Plan::Source(collection) => Plan::Source(collection),
            Plan::Unary { op, input } => {
                let input = input.optimize();
                if op.is_identity() {
                    input
                } else {
                    Plan::Unary {
                        op,
                        input: Box::new(input),
                    }
                }
            }
            Plan::Binary { op, lhs, rhs } => Plan::Binary {
                op,
                lhs: Box::new(lhs.optimize()),
                rhs: Box::new(rhs.optimize()),
            },
        }
    }

    /// 最適化せず、プランをそのまま実行して結果を材化する。
    pub fn execution(self) -> Result<C, Error> {
        match self {
            Plan::Source(collection) => Ok(collection),
            Plan::Unary { op, input } => {
                let input = input.execution()?;
                op.run(&input)
            }
            Plan::Binary { op, lhs, rhs } => {
                let lhs = lhs.execution()?;
                let rhs = rhs.execution()?;
                op.run(&lhs, &rhs)
            }
        }
    }

    /// 最適化してから実行する。
    pub fn optimized_execution(self) -> Result<C, Error> {
        self.optimize().execution()
    }
}
