use crate::{Coordinate, CoverRangeIds, CoverSingleIds, Error, GeometryError, RangeId, SingleId};

///地域メッシュを表す型
//より詳細なメッシュが必要になったらu32ではなくu64でもいいと思う
pub struct RegionMesh(MeshType);

impl RegionMesh {
    pub fn new(code: u32) -> Result<Self, Error> {
        let digit = if code == 0 { 1 } else { code.ilog10() + 1 };
        if digit == 4 {
            Ok(Self(MeshType::First(code)))
        } else if digit == 6 {
            let f2 = (code % 100) / 10;
            let h2 = code % 10;
            if f2 > 7 || h2 > 7 {
                return Err(GeometryError::RegionmeshNotExist { index: code }.into());
            }
            Ok(Self(MeshType::Second(code)))
        } else if digit == 8 {
            let f2 = (code % 10000) / 1000;
            let h2 = (code % 1000) / 100;
            if f2 > 7 || h2 > 7 {
                return Err(GeometryError::RegionmeshNotExist { index: code }.into());
            }

            Ok(Self(MeshType::Third(code)))
        } else {
            Err(GeometryError::RegionmeshNotExist { index: code }.into())
        }
    }
    /// # Safety
    /// この操作はunsafeである。地域メッシュの規格に適合するかどうかに関わらず、RegionMesh型に包む。
    pub unsafe fn new_unchecked(mesh: MeshType) -> Self {
        Self(mesh)
    }
    /// 範囲メッシュのコードを返す
    pub fn code(&self) -> u32 {
        self.0.code()
    }

    /// 複数の [`RegionMesh`] と値のペアを、指定したズームレベル `z` で [`hashbrown::HashMap<SingleId, V>`] に変換する。
    /// 重なり合う空間領域の値は `merge_function` に従って集約・競合解消される。
    pub fn regionmesh_into_hashmap<V, I, F>(
        iter: I,
        z: impl Into<u8>,
        merge_function: F,
    ) -> Result<hashbrown::HashMap<SingleId, V>, Error>
    where
        I: IntoIterator<Item = (RegionMesh, V)>,
        V: Clone + PartialEq + Send + Sync + 'static,
        F: Fn(V, V) -> V,
    {
        let z = z.into();
        let mut aggregated = hashbrown::HashMap::<SingleId, V>::new();

        for (mesh, value) in iter {
            for single_id in mesh.cover_single_ids(z)? {
                aggregated
                    .entry(single_id)
                    .and_modify(|existing| {
                        *existing = merge_function(existing.clone(), value.clone());
                    })
                    .or_insert_with(|| value.clone());
            }
        }
        Ok(aggregated)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
///メッシュの種類による分岐を表す列挙型
pub enum MeshType {
    ///1次メッシュを表す
    First(u32),
    ///2次メッシュを表す
    Second(u32),
    ///3次メッシュを表す
    Third(u32),
}

impl MeshType {
    pub fn code(&self) -> u32 {
        match *self {
            MeshType::First(code) | MeshType::Second(code) | MeshType::Third(code) => code,
        }
    }
}

impl CoverSingleIds for RegionMesh {
    fn cover_single_ids(&self, z: impl Into<u8>) -> Result<impl Iterator<Item = SingleId>, Error> {
        Ok(self.cover_range_ids(z.into())?.flat_map(|r| r.single_ids()))
    }
    fn cover_single_ids_with<V>(
        &self,
        z: impl Into<u8>,
        value: V,
    ) -> Result<impl Iterator<Item = (SingleId, V)>, Error>
    where
        V: Clone + 'static,
    {
        Ok(self
            .cover_range_ids(z.into())?
            .flat_map(|r| r.single_ids())
            .map(move |id| (id, value.clone())))
    }
}

///標準地域メッシュを覆うようにRangeIdを生成する。Fにはデフォルトで0が割り当てられている。
impl CoverRangeIds for RegionMesh {
    fn cover_range_ids_with<V>(
        &self,
        z: impl Into<u8>,
        value: V,
    ) -> Result<impl Iterator<Item = (RangeId, V)>, Error> {
        let z = z.into();
        let (id1, id2) = match self.0 {
            MeshType::First(code) => {
                let f = (code / 100) as f64;
                let h = (code % 100) as f64;
                let lat_min = f / 1.5;
                let lat_max = lat_min + 1.0 / 1.5;
                let lon_min = h + 100.0;
                let lon_max = lon_min + 1.0;
                let id1 = Coordinate::new(lat_min, lon_min, 0.0)?.single_id(z)?;
                let id2 = Coordinate::new(lat_max, lon_max, 0.0)?.single_id(z)?;
                (id1, id2)
            }
            MeshType::Second(code) => {
                let f1 = (code / 10000) as f64;
                let h1 = ((code % 10000) / 100) as f64;
                let f2 = ((code % 100) / 10) as f64;
                let h2 = (code % 10) as f64;
                let lat_min = f1 / 1.5 + f2 / 12.0;
                let lat_max = lat_min + 1.0 / 12.0;
                let lon_min = h1 + 100.0 + h2 / 8.0;
                let lon_max = lon_min + 1.0 / 8.0;
                let id1 = Coordinate::new(lat_min, lon_min, 0.0)?.single_id(z)?;
                let id2 = Coordinate::new(lat_max, lon_max, 0.0)?.single_id(z)?;
                (id1, id2)
            }
            MeshType::Third(code) => {
                let f1 = (code / 1000000) as f64;
                let h1 = ((code % 1000000) / 10000) as f64;
                let f2 = ((code % 10000) / 1000) as f64;
                let h2 = ((code % 1000) / 100) as f64;
                let f3 = ((code % 100) / 10) as f64;
                let h3 = (code % 10) as f64;
                let lat_min = f1 / 1.5 + f2 / 12.0 + f3 / 120.0;
                let lat_max = lat_min + 1.0 / 120.0;
                let lon_min = h1 + 100.0 + h2 / 8.0 + h3 / 80.0;
                let lon_max = lon_min + 1.0 / 80.0;
                let id1 = Coordinate::new(lat_min, lon_min, 0.0)?.single_id(z)?;
                let id2 = Coordinate::new(lat_max, lon_max, 0.0)?.single_id(z)?;
                (id1, id2)
            }
        };
        let range_id = RangeId::new(
            z,
            [id1.f(), id2.f()],
            [id1.x(), id2.x()],
            [id1.y(), id2.y()],
        )?;

        Ok(core::iter::once((range_id, value)))
    }
}
