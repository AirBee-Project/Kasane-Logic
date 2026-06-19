/// 特定の次元の方向へ移動
pub mod shift;

/// 特定の次元の方向へ引き延ばし
pub mod stretch;

/// 特定の次元の占有を絶対座標範囲へ揃える（起伏を平坦化）
pub mod level;

/// 値を持つ領域の最小範囲（AABB）の隙間へ既定値を割り当てる
pub mod fill;

/// 値を持つセルから周囲へ、距離に応じて減衰させた値を波及させる
pub mod diffuse;
