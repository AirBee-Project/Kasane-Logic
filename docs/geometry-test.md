# geometry テストガイドライン

`geometry` モジュール内の各図形型（`Line`, `Triangle`, `Polygon`, `Sphere`, `Solid`, `Cylinder`, `Tube`）に対する変換テストの方針と運用手順を記述する。

## テストの方針

`CoverSingleIds::cover_single_ids()` などの変換関数は出力する空間IDの順序を保証しない。そのため、テストでは出力をソートしてからスナップショットと比較する。

スナップショットには [insta](https://docs.rs/insta) クレートを使用する。

### スナップショットを使う理由

空間IDへの変換結果は数値的に正しいかどうかをアサーションで書くより、スナップショットで「この入力に対してこの出力が得られる」という事実を固定する方が保守しやすい。計算ロジックの変更によって意図せず出力が変わった場合、差分として可視化できる。

## ファイル構成

各図形のフォルダ内に `tests.rs` を置く。

```
src/geometry/shape/
  line/
    mod.rs           # #[cfg(test)] mod tests; の宣言を含む
    impls.rs
    tests.rs         # スナップショットテスト
    snapshots/       # insta が自動生成するスナップショットファイル
  triangle/
    tests.rs
    snapshots/
  polygon/
    tests.rs
    snapshots/
  sphere/
    tests.rs
    snapshots/
  solid/
    tests.rs
    snapshots/
  cylinder/
    tests.rs
    snapshots/
  tube/
    tests.rs
    snapshots/
```

`mod.rs` での宣言:

```rust
#[cfg(test)]
mod tests;
```

## tests.rs の構成

テストはメソッド名をモジュール名にして、その内部にテスト関数を並べる。

```rust
// tests.rs の構成例（Line の場合）
use crate::{Coordinate, CoverSingleIds, Line};

fn sorted_ids(line: &Line, z: u8) -> Vec<String> {
    let mut ids: Vec<String> = line
        .cover_single_ids(z)
        .unwrap()
        .map(|id| id.to_string())
        .collect();
    ids.sort();
    ids
}

mod cover_single_ids {
    use super::*;

    #[test]
    fn flat_segment_at_z20() { ... }

    #[test]
    fn vertical_segment_at_z20() { ... }
}
```

新しいトレイトのテストを追加するときは、同じ `tests.rs` に `mod cover_range_ids { ... }` のように別モジュールを追加する。

### 命名規則

| 項目 | 規則 | 例 |
|------|------|-----|
| モジュール名 | テスト対象のメソッド名（snake_case） | `cover_single_ids` |
| テスト関数名 | `{図形の特徴}_at_z{ズームレベル}` | `flat_segment_at_z25` |
| ヘルパー関数名 | `sorted_ids` など動作を表す動詞句 | `sorted_ids` |


## テストの実行方法

### 通常実行

```sh
cargo test --lib -- geometry::shape
```

### 初回スナップショット生成・更新

スナップショットが存在しない、または更新が必要な場合:

```sh
# テストを実行して .snap.new ファイルを生成
cargo test --lib -- geometry::shape

# 差分を確認してすべて承認
cargo insta accept

# または対話的にレビュー
cargo insta review
```

> [!NOTE]
> `cargo insta` は `cargo install cargo-insta` でインストールする。

### スナップショットの確認

生成されたスナップショットファイル（`snapshots/*.snap`）はリポジトリにコミットする。`.snap.new` ファイルはコミットしない（`.gitignore` で除外推奨）。

スナップショットのファイル名はテストの完全なパスに対応している。例:

```
src/geometry/shape/line/snapshots/
  kasane_logic__geometry__shape__line__tests__cover_single_ids__flat_segment_at_z20.snap
```

## スナップショット更新のワークフロー

変換ロジックを修正した場合、関連するスナップショットが更新される可能性がある。

1. `cargo test --lib -- geometry::shape` を実行して差分を確認する
2. `cargo insta review` で変更内容を1件ずつ確認する
3. 意図した変更であれば `accept`、意図しない変更であれば実装を修正する
4. 承認済みの `.snap` ファイルをコミットする

> [!IMPORTANT]
> スナップショットの変更は必ずレビューしてからコミットすること。自動的に `accept` すると意図しない退行を見逃す恐れがある。
