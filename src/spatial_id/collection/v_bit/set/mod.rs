use crate::{FlexId, spatial_id::collection::v_bit::core::VBitCore};
pub mod impls;
pub mod ops;
pub mod test;

#[derive(Default, Debug, Clone)]
pub struct VBitSet {
    core: VBitCore<()>,
}

impl VBitSet {
    unsafe fn join_insert_unchecked(&mut self, flex_id: FlexId) {
        // 兄弟がに存在すればそれを削除し、昇格すべき親を返す。
        let check_and_remove =
            |core: &mut VBitCore<()>, sibling: FlexId, parent: Option<FlexId>| -> Option<FlexId> {
                if core.contains(&sibling) {
                    core.remove(&sibling.flex_id_rank());
                    parent
                } else {
                    None
                }
            };

        // F軸の結合チェック
        if let Some(parent) =
            check_and_remove(&mut self.core, flex_id.f_sibling(), flex_id.f_parent())
        {
            unsafe { self.insert_unchecked(parent) };
            return;
        }

        // X軸の結合チェック
        if let Some(parent) =
            check_and_remove(&mut self.core, flex_id.x_sibling(), flex_id.x_parent())
        {
            unsafe { self.insert_unchecked(parent) };
            return;
        }

        // Y軸の結合チェック
        if let Some(parent) =
            check_and_remove(&mut self.core, flex_id.y_sibling(), flex_id.y_parent())
        {
            unsafe { self.insert_unchecked(parent) };
            return;
        }
    }

    unsafe fn insert_unchecked(&mut self, flex_id: FlexId) {
        self.core.insert(flex_id, ());
    }
}
