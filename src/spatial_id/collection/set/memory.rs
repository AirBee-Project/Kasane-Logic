use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Collection, Rank,
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
pub struct SetOnMemoryInner {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<Rank, FlexId>,
    next_rank: u64,
    recycled_ranks: Vec<u64>,
}

impl Default for SetOnMemoryInner {
    fn default() -> Self {
        Self {
            f: Default::default(),
            x: Default::default(),
            y: Default::default(),
            main: Default::default(),
            next_rank: 0,
            recycled_ranks: vec![],
        }
    }
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
}

impl Collection for SetOnMemoryInner {
    type Dimension = BTreeMap<Segment, RoaringTreemap>;

    fn f(&self) -> &Self::Dimension {
        todo!()
    }

    fn f_mut(&mut self) -> &mut Self::Dimension {
        todo!()
    }

    fn x(&self) -> &Self::Dimension {
        todo!()
    }

    fn x_mut(&mut self) -> &mut Self::Dimension {
        todo!()
    }

    fn y(&self) -> &Self::Dimension {
        todo!()
    }

    fn y_mut(&mut self) -> &mut Self::Dimension {
        todo!()
    }

    fn fetch_rank(&mut self) -> u64 {
        todo!()
    }

    fn return_rank(&mut self, rank: u64) {
        todo!()
    }
}
//===========================================
