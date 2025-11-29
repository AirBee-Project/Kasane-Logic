use crate::encode_id_map::EncodeIDMap;

pub mod decode;
pub mod difference;
pub mod get;
pub mod insert;
pub mod intersection;
pub mod iterator;
pub mod remove;
pub mod union;

// Re-export LayerInfo from encode_id_map for backward compatibility
pub use crate::encode_id_map::LayerInfo;

/// 時空間IDの集合を効率的に管理するデータ構造
/// 重複する範囲を自動的に統合し、階層構造を用いて効率的に格納する。
/// 内部的には`EncodeIDMap<()>`を使用しています。
#[derive(Debug, Clone)]
pub struct EncodeIDSet {
    inner: EncodeIDMap<()>,
}

impl EncodeIDSet {
    /// 新しい空の時空間ID集合を作成
    pub fn new() -> Self {
        Self {
            inner: EncodeIDMap::new(),
        }
    }

    /// 内部のEncodeIDMapへの参照を取得する
    pub(crate) fn inner(&self) -> &EncodeIDMap<()> {
        &self.inner
    }

    /// 内部のEncodeIDMapへの可変参照を取得する
    pub(crate) fn inner_mut(&mut self) -> &mut EncodeIDMap<()> {
        &mut self.inner
    }
}

impl Default for EncodeIDSet {
    fn default() -> Self {
        Self::new()
    }
}
