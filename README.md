# Kasane-Logic

Kasane-Logic は、解析解を用いた図形から空間IDへの高精度な変換アルゴリズムを Rust で実装したコアライブラリです。

## Features

- `unstable`: 実験的な機能を提供します。利用する場合は明示的に有効化してください。

### unstable な API の書き方

`unstable` feature の API は SemVer の対象外で、予告なく変更・削除されることがあります。docs.rs では全 feature を有効にしてビルドするので、unstable な API も通常の API と同じドキュメントに載り、「Available on crate feature `unstable` only.」と表示されます。

unstable な API には `#[cfg(feature = "unstable")]` を付けます。

```rust
#[cfg(feature = "unstable")]
pub fn experimental() {}
```

stable な API の引数、戻り値、trait の実装には、unstable な型を使わないでください。unstable な型を変更したときに、stable な API も壊れるためです。

docs.rs と同じ表示を手元で確かめるには、nightly で次のコマンドを実行します。

```sh
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features --open
```
