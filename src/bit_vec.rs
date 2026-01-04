//! ビット列を用いた時空間ID階層管理モジュール
//!
//! このモジュールは、時空間IDの各次元の階層構造をビット列で表現し、
//! 効率的なビット操作を提供します。

use crate::id::space_id::constants::F_MAX;
use crate::id::space_id::segment::Segment;
use core::fmt;

/// ビット列を用いて時空間IDの各次元の階層構造を管理する
///
/// 内部的にはバイト配列として保持し、階層ごとのビット操作を効率的に行う
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BitVec(pub(crate) Vec<u8>);

impl BitVec {
    /// `Vec<u8>` から BitVec を生成
    pub fn from_vec(v: Vec<u8>) -> Self {
        BitVec(v)
    }

    /// スライスから BitVec を生成
    pub fn from_slice(s: &[u8]) -> Self {
        BitVec(s.to_vec())
    }

    /// 空の BitVec を生成
    pub fn new() -> Self {
        BitVec(Vec::new())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }

    /// self の全ての上位階層を列挙して返す
    ///
    /// 返される Vec は、上位階層の BitVecを順番に格納
    /// 例えば self = [10 11 10 00] の場合、返り値は [[10 00], [10 11 00],[10 11 10 00]] となる。
    pub fn ancestors(&self) -> Vec<BitVec> {
        let mut result: Vec<BitVec> = vec![];
        for byte in &self.0 {
            for i in 0..4 {
                let masked: u8 = byte & (0b11000000 >> 2 * i);

                if masked == 0 {
                    break;
                }

                match result.last() {
                    Some(v) => {
                        let mut copy = v.clone();
                        if i == 0 {
                            copy.0.push(masked);
                        } else if let Some(last) = copy.0.last_mut() {
                            *last |= masked;
                        }
                        result.push(copy);
                    }
                    None => result.push(BitVec(vec![masked])),
                }
            }
        }
        result
    }

