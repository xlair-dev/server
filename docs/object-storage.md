# Object Storage の設定

server は Cloudflare R2 に楽曲のアセットを保存する。R2 バケットは非公開で運用し、読み書きともに server の API 経由で行う。R2 の公開アクセスやカスタムドメインは設定しない。

オブジェクトは次の形式で保存する。

- `jackets/{musicId}/{sha256}.png`
- `musics/{musicId}/{sha256}.wav`
- `sheets/{sheetId}/{sha256}.sus`

環境ごとに次の R2 バケットを作成する。

| 環境 | バケット |
| --- | --- |
| local | `xlair-assets-local` |
| staging | `xlair-assets-stg` |
| production | `xlair-assets-prod` |

使用する環境のバケット名を API サーバーの `.env` に設定する。

```env
R2_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
R2_BUCKET=xlair-assets-local
R2_ACCESS_KEY_ID=<access-key-id>
R2_SECRET_ACCESS_KEY=<secret-access-key>
```

`R2_ACCESS_KEY_ID` と `R2_SECRET_ACCESS_KEY` には、対象バケットへの読み書き権限を持つ R2 API トークンを使用する。

## API Token の発行

Cloudflare Dashboard の R2 Overview から API Tokens を開き、次の設定で API Token を作成する。

- Permission: `Object Read & Write`
- Apply to specific buckets only: 使用する環境のバケット

作成後に表示される Access Key ID と Secret Access Key を `.env` に設定する。Secret Access Key は作成後に再表示できない。
