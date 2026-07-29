//! ディスク形式（`MapArena` の rkyv バイト列）の golden テスト。
//!
//! # 何を守っているか
//!
//! [`SpatialIdMap::to_bytes`](crate::SpatialIdMap) の出力バイト列を固定する。目的は2つ：
//!
//! 1. **`FlexTreeCore` / `Node` にフィールドを足してもディスク形式が変わらない**ことの保証。
//!    書き込み側の `build_node` は `Node::Branch { level, lower_child, upper_child, .. }` と
//!    `..` を使うため、キャッシュ用フィールドを足しても**無警告で**無視される。
//!    それが意図どおり（=バイト列不変）であることを機械的に確認できるのはこのテストだけ。
//!
//! 2. **`MapArena` / `ArenaNode` をうっかり変えたら落ちる**こと。
//!    こちらはディスク形式そのものなので、変更するなら移行を計画しなければならない。
//!
//! # 落ちたときの判断
//!
//! - `FlexTreeCore` / `Node` にキャッシュを足しただけなら → **バグ**。意図せず永続化されている
//! - `MapArena` / `ArenaNode` を変えたなら → **想定どおり**。形式バージョンを上げ、
//!   移行方針を決めてからスナップショットを更新する
//! - rkyv のバージョンを上げたなら → **想定どおり**。rkyv のレイアウトが変わった可能性がある。
//!   既存データを読めるか確認してからスナップショットを更新する
//!
//! 直列化は決定的（値辞書は `BTreeMap`、ノード配列は後行順）なので、同じ入力なら常に同じバイト列になる。

#[cfg(all(test, feature = "persist"))]
mod golden_tests {
    use crate::{FlexId, RangeId, SingleId, SpatialIdMap};
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// `MapArena` の全フィールドを踏む標本。
    ///
    /// - `lower_root` / `upper_root`: f<0 と f>=0 の両半空間にセルを置く
    /// - `shard`: シャード領域を持たせる
    /// - `nodes`: 分岐と葉の両方
    /// - `dictionary`: 重複する値（共有される）と単独の値
    fn sample() -> SpatialIdMap<Vec<u8>> {
        // 上位ズームのシャード領域に閉じたマップ
        let region = FlexId::new(0, 0, 2, 0, 2, 0).unwrap();
        let mut m = SpatialIdMap::new_in_shard(region);

        // 上半分（f >= 0）: 同じ値を2セル → 辞書エントリは共有される
        m.insert(SingleId::new(4, 0, 0, 0).unwrap(), alloc::vec![0xAA]);
        m.insert(SingleId::new(4, 1, 1, 0).unwrap(), alloc::vec![0xAA]);
        // 上半分: 別の値
        m.insert(SingleId::new(4, 2, 0, 1).unwrap(), alloc::vec![0xBB, 0xCC]);
        // 下半分（f < 0）
        m.insert(SingleId::new(4, -1, 1, 1).unwrap(), alloc::vec![0xDD]);
        // 範囲挿入（畳まれて浅い部分木になる）
        m.insert(
            RangeId::new(4, [3, 4], [2, 3], [2, 3]).unwrap(),
            alloc::vec![0xEE],
        );

        m
    }

    /// バイト列を1行16バイトの16進ダンプへ。差分が読める形にする。
    fn hex_dump(bytes: &[u8]) -> String {
        let mut out = String::new();
        for (i, chunk) in bytes.chunks(16).enumerate() {
            out.push_str(&format!("{:04x}  ", i * 16));
            for b in chunk {
                out.push_str(&format!("{b:02x} "));
            }
            out.push('\n');
        }
        out
    }

    /// ディスク形式のバイト列そのものを固定する。
    #[test]
    fn persisted_bytes_are_stable() {
        let bytes = sample().to_bytes().unwrap();
        insta::assert_snapshot!(hex_dump(&bytes));
    }

    /// バイト長も別途固定する（スナップショット差分より先に気付けるように）。
    #[test]
    fn persisted_byte_length_is_stable() {
        let bytes = sample().to_bytes().unwrap();
        insta::assert_snapshot!(bytes.len());
    }

    /// 標本が往復できること（golden が「読めないバイト列」を固定してしまう事故を防ぐ）。
    ///
    /// 保存していない導出値（`leaf_count` / `max_zoom` / `split_mask`）が
    /// 読み込み時に正しく畳み直されることの確認も兼ねる。
    #[test]
    fn sample_round_trips() {
        let original = sample();
        let bytes = original.to_bytes().unwrap();
        let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes).unwrap() };

        let mut before: Vec<(FlexId, Vec<u8>)> =
            original.iter().map(|(id, v)| (id, v.clone())).collect();
        let mut after: Vec<(FlexId, Vec<u8>)> =
            restored.iter().map(|(id, v)| (id, v.clone())).collect();
        before.sort();
        after.sort();

        assert_eq!(before, after, "往復で内容が変わっている");
        assert_eq!(original.count(), restored.count(), "セル数が変わっている");
    }

    /// 同じ入力なら常に同じバイト列になること（golden テストの前提）。
    #[test]
    fn serialization_is_deterministic() {
        let a = sample().to_bytes().unwrap();
        let b = sample().to_bytes().unwrap();
        assert_eq!(a, b, "直列化が決定的でない。golden テストが成立しない");
    }
}
