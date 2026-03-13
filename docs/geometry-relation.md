# Geometry Relation

```mermaid
graph TB

Coordinate((Coordinate))
Line((Line))
Triangle((Triangle))
Polygon((Polygon))
Solid((Solid))
Sphere((Sphere))

subgraph Geometry
  direction LR
    subgraph Point [Point]
        Coordinate
        Ecef
    end

    subgraph Shape [Shape]
        Line
        Triangle
        Polygon
        Solid
        Sphere
    end

end

%%点の定義
Coordinate --> Ecef
Ecef -.-> Coordinate


%%点以外の型定義
Line -->|"Line → Coordinate×2"| Coordinate
Triangle -->|"Triangle → Line×3"| Line
Polygon -->|"Polygon → Triangle×N"| Triangle
Solid -->|"Solid → Polygon×N"| Polygon
```

# Trait `Shape`

Shapeは「空間ID以外で定義される地理空間情報」を指す。`Shape`では下記のメゾットを利用することができる。

- `center`
  - 重心を求めることができる。
- `single_ids`
  - 指定したズームレベルの空間IDに変換することができる。
- `range_ids`
  - 指定したズームレベルの空間IDの区間表現に変換することができる。
- `optimze_single_ids`
  - 最小個数を保証して、`SingleId`を出力する。
- `optimze_range_ids`
  - 最小個数を保証して、`RangeId`を出力する。

また、変換可能な全てのオブジェクトに対して、`Into<Box<dyn Iterator<Item = T>>> for K`を提供する。順番はなるべく意味を保つものにするが、その順序に正確な規則を保証しない。

---

例1:`Solid`は以下の型のイテレーターに変換できる。

- `Polygon`
- `Triangle`
- `Line`
- `Coordinate`

例2:`Triangle`は以下の型のイテレーターに変換できる。

- `Line`
- `Coordinate`

例3:`Line`は以下の型のイテレーターに変換できる。

- `Coordinate`

---

> [!NOTE]
> Geometryの関係は上記の図で表される。なお、内部的に保持している値ではなく、幾何学的な整合を優先して図が書かれているため、例外が存在する。例えば、実際には`Triangle`型は中に3つの`Coordinate`型を保持している。

# Trait `Point`

基本的に全てのGeometryは内部的には`Coordinate`の集合として表される。3次元空間上の点を表す型が持つ特徴である。現在はTrait関数は存在しないが、今後増える可能性がある。

## Type `Coordinate`

空間IDが定義される範囲内の「緯度/経度/高度」を表し、`Ecef`に必ず変換することができる。本ライブラリの性質上、大体のGeometryは`Coordinate`の集合で表現される。

## Type `Ecef`

制約のない地心直交座標系を表す。必ずしも`Coordinate`に**変換することができない**。
