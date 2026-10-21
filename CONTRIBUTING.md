# CONTRIBUTING

## コードの作成とレビュー

- AIが生成したコードは、内容を理解したうえでPull Requestを作成してください。自分がPRを出したコードについては、他人にしてください。

## Git
- コミットメッセージは [Conventional Commits](https://www.conventionalcommits.org/ja/v1.0.0/) に従います。`<type>` と `<optional scope>` 以外は、日本語で記述してください。
- ブランチ名は Conventional Branch の仕様に従います。ただし、`main`、`master`、`develop` 等のブランチは対象外です。
- PRがレビュー可能な状態になったら、必ずレビュアーにリクエストを送ってください。誰を指定するか分からない場合は、`Tomoro0726` を指定してください。

## Rustの仕様とバージョンアップ
- バージョンアップは、必ず [Cargo SemVer](https://doc.rust-lang.org/cargo/reference/semver.html) に従ってください。
- 実験的な機能を追加する場合は、`unstable`にしてください。
