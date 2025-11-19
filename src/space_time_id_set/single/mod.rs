//Singleは特殊なDirである。
//SpaceTimeIdはSetへの挿入時に各次元のズームレベルが異なるIDの集合に変換される。
// それらを数値情報、Bitの情報などへ変換するための関数が入っている。

pub mod convert_bitvec_f;
pub mod convert_bitvec_xy;
pub mod convert_single_f;
pub mod convert_single_xy;
pub mod invert_bitvec_f;
pub mod invert_bitvec_xy;
