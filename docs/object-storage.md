# R2 の設定

server は Cloudflare R2 に jacket 画像を保存する。

R2 に `assets` バケットを作成し、公開 URL を設定する。API サーバーの `.env` に次を追加する。

`<account-id>` は Cloudflare Dashboard の Account ID であり、バケット名ではない。

```env
R2_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
R2_BUCKET=assets
R2_ACCESS_KEY_ID=<access-key-id>
R2_SECRET_ACCESS_KEY=<secret-access-key>
R2_PUBLIC_BASE_URL=https://assets.xlair.dev
```

`R2_ACCESS_KEY_ID` と `R2_SECRET_ACCESS_KEY` には、対象バケットへの読み書き権限を持つ R2 API トークンを使用する。
