バグ報告・機能提案・ドキュメント修正・テスト追加など、あらゆる貢献を歓迎します。

# 開発手順

1. **本リポジトリを fork する**
2. **ローカルで作業用 branch を作成**
   - ブランチ名は`<種別>/<短い説明>(-<issue番号>)`issue 番号は issue が存在する場合
   - 種別
     - `feat` → 新機能
     - `fix` → バグ修正
     - `docs` → ドキュメント修正
     - `refactor` → リファクタリング
     - `chore` → 雑務（CI 設定、依存パッケージ更新など）
   - 例:`feat/add-user-login-123`
3. **開発・commit**
   - commit メッセージは任意の短い説明
4. **GitHub に push**
5. **Pull Request**
   - タイトルはブランチ名を参照し、`[種別] #<issue番号> 短い説明`issue 番号は issue が存在する場合

# ベンチマーク

`/target/memori/report.html`に結果が出力される。

```sh
cargo bench
```
