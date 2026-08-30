# api-server

API server for XLAIR

## ローカル開発

通常の起動ではDBコンテナだけを起動します。マイグレーションは必要なときに明示的に実行してください。

```sh
docker compose up -d
docker compose --profile migration run --rm migrator up
cargo run -p presentation
```

マイグレーションを追加した場合も、同じコマンドで未適用分だけが適用されます。開発用DBを作り直す場合に限り、次を実行してください。

```sh
docker compose --profile migration run --rm migrator refresh
```

## デプロイ手順 (オンプレ)

1. `.env.prod.example` を参考に本番用 `.env` を作成します。
2. GHCR 上の公開イメージを利用するホストで次を実行します。
   ```sh
   docker compose --env-file ./.env -f compose.prod.yml pull
   docker compose --env-file ./.env -f compose.prod.yml up -d --wait
   ```
3. スキーマ変更がある場合は、アプリ再起動前に以下でマイグレーションを実行します。
   ```sh
   docker compose --env-file ./.env -f compose.prod.yml --profile migration run --rm migrator up
   ```

## CI での想定フロー

- `Dockerfile.prod` を利用し、`ghcr.io/xlair-dev/xlair-api:<git-sha>` のようなタグで公開イメージをビルド・push します。
- CI からはオンプレ環境へのデプロイは行わず、イメージ公開をもってリリースとし、運用担当者が上記手順でデプロイします。

## 環境変数

- 開発環境は `.env.dev.example`、本番環境は `.env.prod.example` を参照してください。`.env` はアプリケーションと `compose.prod.yml` の双方から読み込まれる前提です。
