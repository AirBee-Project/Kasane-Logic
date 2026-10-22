# CONTRIBUTING

## コードの作成とレビュー

- AIが生成したコードは、内容を理解したうえでPull Requestを作成してください。
- 自分が作成したPRのレビューは、他の人に依頼してください。

## Git
- コミットメッセージは [Conventional Commits](https://www.conventionalcommits.org/ja/v1.0.0/) に従います。`<type>` と `<optional scope>` 以外は、日本語で記述してください。
- ブランチ名は [Conventional Branch](https://conventionalbranch.org/ja/) の仕様に従います。ただし、`main`、`master`、`develop` 等のブランチは対象外です。

## 分類ラベル

IssueとPRには、内容に合う分類ラベルを付けてください。分類ラベルは、リポジトリにある次のラベルから選びます。

| ラベル | 内容 |
|---|---|
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `hotfix` | 本番環境の緊急修正 |
| `bug` | バグ報告 |
| `perf` | パフォーマンス向上 |
| `refactor` | 挙動を変えないコード整理 |
| `style` | 挙動を変えない整形のみ |
| `test` | テストの追加や修正 |
| `docs` | ドキュメントのみ |
| `build` | ビルド関連 |
| `chore` | 他に当てはまらない雑務 |

## Pull Request

### 作成からマージまでの流れ

1. `main` 向けのPRを作成し、分類ラベルを付けます。
2. レビュー可能な状態になったら、必ずレビュアーにリクエストを送ってください。誰を指定するか分からない場合は、`Tomoro0726` を指定してください。
3. レビューで承認されたらマージします。

PRはマージコミットで取り込むため、PR内の各コミットのメッセージがそのままリリースの判定と CHANGELOG に使われます。マージ前に、各コミットの type が変更内容に合っているかを確かめてください。

### 破壊的変更の検証

PRを作成・更新すると、CI が `cargo semver-checks` で `main` と公開 API を比べます。後方互換のない変更が見つかると CI が失敗します。

意図して後方互換のない変更を入れる場合は、コミットの type に `!` を付ける（例: `feat!: ...`）か、`BREAKING CHANGE:` フッターを書いてください。そのようなコミットがあるPRでは検証をスキップし、release-plz がそのコミットをもとにバージョンを上げます。

挙動だけが変わる変更や、`spatial-id-derive` が生成するコードの変更は検出できません。テストとレビューで確かめてください。

## バージョンとリリース

リリースは [release-plz](https://release-plz.dev/) で自動化しています。crates.io には公開せず、git のタグと GitHub Release だけを作ります。

1. `main` にPRがマージされると、release-plz がリリースPR（ブランチ名は `release-plz-` で始まる）を作成、または更新します。リリースPRでは、`Cargo.toml`、`spatial-id-derive/Cargo.toml`、`CHANGELOG.md` が更新されます。
2. リリースPRをマージすると、タグ `vX.Y.Z` と GitHub Release が作られます。

バージョンは手動で変更しないでください。release-plz が、前回のリリースからのコミットの type と `cargo semver-checks` の結果をもとに、[Cargo SemVer](https://doc.rust-lang.org/cargo/reference/semver.html) に従って上げます。

### unstable な API の書き方

`unstable` feature の API は SemVer の対象外で、`cargo semver-checks` の検証からも外れます。docs.rs では全 feature を有効にしてビルドするので、unstable な API も通常の API と同じドキュメントに載り、「Available on crate feature `unstable` only.」と表示されます。

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
