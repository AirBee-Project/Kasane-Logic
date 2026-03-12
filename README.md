# Kasane-Logic

Kasane-Logic は、解析解を用いた図形から空間IDへの高精度な変換アルゴリズムを Rust で実装したコアライブラリです。 MIT ライセンスで公開しており、空間 ID に関する豊富な型定義を活用して、第三者が空間ID関連アプリケーションを実装できる設計になっています。

## 特徴

- 座標系: `Coordinate`（緯度・経度・高度）と `Ecef`（地心直交座標）を提供
- 空間ID型: `SingleId` / `RangeId` / `FlexId` を提供
- 時間ID: `TemporalId` による時間区間表現を提供
- 図形入力: `Polygon` / `Solid` から空間ID集合への変換を提供
- 集合演算補助: `RoaringTreemap` と `fast_intersect` を提供

## インストール

```toml
[dependencies]
kasane-logic = "0.1.3"
```

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

- `SingleId`: 1セルを表す最小単位の空間ID
- `RangeId`: 連続する直方体領域を表す空間ID
- `FlexId`: 各次元で異なるズーム粒度を扱える拡張空間ID
- `TemporalId`: 時間区間を表すID
- `Coordinate`: 緯度・経度・高度の地理座標
- `Ecef`: 地心直交座標
- `Polygon` / `Solid`: 図形から空間IDへ変換するための形状型

## 開発者向けコマンド

- テスト: `cargo test`
- Docテスト: `cargo test --doc`

## ライセンス

MIT License
