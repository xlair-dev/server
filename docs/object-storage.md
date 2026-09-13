# R2 の設定

server は Cloudflare R2 に jacket 画像を保存する。

オブジェクトは `jackets/{musicId}/{sha256}.png` として保存する。

R2 に `assets` バケットを作成し、公開 URL を設定する。API サーバーの `.env` に次を追加する。

```env
R2_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
R2_BUCKET=assets
R2_ACCESS_KEY_ID=<access-key-id>
R2_SECRET_ACCESS_KEY=<secret-access-key>
R2_PUBLIC_BASE_URL=https://assets.xlair.dev
```

`R2_ACCESS_KEY_ID` と `R2_SECRET_ACCESS_KEY` には、対象バケットへの読み書き権限を持つ R2 API トークンを使用する。

## API Token の発行

Cloudflare Dashboard の R2 Overview から API Tokens を開き、次の設定で API Token を作成する。

- Permission: `Object Read & Write`
- Apply to specific buckets only: `assets`

作成後に表示される Access Key ID と Secret Access Key を `.env` に設定する。Secret Access Key は作成後に再表示できない。

## カスタムドメイン

R2 バケットの Settings > Custom Domains から `assets.xlair.dev` を追加する。ドメイン接続後、次を設定する。

```env
R2_PUBLIC_BASE_URL=https://assets.xlair.dev
```

カスタムドメインは R2 オブジェクトを公開するために使用する。ドメインの DNS zone は R2 バケットと同じ Cloudflare アカウントに登録されている必要がある。