    /// 最下層の2ビットペアが表す位置を反転する:
    /// - `10` → `11`
    /// - `11` → `10`
    pub fn flip_lowest_layer(&mut self) {
        if let Some(last) = self.0.last_mut() {
            for i in 0..=3 {
                let mask = 0b00000011 << (i * 2);
                let masked = *last & mask;

                match masked {
                    v if v == (0b00000010 << (i * 2)) => {
                        *last |= 0b00000001 << (i * 2); // 10 -> 11
                        break;
                    }
                    v if v == (0b00000011 << (i * 2)) => {
                        *last ^= 0b00000001 << (i * 2); // 11 -> 10
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    /// self と other の階層構造の関係を返す
    ///
    /// 返り値は `other` の `self` に対する関係を表す：
    /// - `Ancestor`: `other` が `self` の祖先（`other` が `self` を包含する）
    /// - `Descendant`: `other` が `self` の子孫（`self` が `other` を包含する）
    /// - `Equal`: `self` と `other` が同じ
    /// - `Unrelated`: 無関係
    pub fn relation(&self, other: &Self) -> BitVecRelation {
        let self_upper = self.upper_move();
        let other_upper = other.upper_move();

        if self == other {
            return BitVecRelation::Equal;
        }

        // self が other を包含する（self が親、other が子）
        if self < other && other < &self_upper {
            return BitVecRelation::Descendant;
        }

        // other が self を包含する（other が親、self が子）
        if other < self && self < &other_upper {
            return BitVecRelation::Ancestor;
        }

        // それ以外は無関係
        BitVecRelation::Unrelated
    }

    /// 最下層の有効な2ビット階層を取り除く
    ///
    /// - 最下層の最初に見つかった有効階層（00以外）を 00 にリセット
    /// - もし最後のu8が空になった場合、Vecから削除
    pub fn remove_lowest_layer(&mut self) {
        if let Some(last) = self.0.last_mut() {
            for i in 0..=3 {
                let mask = 0b00000011 << (i * 2);
                let masked = *last & mask;

                if masked != 0 {
                    *last &= !mask;

                    if *last == 0 {
                        self.0.pop();
                    }

                    break;
                }
            }
        }
    }

    pub(crate) fn to_segment_f(&self) -> Segment<i64> {
        let segment = self.to_segment_xy();

        if *self
            .0
            .first()
            .expect("Internal error: BitVec is empty in invert_bitmask_f")
            >= 0b11000000
        {
            return Segment {
                z: segment.z,
                dim: -(segment.dim as i64) + F_MAX[segment.z as usize],
            };
        } else {
            return Segment {
                z: segment.z,
                dim: segment.dim as i64,
            };
        }
    }

    pub(crate) fn to_segment_xy(&self) -> Segment<u64> {
        let bytes = &self.0;
        let total_bits = bytes.len() * 8;
        let total_layers = (total_bits + 1) / 2;

        let mut uxy: u64 = 0;
        let mut max_z: i32 = -1; // 見つかった最大のz

        // now_z=0 から順に処理
        for now_z in 0..total_layers {
            let index = (now_z * 2) / 8;
            let in_index = now_z % 4;

            let byte = bytes[index];
            let valid = (byte >> (7 - in_index * 2)) & 1;
            let branch = (byte >> (6 - in_index * 2)) & 1;

            if valid == 1 {
                max_z = now_z as i32;
                // now_z の位置に branch を配置
                uxy |= (branch as u64) << now_z;
            }
        }

        // uxy を反転（ビットの並びを逆にする）
        let final_z = max_z as u8;
        let mut reversed_uxy = 0u64;
        for i in 0..=final_z {
            let bit = (uxy >> i) & 1;
            reversed_uxy |= bit << (final_z - i);
        }

        Segment {
            z: final_z,
            dim: reversed_uxy,
        }
    }

    /// target の範囲から division の複数区間を順に除外し、
    /// 残った範囲の開始点のみ返す
    pub fn subtract_ranges(&self, division: &[BitVec]) -> Vec<BitVec> {
        let mut result = vec![self.clone()]; // self を target として使用

        for div in division {
            let mut next = vec![];

            for now in result.into_iter() {
                // div が now の範囲に含まれる場合 → 分割
                if div >= &now && &div <= &&now.upper_move() {
                    let div_clone = div.clone();
                    next.extend(BitVec::subtract_range(&now, &div_clone));
                } else {
                    next.push(now);
                }
            }

            result = next;
        }

        result
    }

    /// 単体範囲 BitVec から単体 sub を引いて残りの開始点を返す
    pub fn subtract_range(&self, sub: &BitVec) -> Vec<BitVec> {
        let mut result: Vec<BitVec> = vec![];
        let mut sub_clone = sub.clone();

        // sub が self と一致するまで leaf を操作
        while self != &sub_clone {
            sub_clone.flip_lowest_layer();
            result.push(sub_clone.clone());
            sub_clone.remove_lowest_layer();
        }

        result
    }

    /// # 概要
    /// この関数は、`BitVec` が表す階層ID（2ビット単位の階層構造）について、
    /// 同一の prefix に属する範囲の **右側開区間上限（upper move）** を計算して返します。
    ///
    /// # 動作例
    /// - 入力 `1010111011000000`→ 出力 `10101111`
    /// - 入力 `11101000`→ 出力 `11101100`
    pub fn upper_move(&self) -> BitVec {
        let mut copyed = self.clone();

        // upper_move 本体（2bit単位で後ろから走査）
        for (_byte_index, byte) in copyed.0.iter_mut().enumerate().rev() {
            for i in 0..=3 {
                let mask = 0b00000011 << (i * 2);
                let masked = *byte & mask;

                match masked >> (i * 2) {
                    0b10 => {
                        // 10 -> 11 で終了
                        *byte |= 0b01 << (i * 2);

                        // 末尾の空バイト削除
                        while let Some(&last) = copyed.0.last() {
                            if last == 0 {
                                copyed.0.pop();
                            } else {
                                break;
                            }
                        }
                        return copyed;
                    }
                    0b11 => {
                        // 11 -> 10 に戻して繰り上げ継続
                        *byte ^= 0b11 << (i * 2);
                    }
                    _ => {}
                }
            }
        }

        // ここでも末尾の空バイト削除
        while let Some(&last) = copyed.0.last() {
            if last == 0 {
                copyed.0.pop();
            } else {
                break;
            }
        }

        copyed
    }
}

impl fmt::Display for BitVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ebit in &self.0 {
            write!(f, "{:08b}", ebit)?;
        }
        Ok(())
    }
}

/// - `Ancestor`  
///     - `other` が `self` を包含する上位範囲（`other` is an ancestor of `self`）
///
/// - `Equal`  
///     - `self` と `other` が同じ範囲
///
/// - `Descendant`  
///     - `other` が `self` の下位範囲（`other` is a descendant of `self`）
///
/// - `Unrelated`  
///     - `self` と `other` は無関係
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitVecRelation {
    /// other が self を包含する（other is an ancestor of self）
    Ancestor,

    /// self と other が同じ範囲
    Equal,

    /// self が other を包含する（self is an ancestor of other, i.e., other is a descendant）
    Descendant,

    /// 無関係
    Unrelated,
}
