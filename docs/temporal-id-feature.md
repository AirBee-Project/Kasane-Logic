# TemporalId と feature フラグ

この文書は、`kasane_logic` における時間機能の扱いと、`feature` による切り替え仕様をまとめるものである。

## 1. 基本方針

- 時間機能は `temporal_id` feature で切り替える。
- feature が無効な場合は、空間のみを扱う利用を優先する。
- feature が有効な場合は、`TemporalId` を使った時空間 ID を扱える。
- 既存の空間 API は維持し、時間機能は追加機能として分離する。

## 2. `temporal_id` feature の意味

### 無効時

- `TemporalId` は全範囲を表す `WHOLE` 相当の扱いになる。
- `FlexId`、`SingleId`、`RangeId` の既定生成では時間は固定される。
- `FlexTreeCore` は現時点では `WHOLE` 以外の時空間 ID を受け付けない。
- 時間を使わない空間専用ビルドとして扱う。

### 有効時

- `TemporalId::new` による任意の時間区間を利用できる。
- `TemporalId::intersection` や `TemporalId::difference` を使った時間演算が有効になる。
- `FlexId`、`SingleId`、`RangeId` は保持した時間情報をそのまま伝播する。
- `FlexTreeCore` は時間付きの挿入を受け入れる前提に切り替えられる。

## 3. 時間 ID の役割

`TemporalId` は時間区間を表す型である。

- `WHOLE` は Unix 時間の全範囲を表す標準定数である。
- `new(i, [t1, t2])` は時間間隔 `i` と区間 `[t1, t2]` を受け取り、内部表現を正規化する。
- `is_whole()` はその値が全範囲かどうかを判定する。

## 4. 時間と空間の分離

このクレートでは、空間と時間を完全に同列では扱っていない。

- 空間 ID は `SingleId`、`RangeId`、`FlexId` を中心に扱う。
- 時間は `TemporalId` として別フィールドに保持する。
- `FlexTreeCore` は空間索引としての性格が強く、時間は現時点では制約付きでしか扱わない。

このため、時間機能を使うかどうかは feature で明示的に選ぶ必要がある。

## 5. ビルド時の使い分け

### 空間のみを使う

```toml
[features]
default = []
```

この場合は `temporal_id` feature を有効にしない。

### 時間も使う

```toml
[features]
default = []
temporal_id = []
```

利用時は `--features temporal_id` を付ける。

例:

```bash
cargo test --features temporal_id
cargo doc --features temporal_id --no-deps
```

## 6. 利用上の注意

- feature が無効なビルドでは、時間付きの挙動は抑制される。
- feature が有効なビルドでは、時間情報を保持した ID を前提に実装される。
- どちらのモードでも、公開 API の見え方を変えすぎないようにする。

## 7. 将来の拡張

今後、`FlexTreeCore` や関連コレクションが時間をネイティブに扱うようになったら、この文書を更新する。

そのときは次の項目を明確化する。

- 同一空間に複数の `TemporalId` を載せられるか。
- `get` / `remove` が時間をどう扱うか。
- `WHOLE` 専用の制約を残すか削除するか。
- feature 無効時の互換性をどこまで維持するか。
