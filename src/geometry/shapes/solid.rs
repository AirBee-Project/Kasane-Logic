use crate::geometry::shapes::{polygon::Polygon, triangle::Triangle};
use crate::spatial_id::SpatialId;
use crate::{Coordinate, Ecef, Error, RangeId, SingleId};
use std::collections::{HashMap, HashSet, VecDeque};

/// 立体を表す型。
///
/// 複数の [Polygon] によって構成される閉じた領域を表現します。
/// この型は、作成時に必ず閉合性が検証されるため、作成できている場合は「閉じている」ことが保証されます。
#[derive(Debug, Clone)]
pub struct Solid {
    /// 立体を構成する面のリスト
    polygons: Vec<Polygon>,
}

impl Solid {
    /// 生の座標リスト群から [Solid] 作成。
    ///
    /// # 処理内容
    /// - 各面を[Polygon] に変換。
    /// - 閉合性の検証。
    /// - 穴がある場合は [Error::SolidNotWatertight] を返します。
    ///
    /// # 引数
    /// -  `raw_surfaces` - 各面の頂点リストの集合（LOD2の `lod2MultiSurface` などを想定）
    /// - `epsilon` - 同一点とみなす許容誤差（メートル単位）。
    pub fn new(raw_surfaces: Vec<Vec<Coordinate>>, epsilon: f64) -> Result<Self, Error> {
        let polygons: Vec<Polygon> = raw_surfaces
            .into_iter()
            .map(|coords| Polygon::new(coords, epsilon))
            .filter(|p| !p.vertices().is_empty()) // 無効な面は除外
            .collect();

        if polygons.is_empty() {
            return Err(Error::SolidNotWatertight { open_edge_count: 0 });
        }

        // 閉合性チェック
        let open_edges = Self::count_open_edges(&polygons, epsilon);
        if open_edges > 0 {
            return Err(Error::SolidNotWatertight {
                open_edge_count: open_edges,
            });
        }

        Ok(Self { polygons })
    }

    /// [Solid] 全体を三角形分割し、構成する [Triangle] のリストを返します。
    pub fn triangulate(&self) -> Vec<Triangle> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.triangulate())
            .collect()
    }

    /// 閉じていないエッジの数を数える内部ヘルパー関数
    fn count_open_edges(polygons: &[Polygon], epsilon: f64) -> usize {
        // エッジの出現回数を記録するマップ
        let mut edge_counts: HashMap<(QuantizedCoord, QuantizedCoord), usize> = HashMap::new();

        for polygon in polygons {
            let vertices = &polygon.vertices();
            let len = vertices.len();
            if len < 3 {
                continue;
            }

            for i in 0..len {
                let p1 = &vertices[i];
                let p2 = &vertices[(i + 1) % len]; // 最後の点と最初の点を結ぶ

                // ECEF座標系で量子化
                let e1: Ecef = (*p1).into();
                let e2: Ecef = (*p2).into();
                let q1 = QuantizedCoord::new(&e1, epsilon);
                let q2 = QuantizedCoord::new(&e2, epsilon);

                // 縮退辺は無視
                if q1 == q2 {
                    continue;
                }

                // エッジの向きを正規化（小さい方を先に）してカウント
                let edge_key = if q1 < q2 { (q1, q2) } else { (q2, q1) };
                *edge_counts.entry(edge_key).or_insert(0) += 1;
            }
        }

        // 奇数回出現するエッジの数をカウント
        edge_counts.values().filter(|&count| count % 2 != 0).count()
    }

    /// 指定されたズームレベル `z` における、この [Solid] の表面を覆う [SingleId] の集合を返す。
    pub fn surface_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        // HashSetで重複を除去
        let mut unique_ids = HashSet::new();

        for polygon in &self.polygons {
            let triangles = polygon.triangulate();
            for triangle in triangles {
                let ids_iter = triangle.single_ids(z)?;
                for id in ids_iter {
                    unique_ids.insert(id);
                }
            }
        }

        Ok(unique_ids.into_iter())
    }

    //Todo

    pub fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let surface_set: HashSet<SingleId> = self.surface_single_ids(z)?.collect();
        let existence_range = surface_set.iter().fold(None, |acc, s| {
            match acc {
                None => Some((s.f(), s.x(), s.y(), s.f(), s.x(), s.y())), // 最初の要素を初期値にする
                Some((min_f, min_x, min_y, max_f, max_x, max_y)) => Some((
                    min_f.min(s.f()),
                    min_x.min(s.x()),
                    min_y.min(s.y()),
                    max_f.max(s.f()),
                    max_x.max(s.x()),
                    max_y.max(s.y()),
                )),
            }
        });
        let result = existence_range.unwrap();
        let mut cuboid_set: HashSet<SingleId> = RangeId::new(
            z,
            [result.0, result.3],
            [result.1, result.4],
            [result.2, result.5],
        )?
        .single_ids()
        .collect();
        let mut open_list: VecDeque<SingleId> = VecDeque::new();

        cuboid_set.retain(|id| {
            let is_boundary = id.f() == result.0
                || id.f() == result.3
                || id.x() == result.1
                || id.x() == result.4
                || id.y() == result.2
                || id.y() == result.5;

            if is_boundary && !surface_set.contains(id) {
                open_list.push_back(id.clone());
                false
            } else {
                true
            }
        });
        let directions = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        while let Some(current) = open_list.pop_front() {
            for (df, dx, dy) in directions {
                let mut neighbor = current.clone();

                // 各方向に移動を試みる
                // move_x は循環するため Ok 固定、move_f と move_y は範囲外ならエラーになる
                let move_result = if df != 0 {
                    neighbor.move_f(df)
                } else if dx != 0 {
                    neighbor.move_x(dx);
                    Ok(())
                } else {
                    neighbor.move_y(dy)
                };

                // 移動が成功（範囲内）した場合のみ判定
                if move_result.is_ok() {
                    // 条件：cuboidに含まれる かつ surfaceに含まれない
                    if cuboid_set.contains(&neighbor) && !surface_set.contains(&neighbor) {
                        cuboid_set.remove(&neighbor);
                        open_list.push_back(neighbor);
                    }
                }
            }
        }
        Ok(cuboid_set.into_iter())
    }

    // pub fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
    //     todo!()
    // }
}

/// ハッシュマップのキーにするために座標を整数化するラッパー
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
struct QuantizedCoord {
    x: i64,
    y: i64,
    z: i64,
}

impl QuantizedCoord {
    fn new(e: &Ecef, precision: f64) -> Self {
        Self {
            x: (e.x() / precision).round() as i64,
            y: (e.y() / precision).round() as i64,
            z: (e.z() / precision).round() as i64,
        }
    }
}
