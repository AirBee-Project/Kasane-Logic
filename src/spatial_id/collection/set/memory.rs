use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Rank,
        set::{SetStorage, logic::SetLogic},
    },
    flex_id::FlexId,
    segment::Segment,
};

//===========================================
//ユーザーが実際に触るデフォルトの「Set」型
//SetOnMemoryInner型が見えるとややこしいので薄いラップ
//基本的な公開メゾットはDerefとDerefMutによりそのまま伝播するようになっている
#[derive(Default)]
pub struct SetOnMemory(SetLogic<SetOnMemoryInner>);

impl Deref for SetOnMemory {
    type Target = SetLogic<SetOnMemoryInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SetOnMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
//===========================================

//===========================================
//SetLogicをメモリ上でBTreeMapを用いて実装したもの
//SetLogicの恩恵によりストレージの記法さえ実装すれば動作するようにできている
#[derive(Default)]
pub struct SetOnMemoryInner {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<Rank, FlexId>,
}

impl SetStorage for SetOnMemoryInner {
    type Main = BTreeMap<Rank, FlexId>;
    type Dimension = BTreeMap<Segment, RoaringTreemap>;

    fn main(&self) -> &Self::Main {
        &self.main
    }

    fn main_mut(&mut self) -> &mut Self::Main {
        &mut self.main
    }

    fn f(&self) -> &Self::Dimension {
        &self.f
    }

    fn f_mut(&mut self) -> &mut Self::Dimension {
        &mut self.f
    }

    fn x(&self) -> &Self::Dimension {
        &self.x
    }

    fn x_mut(&mut self) -> &mut Self::Dimension {
        &mut self.x
    }

    fn y(&self) -> &Self::Dimension {
        &self.y
    }

    fn y_mut(&mut self) -> &mut Self::Dimension {
        &mut self.y
    }
}
//===========================================
