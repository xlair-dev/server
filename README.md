# api-server

API server for XLAIR

## デプロイ手順 (オンプレ)

1. `.env.prod.example` を参考に本番用 `.env` を作成します。`APP_IMAGE` と `IMAGE_TAG` を必要に応じて指定してください。
2. GHCR 上の公開イメージを利用するホストで次を実行します。
   ```sh
   docker compose --env-file ./.env -f compose.prod.yml pull
   docker compose --env-file ./.env -f compose.prod.yml up -d --wait
   ```
3. スキーマ変更がある場合は、アプリ再起動前に以下でマイグレーションを実行します。
   ```sh
   docker compose --env-file ./.env -f compose.prod.yml run --rm migrator migrate up
   ```

## CI での想定フロー

- `Dockerfile.prod` を利用し、`ghcr.io/xlair-dev/xlair-api:<git-sha>` のようなタグで公開イメージをビルド・push します。
- CI からはオンプレ環境へのデプロイは行わず、イメージ公開をもってリリースとし、運用担当者が上記手順でデプロイします。

## 環境変数

- 開発環境は `.env.dev.example`、本番環境は `.env.prod.example` を参照してください。`.env` はアプリケーションと `compose.prod.yml` の双方から読み込まれる前提です。
