//! ドローン経路探索モジュール
//!
//! A*アルゴリズムを使用して、禁止区域を避けながら
//! 2点間の最適な経路を計算する。

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use crate::{encode_id_set::EncodeIDSet, point::Coordinate};

use super::line::{coordinate_to_voxel_float, get_line_voxels_dda, Voxel};

/// A*探索用のノード
#[derive(Clone)]
#[allow(dead_code)]
struct Node {
    voxel: Voxel,
    parent: Option<Box<Node>>,
    g: f64,       // スタートからのコスト
    h: f64,       // ゴールへのヒューリスティック
    pen: f64,     // タイブレーカーペナルティ
    f: f64,       // 総コスト (g + h + pen)
}

impl Node {
    fn new(voxel: Voxel, parent: Option<Box<Node>>, g: f64, h: f64, pen: f64) -> Self {
        Self {
            voxel,
            parent,
            g,
            h,
            pen,
            f: g + h + pen,
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.voxel == other.voxel
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeapは最大ヒープなので、fが小さい方を優先するために逆順にする
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.voxel.hash(state);
    }
}

/// ヒューリスティック関数（高さ方向の比率を考慮）
fn heuristic_ratio(a: &Voxel, b: &Voxel, vertical_length: f64) -> f64 {
    let dx = (a.x as i64 - b.x as i64).abs() as f64;
    let dy = (a.y as i64 - b.y as i64).abs() as f64;
    let df = (a.f - b.f).abs() as f64;
    let k = 1.2; // ヒューリスティックの重み
    k * (dx * dx + dy * dy + (df * vertical_length).powi(2)).sqrt()
}

/// パスが通過可能かどうかをDDAで確認
fn is_path_clear(v1: &Voxel, v2: &Voxel, ban_set: &HashSet<Voxel>) -> bool {
    let line_voxels = get_line_voxels_dda(v1.to_voxel_float(), v2.to_voxel_float());
    for voxel in line_voxels {
        if ban_set.contains(&voxel) {
            return false;
        }
    }
    true
}

/// 隣接する2点間の安全な経路を探索
fn search_saferoute(v1: &Voxel, v2: &Voxel, ban_set: &HashSet<Voxel>) -> Vec<Voxel> {
    let mut route = vec![*v1];

    let (f1, x1, y1) = (v1.f, v1.x as i64, v1.y as i64);
    let (f2, x2, y2) = (v2.f, v2.x as i64, v2.y as i64);

    let df = f2 - f1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let relation = df.abs() + dx.abs() + dy.abs();

    if relation == 1 {
        route.push(*v2);
    } else if relation == 2 {
        let mid_candidates = [
            v1.adjust(df, 0, 0),
            v1.adjust(0, dx, 0),
            v1.adjust(0, 0, dy),
        ];
        for v in &mid_candidates {
            if *v != *v1 && !ban_set.contains(v) {
                route.push(*v);
                break;
            }
        }
        route.push(*v2);
    } else if relation == 3 {
        let first_candidates = [
            v1.adjust(df, 0, 0),
            v1.adjust(0, dx, 0),
            v1.adjust(0, 0, dy),
        ];
        for (i, first_cand) in first_candidates.iter().enumerate() {
            if !ban_set.contains(first_cand) {
                let second_candidates: [Voxel; 2] = match i {
                    0 => [v1.adjust(df, dx, 0), v1.adjust(df, 0, dy)],
                    1 => [v1.adjust(df, dx, 0), v1.adjust(0, dx, dy)],
                    _ => [v1.adjust(df, 0, dy), v1.adjust(0, dx, dy)],
                };
                for second_cand in &second_candidates {
                    if !ban_set.contains(second_cand) {
                        route.push(*first_cand);
                        route.push(*second_cand);
                        break;
                    }
                }
                break;
            }
        }
        route.push(*v2);
    }

    route
}

/// パスをスムージング
fn smooth_path(path: &[Voxel], ban_set: &HashSet<Voxel>) -> Vec<Voxel> {
    if path.len() <= 1 {
        return path.to_vec();
    } else if path.len() == 2 {
        return search_saferoute(&path[0], &path[1], ban_set);
    }

    let mut smoothed_path = vec![path[0]];
    let mut current_idx = 0;

    while current_idx < path.len() - 1 {
        let mut next_idx = current_idx + 1;
        let mut flag = false;
        let mut line: Vec<Voxel> = Vec::new();

        // 現在位置からできるだけ遠くの到達可能な点を探す
        for check_idx in (current_idx + 2..path.len()).rev() {
            if is_path_clear(&path[current_idx], &path[check_idx], ban_set) {
                next_idx = check_idx;
                line = get_line_voxels_dda(
                    path[current_idx].to_voxel_float(),
                    path[check_idx].to_voxel_float(),
                );
                if !line.is_empty() {
                    line.remove(0); // 始点を削除
                }
                flag = true;
                break;
            }
        }

        if flag {
            smoothed_path.extend(line);
        } else {
            let mut complemented = search_saferoute(&path[current_idx], &path[next_idx], ban_set);
            if !complemented.is_empty() {
                complemented.remove(0); // 始点を削除
            }
            smoothed_path.extend(complemented);
        }
        current_idx = next_idx;
    }

    smoothed_path
}

/// タイブレーカーペナルティを計算
/// スタートからゴールへの直線から離れるほどペナルティが大きくなる
fn tie_penalty(
    current: &Voxel,
    start: &Voxel,
    goal: &Voxel,
    vertical_length: f64,
    sq_inv_abs_vec_sg: f64,
) -> f64 {
    let vec_sg_x = (goal.x as i64 - start.x as i64) as f64;
    let vec_sg_y = (goal.y as i64 - start.y as i64) as f64;
    let vec_sg_f = (goal.f - start.f) as f64 * vertical_length;

    let vec_cg_x = (goal.x as i64 - current.x as i64) as f64;
    let vec_cg_y = (goal.y as i64 - current.y as i64) as f64;
    let vec_cg_f = (goal.f - current.f) as f64 * vertical_length;

    let dot_product = vec_sg_f * vec_cg_f + vec_sg_x * vec_cg_x + vec_sg_y * vec_cg_y;
    let sq_abs_vec_cg = vec_cg_f.powi(2) + vec_cg_x.powi(2) + vec_cg_y.powi(2);
    let cg_sin = (sq_abs_vec_cg - sq_inv_abs_vec_sg * dot_product.powi(2))
        .max(0.0)
        .sqrt();

    let p = 0.2; // ペナルティの重み
    p * cg_sin
}

/// EncodeIDSetから禁止区域のVoxelのHashSetを構築
fn build_ban_set(z: u8, ban_list: &EncodeIDSet) -> HashSet<Voxel> {
    let mut ban_set = HashSet::new();

    for encode_id in ban_list.iter() {
        // EncodeIDからSpaceTimeIDにデコードし、Voxelを取得
        let stid = encode_id.decode();
        // 同じズームレベルの場合のみ追加
        if stid.z == z {
            // 範囲内の全てのVoxelを追加
            for f in stid.f[0]..=stid.f[1] {
                for x in stid.x[0]..=stid.x[1] {
                    for y in stid.y[0]..=stid.y[1] {
                        ban_set.insert(Voxel { z, f, x, y });
                    }
                }
            }
        }
    }

    ban_set
}

/// ドローンの経路を計算する
///
/// A*アルゴリズムを使用して、禁止区域を避けながら最適な経路を探索する。
///
/// # 引数
/// * `z` - ズームレベル
/// * `start` - 出発地点の座標
/// * `end` - 目的地の座標
/// * `ban_list` - 飛行禁止区域のID集合
///
/// # 戻り値
/// 経路上のボクセルIDの集合。経路が見つからない場合は空の集合を返す。
pub fn route(z: u8, start: Coordinate, end: Coordinate, ban_list: &EncodeIDSet) -> EncodeIDSet {
    const MAX_ITER: usize = 10_000_000;

    // 座標からボクセルに変換
    let start_voxel = coordinate_to_voxel_float(&start, z).to_voxel();
    let goal_voxel = coordinate_to_voxel_float(&end, z).to_voxel();

    // 禁止区域のセットを構築
    let ban_set = build_ban_set(z, &ban_list);

    // 始点か終点が禁止区域の場合は空を返す
    if ban_set.contains(&start_voxel) || ban_set.contains(&goal_voxel) {
        return EncodeIDSet::new();
    }

    // 高さ方向の比率を計算
    let len_xy = (start_voxel.get_length("x")
        + start_voxel.get_length("y")
        + goal_voxel.get_length("x")
        + goal_voxel.get_length("y"))
        / 4.0;
    let len_f = (start_voxel.get_length("f") + goal_voxel.get_length("f")) / 2.0;
    let vertical_length = len_f / len_xy;

    // A*探索用のデータ構造
    let mut open_list: BinaryHeap<Node> = BinaryHeap::new();
    let mut closed_set: HashSet<Voxel> = HashSet::new();

    // スタートからゴールへのベクトルの二乗ノルムの逆数を事前計算
    let df_sg = (goal_voxel.f - start_voxel.f) as f64 * vertical_length;
    let dx_sg = (goal_voxel.x as i64 - start_voxel.x as i64) as f64;
    let dy_sg = (goal_voxel.y as i64 - start_voxel.y as i64) as f64;
    let sq_abs_vec_sg = df_sg.powi(2) + dx_sg.powi(2) + dy_sg.powi(2);
    let sq_inv_abs_vec_sg = if sq_abs_vec_sg > 0.0 {
        1.0 / sq_abs_vec_sg
    } else {
        0.0
    };

    // スタートノードを追加
    let start_node = Node::new(
        start_voxel,
        None,
        0.0,
        heuristic_ratio(&start_voxel, &goal_voxel, vertical_length),
        0.0,
    );
    open_list.push(start_node);

    let mut found_node: Option<Node> = None;
    let mut loop_count = 0;

    // A*メインループ
    while let Some(current_node) = open_list.pop() {
        loop_count += 1;
        if loop_count > MAX_ITER {
            // タイムアウト
            return EncodeIDSet::new();
        }

        // ゴールに到達したか確認
        if current_node.voxel == goal_voxel {
            found_node = Some(current_node);
            break;
        }

        // 既に探索済みならスキップ
        if closed_set.contains(&current_node.voxel) {
            continue;
        }

        closed_set.insert(current_node.voxel);

        // 26近傍を探索（3x3x3 - 1）
        for df in -1..=1 {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if df == 0 && dx == 0 && dy == 0 {
                        continue;
                    }

                    let neighbor_voxel = current_node.voxel.adjust(df, dx, dy);

                    // 禁止区域または探索済みならスキップ
                    if ban_set.contains(&neighbor_voxel) || closed_set.contains(&neighbor_voxel) {
                        continue;
                    }

                    // すり抜け防止チェック
                    let move_sum = df.abs() + dx.abs() + dy.abs();
                    if move_sum > 1 {
                        let mut blocked_components = 0;
                        let mut check_count = 0;

                        if df != 0 {
                            check_count += 1;
                            if ban_set.contains(&current_node.voxel.adjust(df, 0, 0)) {
                                blocked_components += 1;
                            }
                        }
                        if dx != 0 {
                            check_count += 1;
                            if ban_set.contains(&current_node.voxel.adjust(0, dx, 0)) {
                                blocked_components += 1;
                            }
                        }
                        if dy != 0 {
                            check_count += 1;
                            if ban_set.contains(&current_node.voxel.adjust(0, 0, dy)) {
                                blocked_components += 1;
                            }
                        }

                        // 全ての単軸移動がブロックされている場合はスキップ
                        if blocked_components == check_count {
                            continue;
                        }
                    }

                    // 移動コストを計算
                    let cost_sq = (dx as f64).powi(2)
                        + (dy as f64).powi(2)
                        + (df as f64 * vertical_length).powi(2);
                    let move_cost = cost_sq.sqrt();

                    let new_g = current_node.g + move_cost;
                    let new_h = heuristic_ratio(&neighbor_voxel, &goal_voxel, vertical_length);
                    let new_pen = tie_penalty(
                        &neighbor_voxel,
                        &start_voxel,
                        &goal_voxel,
                        vertical_length,
                        sq_inv_abs_vec_sg,
                    );

                    let new_node = Node::new(
                        neighbor_voxel,
                        Some(Box::new(current_node.clone())),
                        new_g,
                        new_h,
                        new_pen,
                    );

                    open_list.push(new_node);
                }
            }
        }
    }

    // 経路が見つからなかった場合
    let found_node = match found_node {
        Some(node) => node,
        None => return EncodeIDSet::new(),
    };

    // 経路を復元
    let mut path_voxels: Vec<Voxel> = Vec::new();
    let mut curr: Option<&Node> = Some(&found_node);
    while let Some(node) = curr {
        path_voxels.push(node.voxel);
        curr = node.parent.as_ref().map(|b| b.as_ref());
    }
    path_voxels.reverse();

    // パスをスムージング
    let smoothed_voxels = smooth_path(&path_voxels, &ban_set);

    // 結果をEncodeIDSetに変換
    let mut result = EncodeIDSet::new();
    for voxel in smoothed_voxels {
        result.uncheck_insert(voxel.to_encode_id());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_same_point() {
        // 同じ点間の経路
        let point = Coordinate {
            latitude: 35.6809,
            longitude: 139.7673,
            altitude: 0.0,
        };
        let ban_list = EncodeIDSet::new();
        let result = route(15, point, point, &ban_list);
        // 同じ点なので1つのボクセルのみ
        assert!(result.iter().count() >= 1);
    }

    #[test]
    fn test_route_no_obstacles() {
        // 障害物なしの経路
        let tokyo = Coordinate {
            latitude: 35.6809,
            longitude: 139.7673,
            altitude: 100.0,
        };
        let shinjuku = Coordinate {
            latitude: 35.6896,
            longitude: 139.6999,
            altitude: 100.0,
        };
        let ban_list = EncodeIDSet::new();
        let result = route(15, tokyo, shinjuku, &ban_list);
        // 経路が存在するはず
        assert!(result.iter().count() >= 1);
    }

    #[test]
    fn test_heuristic_ratio() {
        let a = Voxel {
            z: 15,
            f: 0,
            x: 0,
            y: 0,
        };
        let b = Voxel {
            z: 15,
            f: 1,
            x: 1,
            y: 1,
        };
        let h = heuristic_ratio(&a, &b, 1.0);
        // 3次元のユークリッド距離 * 1.2
        let expected = 1.2 * (3.0_f64).sqrt();
        assert!((h - expected).abs() < 0.001);
    }

    #[test]
    fn test_is_path_clear() {
        let v1 = Voxel {
            z: 15,
            f: 0,
            x: 0,
            y: 0,
        };
        let v2 = Voxel {
            z: 15,
            f: 0,
            x: 3,
            y: 0,
        };
        let ban_set = HashSet::new();
        assert!(is_path_clear(&v1, &v2, &ban_set));

        // 障害物がある場合
        let mut ban_set_with_obstacle = HashSet::new();
        ban_set_with_obstacle.insert(Voxel {
            z: 15,
            f: 0,
            x: 1,
            y: 0,
        });
        assert!(!is_path_clear(&v1, &v2, &ban_set_with_obstacle));
    }

    #[test]
    fn test_search_saferoute_adjacent() {
        let v1 = Voxel {
            z: 15,
            f: 0,
            x: 0,
            y: 0,
        };
        let v2 = Voxel {
            z: 15,
            f: 0,
            x: 1,
            y: 0,
        };
        let ban_set = HashSet::new();
        let route = search_saferoute(&v1, &v2, &ban_set);
        assert_eq!(route.len(), 2);
        assert_eq!(route[0], v1);
        assert_eq!(route[1], v2);
    }
}
