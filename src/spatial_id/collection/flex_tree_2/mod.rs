use crate::{FlexId, SpatialId};

#[derive(Default)]
pub struct FlexTreeCore2 {
    /// ルートの位置を動的に持つ
    /// ルートは内部に抱えるFlexIdをすべて包むような最小のFlexIdとなる
    upper_root: Option<FlexId>,
    lower_root: Option<FlexId>,
}

impl FlexTreeCore2 {
    pub fn new() -> Self {
        Self {
            upper_root: None,
            lower_root: None,
        }
    }

    pub fn insert<S: SpatialId>(&mut self, target: S) {
        for flex_id in target.into_iter() {
            // 処理対象のrootを取り出す
            let mut root = if flex_id.f_index().is_positive() {
                &mut self.upper_root
            } else {
                &mut self.lower_root
            };

            if let Some(root_flex_id) = root {
                root_flex_id
            } else {
                root = &mut Some(flex_id)
            }
        }
    }
}
