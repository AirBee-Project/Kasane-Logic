use roaring::RoaringTreemap;
use std::{collections::BTreeMap, hash::Hash};

use crate::{FlexId, FlexIdRank, Segment, spatial_id::helpers::Dimension};

/// 空間IDの物理的な格納とインデックス管理を行う中核構造体
/// T: IDに関連付けられる追加メタデータ（TableならValueRank、Setなら()）
#[derive(Clone, Debug, Default)]
pub struct CollectionCore<T> {
    segment: BTreeMap<(Dimension, Segment), RoaringTreemap>,
    main: BTreeMap<FlexIdRank, (FlexId, T)>,
}

impl<T> CollectionCore<T> {}
