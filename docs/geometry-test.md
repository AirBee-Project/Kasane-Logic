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

## OSごとの浮動小数点誤差による非決定性のテスト

`Coordinate`（地理座標）や `Ecef`（地心直交座標）などの浮動小数点演算を伴うモジュールでは、計算の最終桁（ULP単位の微小誤差）や `.floor()` 切り捨て時の境界において、OS依存の計算誤差が生じることがあります。

この非決定性を検証し、マルチプラットフォームにおける潜在的な挙動の差異を検出・可視化するため、以下のテストモジュールにおいて `insta` スナップショットテストを配置しています。

### テスト対象とファイル構成

1. **`Coordinate` の投影境界誤差テスト**
   * **場所:** [src/geometry/point/coordinate/tests.rs](file:///e:/repo/Kasane-Logic/src/geometry/point/coordinate/tests.rs)
   * **概要:** 緯度 `40.97989806962012` はズームレベル $z=5$ のとき、メルカトル投影 $y$ がちょうど `12.0` の境界に極めて近くなります。Windows (MSVC) では `12` になりますが、Linux (glibc) や macOS では `11` になるため、スナップショット不一致で CI が落ちます。

2. **`Ecef` の逆変換収束誤差テスト**
   * **場所:** [src/geometry/point/ecef/tests.rs](file:///e:/repo/Kasane-Logic/src/geometry/point/ecef/tests.rs)
   * **概要:** `try_from` による Bowring の反復収束法を用いた測地座標変換において、OSごとの標準数学ライブラリ（`libm`）の僅かな差異から生じる微小な小数点以下の桁（最下位ビット付近）のズレを `insta` の全桁デバッグ表示でキャッチします。

### CI/CD（GitHub Actions）での失敗の確認

`.github/workflows/main-merge-tests.yml` は `ubuntu-latest` (Linux), `macos-latest` (macOS), `windows-latest` (Windows) のマルチOS環境で並列して `cargo test` を実行しています。

Windows上で生成・コミットされたスナップショット（例: `y: 12`）に対して、CI上の `ubuntu-latest` や `macos-latest` でテストが走ると `insta` のスナップショット比較が不一致を検出し、**CI/CDビルドが自動的に失敗（テスト赤）**します。これにより、マルチプラットフォームでの動作差異があることを明確に示すことができます。

