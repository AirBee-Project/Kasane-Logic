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
        let check_and_remove =
            |core: &mut VBitCore<()>, sibling: FlexId, parent: FlexId| -> Option<FlexId> {
                if core.remove(&sibling.flex_id_rank()).is_some() {
                    Some(parent)
                } else {
                    None
                }
            };

        if let Some(p) = flex_id.f_parent() {
            if let Some(parent) = check_and_remove(&mut self.core, flex_id.f_sibling(), p) {
                unsafe { self.join_insert_unchecked(parent) };
                return;
            }
        }

        // X軸
        if let Some(p) = flex_id.x_parent() {
            if let Some(parent) = check_and_remove(&mut self.core, flex_id.x_sibling(), p) {
                unsafe { self.join_insert_unchecked(parent) };
                return;
            }
        }

        // Y軸
        if let Some(p) = flex_id.y_parent() {
            if let Some(parent) = check_and_remove(&mut self.core, flex_id.y_sibling(), p) {
                unsafe { self.join_insert_unchecked(parent) };
                return;
            }
        }

        unsafe { self.insert_unchecked(flex_id) };
    }

    unsafe fn insert_unchecked(&mut self, flex_id: FlexId) {
        self.core.insert(flex_id, ());
    }
}
