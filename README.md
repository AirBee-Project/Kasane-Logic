# Kasane-Logic

Kasane-Logic は、解析解を用いた図形から空間IDへの高精度な変換アルゴリズムを Rust で実装したコアライブラリです。 MIT ライセンスで公開しており、空間 ID に関する豊富な型定義を活用して、第三者が空間ID関連アプリケーションを実装できる設計になっています。

## 特徴

- 座標系: `Coordinate`（緯度・経度・高度）と `Ecef`（地心直交座標）を提供
- 空間ID型: `SingleId` / `RangeId` / `FlexId` を提供
- 時空間ID: 仕様の `{z}/{f}/{x}/{y}_{i}/{t}` に対応（`with_time()` で時刻を付与）
- 図形入力: `Polygon` / `Solid` から空間ID集合への変換を提供
- 集合演算補助: `RoaringTreemap` と `fast_intersect` を提供

## インストール

```toml
[dependencies]
kasane-logic = "0.1.3"
```

### Feature

本ライブラリは、用途に合わせて内部の依存クレートや機能をON/OFFできる `features` を提供しています。

#### デフォルトで有効な機能

- **`random`**
  - **機能**: 内部で `rand` クレートを使用し、各空間IDにおいてランダムなIDの生成機能を有効化します。
  - **用途**: テストデータの自動生成やシチュエーションのシミュレーションにおいて利用します。

- **`temporal_id`**
  - **機能**: 時間軸を考慮した時空間ID (`{z}/{f}/{x}/{y}_{i}/{t}`) を有効化します。`SingleId` / `RangeId` に `with_time()` で時刻を付けられるようになり、コレクション（`SpatialIdSet` など）も時間を第4の軸として索引します。
  - **用途**: 時間を持たない（純粋な3D/2Dの）空間のみを扱う場合は不要です。OFFにすると時間の情報がゼロサイズになり、木の分割軸も F/X/Y の3軸に戻るため、メモリと計算のオーバーヘッドが消えます。

#### デフォルトで無効な機能

- **`persist`**
  - **機能**: `rkyv` によるバイト列への永続化（`SpatialIdMap::to_bytes` / `from_bytes`）を有効化します。
  - **注意**: 保存形式は `temporal_id` の有無で異なります（3軸=バージョン1 / 4軸=バージョン2）。異なる構成で書いたファイルは読み込み時に弾かれます。

#### デフォルト機能をOFFにして軽量化する方法

純粋な空間演算のみが必要で、シリアライズやランダム生成ライブラリの依存ごと削りたい場合は `default-features = false` を指定します。

```toml
[dependencies]
kasane-logic = { version = "0.1", default-features = false, features = ["std"] }
```

※ その後、必要な機能だけを手動で追加できます。 例: `kasane-logic = { version = "0.1", default-features = false, features = ["std", "rayon", "json"] }`

## クイックスタート

### 1. 座標から `SingleId` へ変換

```rust
use kasane_logic::{Coordinate, SingleId};

fn main() {
  let coord = Coordinate::new(35.681236, 139.767125, 12.0).unwrap();
  let id: SingleId = coord.to_single_id(18).unwrap();

  println!("{}", id); // 例: 18/.../.../...
}
```

### 2. `RangeId` を離散セルに展開

```rust
use kasane_logic::{RangeId, SpatialId};

fn main() {
  let range = RangeId::new(5, [-1, 1], [2, 3], [4, 4]).unwrap();
  let cells: Vec<_> = range.single_ids().collect();

  assert_eq!(cells.len(), 6); // F:3 x X:2 x Y:1
}
```

### 3. 閉じた立体 (`Solid`) を空間ID化

```rust
use kasane_logic::{Coordinate, Solid};

fn main() {
  let a = Coordinate::new(35.0, 139.0, 0.0).unwrap();
  let b = Coordinate::new(35.0, 139.001, 0.0).unwrap();
  let c = Coordinate::new(35.001, 139.0, 0.0).unwrap();
  let d = Coordinate::new(35.0, 139.0, 20.0).unwrap();

  // 四面体を構成
  let surfaces = vec![
    vec![a, b, c],
    vec![a, b, d],
    vec![b, c, d],
    vec![c, a, d],
  ];

  let solid = Solid::new(surfaces, 0.01).unwrap();
  let voxels: Vec<_> = solid.single_ids(18).unwrap().collect();

  assert!(!voxels.is_empty());
}
```

## 主な型

- `SingleId`: 1セル（時間も1セル）を表す最小単位の時空間ID。仕様の `{z}/{f}/{x}/{y}_{i}/{t}` に対応
- `RangeId`: 連続する直方体領域（時間も範囲）を表す時空間ID
- `FlexId`: 各軸で異なるズーム粒度を扱える拡張ID（コレクション内部のアドレス）
- `Interval`: 時間間隔 `{i}`（秒数）。時刻そのものは各IDが `interval()` / `t()` で持つ
- `Coordinate`: 緯度・経度・高度の地理座標
- `Ecef`: 地心直交座標
- `Polygon` / `Solid`: 図形から空間IDへ変換するための形状型

## 開発者向けコマンド

- テスト: `cargo test`
- Docテスト: `cargo test --doc`

## ライセンス

MIT License
