# メソッド命名規則

## 1. `spatial_` 接頭辞

空間 ID は空間 3 軸（F / X / Y）と時間（T）を持つ。`temporal_id` feature が有効なとき、名前だけでは「空間だけの操作」か「時空間の操作」かを区別できないため、接頭辞で明示する。

- 空間 3 軸だけを扱い、時間を考慮しないメソッドには `spatial_` を付ける。
  - 時間をそのまま引き継いで、空間だけを変える操作（ズームの変換、親子・兄弟、近傍、移動）。
  - 時間を無視して、空間の情報だけを返す操作（中心座標、頂点）。
- 時間も扱うメソッド、時間だけを扱うメソッドには付けない。

| 区分 | 例 |
| :--- | :--- |
| `spatial_` を付ける | `spatial_center`, `spatial_vertices`, `spatial_parent_at_zoom`, `spatial_children_at_zoom`, `spatial_parent`, `spatial_children`, `spatial_parents`, `spatial_siblings`, `spatial_face_neighbors`, `spatial_edge_neighbors`, `spatial_vertex_neighbors`, `spatial_all_neighbors`, `spatial_shift`, `is_spatial_face_neighbor` |
| 付けない（時空間を扱う） | `contains`, `intersects`, `single_ids` |
| 付けない（時間だけを扱う） | `without_time`, `with_time_span`, `with_time_segment`, `contains_time`, `seconds_range`, `t_index` |

軸を名前に含むメソッド（`move_f`, `length_x_meters` など）は、扱う軸が名前から分かるので付けない。

述語は `is_` の直後に置く（`is_spatial_face_neighbor`）。
