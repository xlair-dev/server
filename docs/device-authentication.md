# 筐体の認証

筐体は Auth0 の共有 M2M Application で access token を取得し、XLAIR API に送信する。

## 設定

筐体の `.env` に次を設定する。

```text
AUTH0_DOMAIN=dev-2dn3mvmvr8tccoss.us.auth0.com
AUTH0_CLIENT_ID=<共有 M2M Application の Client ID>
AUTH0_CLIENT_SECRET=<共有 M2M Application の Client Secret>
AUTH0_AUDIENCE=https://api.xlair.dev
```

Client Secret は provisioning 時に設置し、リポジトリやログへ保存しない。

## token の取得

Auth0 の token endpoint に Client Credentials Grant を要求する。

```sh
curl --request POST \
  --url https://dev-2dn3mvmvr8tccoss.us.auth0.com/oauth/token \
  --header 'content-type: application/json' \
  --data '{
    "client_id": "<共有 M2M Application の Client ID>",
    "client_secret": "<共有 M2M Application の Client Secret>",
    "audience": "https://api.xlair.dev",
    "grant_type": "client_credentials"
  }'
```

レスポンスの `access_token` を API 呼び出しに使用する。token の有効期限が切れた場合は再取得する。

## API の呼び出し

```http
Authorization: Bearer <device access token>
```

筐体は `deviceAuth` が定義された endpoint を利用できる。一般ユーザーは Auth0 に登録せず、card ID を API の入力として送信する。

## Secret の更新

漏洩が疑われる場合は Auth0 で Client Secret を再発行し、全筐体へ新しい Secret を再配置する。再配置が完了するまで旧 Secret を失効させるタイミングは運用担当者が判断する。
